mod builder;
mod cache_stats;
pub mod contract_client_cache;

use crate::contracts::contract_client::builder::Builder;
use crate::contracts::contract_client::contract_client_cache::ContractClientCache;
use crate::errors::{TonError, TonResult};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use ton_core::errors::TonCoreError;
use ton_core::traits::emulation_provider::{EmulationProvider, EmulatorGetMethodRequest, EmulatorGetMethodSuccess};
use ton_core::traits::state_provider::{ContractState, StateProvider};
use ton_core::types::{TonAddress, TxLTHash};

#[derive(Clone)]
pub struct ContractClient {
    inner: Arc<Inner>,
}

impl ContractClient {
    /// Creates a builder using the providers for blockchain state and TVM emulation.
    ///
    /// The result wrapper is retained for compatibility with the previous builder API.
    pub fn builder(
        state_provider: impl StateProvider,
        emulation_provider: impl EmulationProvider,
    ) -> TonResult<Builder> {
        Builder::new(state_provider, emulation_provider)
    }

    pub(super) async fn load_state(
        &self,
        address: &TonAddress,
        tx_id: Option<&TxLTHash>,
    ) -> TonResult<Arc<ContractState>> {
        self.inner.cache.get_or_load_contract(address, tx_id).await
    }

    pub(super) async fn emulate_get_method(
        &self,
        request: EmulatorGetMethodRequest,
    ) -> TonResult<EmulatorGetMethodSuccess> {
        let success = self
            .inner
            .emulation_provider
            .emulate_get_method(request, Some(self.inner.emulation_timeout))
            .await
            .map_err(restore_emulator_error)?;
        validate_emulation_success(success)
    }

    pub fn cache_stats(&self) -> HashMap<String, usize> { self.inner.cache.cache_stats() }
}

fn validate_emulation_success(success: EmulatorGetMethodSuccess) -> TonResult<EmulatorGetMethodSuccess> {
    if success.vm_exit_code != 0 && success.vm_exit_code != 1 {
        return Err(TonError::EmulatorEmulationError {
            vm_exit_code: Some(success.vm_exit_code),
            response_raw: success.raw_response.unwrap_or_default(),
        });
    }
    Ok(success)
}

fn restore_emulator_error(error: TonCoreError) -> TonError {
    let TonCoreError::BoxedError(error) = error else {
        return TonError::TLCoreError(error);
    };
    match error.downcast::<TonError>() {
        Ok(error) => *error,
        Err(error) => TonError::TLCoreError(TonCoreError::BoxedError(error)),
    }
}

struct Inner {
    emulation_provider: Arc<dyn EmulationProvider>,
    emulation_timeout: Duration,
    cache: Arc<ContractClientCache>,
}
