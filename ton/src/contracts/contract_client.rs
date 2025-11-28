mod builder;
mod cache_stats;
pub mod contract_client_cache;
#[cfg(feature = "tonlibjson")]
pub mod tl_provider;

use crate::contracts::contract_client::builder::Builder;
use crate::contracts::contract_client::contract_client_cache::ContractClientCache;
use crate::emulators::emul_bc_config::EmulBCConfig;
use crate::emulators::tvm_emulator::*;
use crate::errors::{TonError, TonResult};
use crate::libs_dict::LibsDict;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::OnceCell;
use ton_core::cell::{TonCell, TonHash};
use ton_core::errors::TonCoreError;
use ton_core::traits::contract_provider::{TonContractState, TonProvider};
use ton_core::traits::tlb::TLB;
use ton_core::types::{TonAddress, TxLTHash};

/// Safe-guard to prevent infinite loops when loading libraries during emulation
const MAX_LIBS_PER_CONTRACT: usize = 100;

#[derive(Clone)]
pub struct ContractClient {
    inner: Arc<Inner>,
}

impl ContractClient {
    pub fn builder(provider: impl TonProvider) -> Builder { Builder::new(provider) }

    pub async fn get_contract(
        &self,
        address: &TonAddress,
        tx_id: Option<&TxLTHash>,
    ) -> TonResult<Arc<TonContractState>> {
        self.inner.cache.get_or_load_contract(address, tx_id).await
    }

    pub async fn emulate_get_method(
        &self,
        state: &TonContractState,
        method_id: i32,
        stack_boc: &[u8],
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
        let code_hash = TonCell::from_boc(code_boc.to_owned())?.hash()?.clone();

        let c7 = TVMEmulatorC7 {
            address: state.address.clone(),
            unix_time: SystemTime::now().duration_since(UNIX_EPOCH).map_err(TonCoreError::from)?.as_secs() as u32,
            balance: state.balance as u64,
            rand_seed: TonHash::ZERO,
            config: self.get_bc_config().await?.clone(),
        };

        let emul_data_boc = state.data_boc.as_ref().map(|x| x.as_slice()).unwrap_or(&[]);
        let mut emulator = TVMEmulator::new(code_boc, emul_data_boc, &c7)?;

        let libs = self.inner.cache.get_or_load_code_libs(code_hash.clone()).await?;
        let mut libs_dict = LibsDict::from(libs);
        if !libs_dict.is_empty() {
            emulator.set_libs(&libs_dict.to_boc()?)?;
        }
        let mut emul_response = emulator.run_get_method(method_id, stack_boc)?;
        let mut iteration = 0;
        while let Some(missing_lib_hash) = emul_response.missing_lib()? {
            iteration += 1;
            if iteration > MAX_LIBS_PER_CONTRACT {
                return Err(TonError::EmulatorTooManyLibraries(MAX_LIBS_PER_CONTRACT));
            }
            let Some(lib) = self.inner.cache.get_or_load_lib(missing_lib_hash.clone()).await? else {
                return Err(TonError::EmulatorMissingLibrary(missing_lib_hash));
            };
            self.inner.cache.update_code_libs(code_hash.to_owned(), missing_lib_hash.clone());

            libs_dict.insert(missing_lib_hash, lib.into());
            emulator.set_libs(&libs_dict.to_boc()?)?;
            emul_response = emulator.run_get_method(method_id, stack_boc)?;
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
    cache: Arc<ContractClientCache>,
    bc_config: OnceCell<EmulBCConfig>,
}
