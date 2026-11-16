use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use async_trait::async_trait;
use ton_core::cell::TonHash;
use ton_core::errors::TonCoreResult;
use ton_core::traits::emulation_provider::{
    EmulationProvider, EmulatorContractState, EmulatorGetMethodRequest, EmulatorGetMethodSuccess,
};
use ton_core::traits::state_provider::{ContractState, StateProvider};
use ton_core::types::{TonAddress, TxLTHash};

use crate::contracts::ContractClient;
use crate::errors::TonError;

const TIMEOUT: Duration = Duration::from_secs(7);

struct RecordingStateProvider {
    state: Arc<ContractState>,
    calls: Arc<AtomicUsize>,
}

#[async_trait]
impl StateProvider for RecordingStateProvider {
    async fn last_mc_seqno(&self) -> TonCoreResult<u32> {
        panic!("state cache refresh is disabled")
    }

    async fn load_state(&self, address: TonAddress, tx_id: Option<TxLTHash>) -> TonCoreResult<ContractState> {
        assert_eq!(address, self.state.address);
        assert_eq!(tx_id, Some(self.state.last_tx_id.clone()));
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok((*self.state).clone())
    }

    async fn load_latest_tx_per_address(&self, _: u32) -> TonCoreResult<Vec<(TonAddress, TxLTHash)>> {
        panic!("state cache refresh is disabled")
    }
}

struct RecordingEmulationProvider {
    resolve: bool,
    supplied: bool,
    state: Arc<ContractState>,
    exit_code: i32,
}

#[async_trait]
impl EmulationProvider for RecordingEmulationProvider {
    fn requires_resolved_state(&self) -> bool {
        self.resolve
    }

    async fn emulate_get_method(
        &self,
        request: EmulatorGetMethodRequest,
        timeout: Option<Duration>,
    ) -> TonCoreResult<EmulatorGetMethodSuccess> {
        assert_eq!(timeout, Some(TIMEOUT));
        assert_eq!(request.method_id, 42);
        assert_eq!(*request.stack_boc, vec![1, 2, 3]);
        match request.contract_state {
            EmulatorContractState::Custom(state) => {
                assert!(self.resolve || self.supplied);
                assert_eq!(*state, *self.state);
                if self.supplied {
                    assert!(Arc::ptr_eq(&state, &self.state));
                }
            },
            EmulatorContractState::Address { address, tx_id } => {
                assert!(!self.resolve && !self.supplied);
                assert_eq!(address, self.state.address);
                assert_eq!(tx_id, Some(self.state.last_tx_id.clone()));
            },
            _ => panic!("unexpected state variant"),
        }
        Ok(EmulatorGetMethodSuccess::new(self.exit_code, vec![0xff]).with_diagnostic(
            Some("vm log".into()),
            12,
            "provider diagnostics".into(),
        ))
    }
}

#[tokio::test]
async fn test_raw_execution_resolves_only_address_requests_when_required() -> anyhow::Result<()> {
    for resolve in [false, true] {
        for supplied in [false, true] {
            for exit_code in [0, 1, 6] {
                let state = Arc::new(ContractState::new(TonAddress::ZERO, TxLTHash::new(123, TonHash::ZERO), 456));
                let calls = Arc::new(AtomicUsize::new(0));
                let client = ContractClient::builder(
                    RecordingStateProvider {
                        state: state.clone(),
                        calls: calls.clone(),
                    },
                    RecordingEmulationProvider {
                        resolve,
                        supplied,
                        state: state.clone(),
                        exit_code,
                    },
                )?
                .with_tvm_emulation_timeout(TIMEOUT)
                .build()?;
                let stack = Arc::new(vec![1, 2, 3]);
                let request = if supplied {
                    EmulatorGetMethodRequest::new_with_state(state, 42, stack)
                } else {
                    EmulatorGetMethodRequest::new_with_address(state.address, Some(state.last_tx_id.clone()), 42, stack)
                };
                let response = client.emulate_get_method(request).await;
                assert_eq!(calls.load(Ordering::SeqCst), usize::from(resolve && !supplied));
                if exit_code == 6 {
                    assert!(matches!(response, Err(TonError::EmulatorEmulationError {
                        vm_exit_code: Some(6), response_raw,
                    }) if response_raw == "provider diagnostics"));
                } else {
                    let response = response?;
                    assert_eq!(response.vm_exit_code, exit_code);
                    assert_eq!(response.stack_boc, vec![0xff]);
                    assert_eq!(response.vm_log.as_deref(), Some("vm log"));
                    assert_eq!(response.gas_used, Some(12));
                    assert_eq!(response.raw_response.as_deref(), Some("provider diagnostics"));
                }
            }
        }
    }
    Ok(())
}
