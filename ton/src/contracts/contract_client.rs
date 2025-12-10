mod builder;
mod cache_stats;
pub mod contract_client_cache;
#[cfg(feature = "tonlibjson")]
pub mod tl_provider;

use crate::contracts::contract_client::builder::Builder;
use crate::contracts::contract_client::contract_client_cache::ContractClientCache;
use crate::emulators::emul_bc_config::EmulBCConfig;
use crate::emulators::emulator_pool::{EmulatorPool, PoolEmulationResponse, TVMRunGetMethodTask};
use crate::emulators::tvm_emulator::*;
use crate::errors::{TonError, TonResult};
use crate::libs_dict::LibsDict;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::OnceCell;
use tokio::try_join;
use ton_core::cell::{TonCell, TonCellUtils, TonHash};
use ton_core::errors::TonCoreError;
use ton_core::traits::contract_provider::{TonContractState, TonProvider};
use ton_core::traits::tlb::TLB;
use ton_core::types::{TonAddress, TxLTHash};

#[derive(Clone)]
pub struct ContractClient {
    inner: Arc<Inner>,
}

impl ContractClient {
    pub fn builder(provider: impl TonProvider) -> TonResult<Builder> { Builder::new(provider) }

    pub async fn get_contract(
        &self,
        address: &TonAddress,
        tx_id: Option<&TxLTHash>,
    ) -> TonResult<Arc<TonContractState>> {
        self.inner.cache.get_or_load_contract(address, tx_id).await
    }

    /// mc_seqno can be specified to run emulation in a specific blockchain state
    /// If mc_seqno is None, head state will be used
    /// Is not used yet
    pub async fn emulate_get_method<S: Into<Arc<Vec<u8>>>>(
        &self,
        state: &TonContractState,
        method_id: i32,
        stack_boc: S,
        _mc_seqno: Option<i32>,
    ) -> TonResult<TVMGetMethodSuccess> {
        let code_boc = match &state.code_boc {
            Some(boc) => boc,
            None => {
                return Err(TonError::TonContractNotFull {
                    address: state.address.clone(),
                    tx_id: Some(state.last_tx_id.clone()),
                    missing_field: "code".to_string(),
                });
            }
        };

        let code_cell = TonCell::from_boc(code_boc.to_owned())?;
        let data_cell = match &state.data_boc {
            Some(boc) => TonCell::from_boc(boc.to_owned())?,
            None => TonCell::empty().to_owned(),
        };
        let code_hash = TonCell::from_boc(code_boc.to_owned())?.hash()?.clone();
        let static_lib_ids = TonCellUtils::extract_lib_ids([&code_cell, &data_cell])?;

        let c7 = TVMEmulatorC7 {
            address: state.address.clone(),
            unix_time: SystemTime::now().duration_since(UNIX_EPOCH).map_err(TonCoreError::from)?.as_secs() as u32,
            balance: state.balance as u64,
            rand_seed: TonHash::ZERO,
            config: self.get_bc_config().await?.clone(),
        };

        let mut emul_task = TVMRunGetMethodTask {
            state: TVMState {
                code_boc: code_boc.to_owned(),
                data_boc: state.data_boc.as_ref().map(|x| x.to_owned()).unwrap_or(Arc::new(vec![])),
                c7,
                libs_boc: None,
                debug_enabled: None,
                gas_limit: None,
            },
            method: method_id.into(),
            stack_boc: stack_boc.into(),
        };

        let (mut emulation_libs, dyn_libs) = try_join!(
            self.inner.cache.get_or_load_libs(static_lib_ids),
            self.inner.cache.get_or_load_code_dyn_libs(code_hash.clone()),
        )?;
        emulation_libs.extend(dyn_libs);

        let mut libs_dict = LibsDict::from(emulation_libs);
        if !libs_dict.is_empty() {
            emul_task.state.libs_boc = Some(Arc::new(libs_dict.to_boc()?));
        }

        let emul_pool = &self.inner.emulator_pool;
        let emul_timeout = Some(self.inner.emulation_timeout);

        let mut emul_response = match emul_pool.exec(emul_task.clone(), emul_timeout).await? {
            PoolEmulationResponse::EmulGetMethod(resp) => resp,
            _ => return Err(TonError::Custom("Unexpected TVMEmulResponse".to_string())),
        };

        let mut iteration = 0;
        while let Some(missing_lib_hash) = emul_response.missing_lib()? {
            iteration += 1;
            if iteration > self.inner.max_dyn_libs_per_contract {
                return Err(TonError::EmulatorTooManyLibraries(self.inner.max_dyn_libs_per_contract));
            }
            let Some(lib) = self.inner.cache.get_or_load_lib(missing_lib_hash.clone()).await? else {
                return Err(TonError::EmulatorMissingLibrary(missing_lib_hash));
            };
            self.inner.cache.add_code_dyn_lib(code_hash.to_owned(), missing_lib_hash.clone());

            libs_dict.insert(missing_lib_hash, lib.into());
            emul_task.state.libs_boc = Some(Arc::new(libs_dict.to_boc()?));
            emul_response = match emul_pool.exec(emul_task.clone(), emul_timeout).await? {
                PoolEmulationResponse::EmulGetMethod(resp) => resp,
                _ => return Err(TonError::Custom("Unexpected TVMEmulResponse".to_string())),
            };
        }
        emul_response.into_success()
    }

    pub fn cache_stats(&self) -> HashMap<String, usize> { self.inner.cache.cache_stats() }

    async fn get_bc_config(&self) -> TonResult<&EmulBCConfig> {
        self.inner
            .bc_config
            .get_or_try_init(|| async {
                let config = self.inner.provider.load_bc_config(None).await?;
                EmulBCConfig::from_boc(&config)
            })
            .await
    }
}

struct Inner {
    provider: Arc<dyn TonProvider>,
    emulator_pool: EmulatorPool,
    emulation_timeout: Duration,
    cache: Arc<ContractClientCache>,
    bc_config: OnceCell<EmulBCConfig>,
    max_dyn_libs_per_contract: usize,
}
