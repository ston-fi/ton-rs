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
use ton_core::traits::emulation_provider::{
    EmulationProvider, EmulatorContractState, EmulatorGetMethodRequest, EmulatorGetMethodSuccess,
};
use ton_core::traits::state_provider::{ContractState, StateProvider};
use ton_core::types::{TonAddress, TxLTHash};

#[derive(Clone)]
pub struct ContractClient {
    inner: Arc<Inner>,
}

impl ContractClient {
    /// Configures a client with separate state and emulation providers.
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
        mut request: EmulatorGetMethodRequest,
    ) -> TonResult<EmulatorGetMethodSuccess> {
        if self.inner.emulation_provider.requires_resolved_state() {
            request.contract_state = match request.contract_state {
                EmulatorContractState::Address { address, tx_id } => {
                    EmulatorContractState::Custom(self.load_state(&address, tx_id.as_ref()).await?)
                },
                state => state,
            };
        }

        let success = self
            .inner
            .emulation_provider
            .emulate_get_method(request, Some(self.inner.emulation_timeout))
            .await
            .map_err(restore_emulator_error)?;
        validate_emulation_success(success)
    }

    pub fn cache_stats(&self) -> HashMap<String, usize> {
        self.inner.cache.cache_stats()
    }
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
