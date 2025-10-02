mod cache_stats;
pub mod contract_client_cache;
#[cfg(feature = "tonlibjson")]
pub mod tl_provider;

use crate::contracts::client::contract_client_cache::ContractClientCache;
use crate::emulators::emul_bc_config::EmulBCConfig;
use crate::emulators::tvm_emulator::*;
use crate::errors::TonError;
use crate::libs_dict::LibsDict;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::OnceCell;
use ton_lib_core::cell::{TonCell, TonCellRef, TonCellUtils, TonHash};
use ton_lib_core::errors::TonCoreError;
use ton_lib_core::traits::contract_provider::{TonContractState, TonProvider};
use ton_lib_core::traits::tlb::TLB;
use ton_lib_core::types::{TonAddress, TxLTHash};

#[derive(Clone, Copy, Debug)]
pub struct ContractClientConfig {
    pub refresh_loop_idle_on_error: Duration,
    pub cache_capacity: u64,
    pub cache_ttl: Duration,
}

impl ContractClientConfig {
    pub fn new_no_cache(idle_on_error: Duration) -> Self {
        ContractClientConfig {
            refresh_loop_idle_on_error: idle_on_error,
            cache_capacity: 0,
            cache_ttl: Duration::from_secs(0),
        }
    }
}

#[derive(Clone)]
pub struct ContractClient(Arc<Inner>);

impl ContractClient {
    pub fn new(config: ContractClientConfig, data_provider: impl TonProvider) -> Result<Self, TonError> {
        let provider = Arc::new(data_provider);
        let inner = Inner {
            provider: provider.clone(),
            cache: ContractClientCache::new(config, provider.clone())?,
            bc_config: OnceCell::new(),
        };
        Ok(ContractClient(Arc::new(inner)))
    }

    pub async fn get_contract(
        &self,
        address: &TonAddress,
        tx_id: Option<&TxLTHash>,
    ) -> Result<Arc<TonContractState>, TonError> {
        self.0.cache.get_or_load_contract(address, tx_id).await
    }

    pub async fn emulate_get_method(
        &self,
        state: &TonContractState,
        method_id: i32,
        stack_boc: &[u8],
    ) -> Result<TVMGetMethodSuccess, TonError> {
        let code_boc = match &state.code_boc {
            Some(boc) => boc,
            None => {
                let err_msg = format!("code is None at state: {state:?}");
                return Err(TonCoreError::ContractError(err_msg).into());
            }
        };

        let data_boc = state.data_boc.as_deref().unwrap_or(&[]);

        let c7 = TVMEmulatorC7 {
            address: state.address.clone(),
            unix_time: SystemTime::now().duration_since(UNIX_EPOCH).map_err(TonCoreError::from)?.as_secs() as u32,
            balance: state.balance as u64,
            rand_seed: TonHash::ZERO,
            config: self.get_bc_config().await?.clone(),
        };

        let mut emulator = TVMEmulator::new(code_boc, data_boc, &c7)?;

        let code_cell = TonCell::from_boc(code_boc)?;
        let data_cell = TonCell::from_boc(data_boc)?;
        let lib_ids = TonCellUtils::extract_lib_ids([&code_cell, &data_cell])?;
        let libs_rsp = self
            .0
            .provider
            .load_libs(lib_ids.into_iter().collect(), state.mc_seqno)
            .await?
            .into_iter()
            .map(|(_, lib)| TonCellRef::from_boc(&lib))
            .collect::<Result<Vec<_>, _>>()?;

        if !libs_rsp.is_empty() {
            emulator.set_libs(&LibsDict::new(libs_rsp)?.to_boc()?)?;
        }
        emulator.run_get_method(method_id, stack_boc)
    }

    pub fn cache_stats(&self) -> HashMap<String, usize> { self.0.cache.cache_stats() }

    async fn get_bc_config(&self) -> Result<&EmulBCConfig, TonError> {
        self.0
            .bc_config
            .get_or_try_init(|| async {
                let config = self.0.provider.load_bc_config(None).await?;
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
