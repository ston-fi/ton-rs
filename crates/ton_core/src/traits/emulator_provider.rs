use crate::errors::TonCoreResult;
use crate::traits::state_provider::ContractState;
use crate::types::{TonAddress, TxLTHash};
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;

/// Contract state used for get-method emulation.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum EmulatorContractState {
    /// Resolve state by address and optional exact transaction reference.
    ///
    /// When `tx_id` is `None`, the provider must use the latest state.
    Address {
        address: TonAddress,
        tx_id: Option<TxLTHash>,
    },
    /// Emulate against state supplied by the caller.
    Custom(ContractState),
}

/// Inputs required to execute a TVM get method.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct EmulatorGetMethodRequest {
    pub contract_state: EmulatorContractState,
    /// Numeric TVM method identifier.
    pub method_id: i32,
    /// Input stack serialized as BOC.
    pub stack_boc: Arc<Vec<u8>>,
}

impl EmulatorGetMethodRequest {
    /// Creates a request that resolves state by address and optional transaction.
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

    /// Creates a request using caller-supplied contract state.
    pub fn new_with_state(state: ContractState, method_id: i32, stack_boc: Arc<Vec<u8>>) -> Self {
        Self {
            contract_state: EmulatorContractState::Custom(state),
            method_id,
            stack_boc,
        }
    }
}

/// Successful get-method execution returned by an [`EmulatorProvider`].
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct EmulatorGetMethodSuccess {
    /// TVM exit code. Codes `0` and `1` indicate success.
    pub vm_exit_code: i32,
    /// Result stack serialized as BOC.
    pub stack_boc: Vec<u8>,
    /// Emulator log, when provided by the implementation.
    pub vm_log: Option<String>,
    /// Gas units consumed by execution, when provided by the implementation.
    pub gas_used: Option<i32>,
    /// Unprocessed provider response retained for diagnostics, when available.
    pub raw_response: Option<String>,
}

impl EmulatorGetMethodSuccess {
    /// Creates a provider-neutral successful response.
    pub fn new(vm_exit_code: i32, stack_boc: Vec<u8>) -> Self {
        Self {
            vm_exit_code,
            stack_boc,
            vm_log: None,
            gas_used: None,
            raw_response: None,
        }
    }

    /// Creates a response with native emulator diagnostics.
    pub fn with_diagnostics(
        vm_exit_code: i32,
        stack_boc: Vec<u8>,
        vm_log: Option<String>,
        gas_used: i32,
        raw_response: String,
    ) -> Self {
        Self {
            vm_exit_code,
            stack_boc,
            vm_log,
            gas_used: Some(gas_used),
            raw_response: Some(raw_response),
        }
    }
}

/// Executes complete TVM get-method requests.
///
/// Implementations own state resolution, blockchain configuration, library
/// loading, missing-library retries, and native or remote emulator execution.
#[async_trait]
pub trait EmulatorProvider: Send + Sync + 'static {
    /// Executes the complete request within the requested timeout, including
    /// state resolution, configuration and library loading, and emulation.
    async fn emulate_get_method(
        &self,
        request: EmulatorGetMethodRequest,
        timeout: Option<Duration>,
    ) -> TonCoreResult<EmulatorGetMethodSuccess>;
}
