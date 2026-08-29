use crate::errors::TonCoreResult;
use crate::traits::state_provider::ContractState;
use crate::types::{TonAddress, TxLTHash};
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;

/// State source for get-method emulation.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum EmulatorContractState {
    /// Resolve the latest state or the state at the exact `tx_id`.
    Address {
        address: TonAddress,
        tx_id: Option<TxLTHash>,
    },
    /// Use caller-supplied state without copying the snapshot.
    Custom(Arc<ContractState>),
}

/// Complete TVM get-method request.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct EmulatorGetMethodRequest {
    /// Contract state used for execution.
    pub contract_state: EmulatorContractState,
    /// Numeric TVM method identifier.
    pub method_id: i32,
    /// Input stack serialized as BOC.
    pub stack_boc: Arc<Vec<u8>>,
}

impl EmulatorGetMethodRequest {
    /// Uses provider-resolved contract state.
    pub fn new_with_address(
        address: TonAddress,
        tx_id: Option<TxLTHash>,
        method_id: i32,
        stack_boc: Arc<Vec<u8>>,
    ) -> Self {
        Self {
            contract_state: EmulatorContractState::Address { address, tx_id },
            method_id,
            stack_boc,
        }
    }

    /// Uses caller-supplied contract state.
    pub fn new_with_state(state: Arc<ContractState>, method_id: i32, stack_boc: Arc<Vec<u8>>) -> Self {
        Self {
            contract_state: EmulatorContractState::Custom(state),
            method_id,
            stack_boc,
        }
    }
}

/// Successful get-method execution.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct EmulatorGetMethodSuccess {
    /// TVM exit code. Codes `0` and `1` indicate success.
    pub vm_exit_code: i32,
    /// Result stack serialized as BOC.
    pub stack_boc: Vec<u8>,
    /// Emulator log, when available.
    pub vm_log: Option<String>,
    /// Gas units consumed by execution.
    pub gas_used: Option<i32>,
    /// Original provider response, when retained for diagnostics.
    pub raw_response: Option<String>,
}

impl EmulatorGetMethodSuccess {
    /// Creates a result without provider-specific diagnostics.
    pub fn new(vm_exit_code: i32, stack_boc: Vec<u8>) -> Self {
        Self {
            vm_exit_code,
            stack_boc,
            vm_log: None,
            gas_used: None,
            raw_response: None,
        }
    }

    /// Attaches provider diagnostics.
    pub fn with_diagnostic(mut self, vm_log: Option<String>, gas_used: i32, raw_response: String) -> Self {
        self.vm_log = vm_log;
        self.gas_used = Some(gas_used);
        self.raw_response = Some(raw_response);
        self
    }
}

/// Executes complete TVM get-method requests and may resolve address-based state.
#[async_trait]
pub trait EmulationProvider: Send + Sync + 'static {
    /// Applies `timeout` to the complete provider operation.
    async fn emulate_get_method(
        &self,
        request: EmulatorGetMethodRequest,
        timeout: Option<Duration>,
    ) -> TonCoreResult<EmulatorGetMethodSuccess>;

    /// Returns whether address-based requests must be resolved through the client's
    /// [`StateProvider`](crate::traits::state_provider::StateProvider) before emulation.
    ///
    /// Defaults to `false` for providers that resolve [`EmulatorContractState::Address`]
    /// themselves. Direct callers must supply [`EmulatorContractState::Custom`] when this
    /// returns `true`; the `ton` crate's `ContractClient` performs that conversion automatically.
    fn requires_resolved_state(&self) -> bool {
        false
    }
}
