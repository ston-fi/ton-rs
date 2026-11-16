//! Typed wrappers for small TVM programs executed through [`ContractClient`].

use std::sync::Arc;

use crate::block_tlb::{FromTVMStack, TVMStack};
use crate::contracts::ContractClient;
use crate::errors::{TonError, TonResult};
use ton_core::cell::{TonCell, TonHash};
use ton_core::traits::emulation_provider::EmulatorGetMethodRequest;
use ton_core::traits::state_provider::ContractState;
use ton_core::traits::tlb::TLB;
use ton_core::types::{Coins, TonAddress, TxLTHash};

/// Returns total forwarding cost before the action-fee split.
/// Counts exclude the message root and its bits; configuration freshness is provider-owned.
pub async fn calculate_forward_fee(
    client: &ContractClient,
    cells: u64,
    bits: u64,
    is_masterchain: bool,
) -> TonResult<Coins> {
    let mut input = TVMStack::default();
    for (name, value) in [("cells", cells), ("bits", bits)] {
        let value = i64::try_from(value).map_err(|_| TonError::UnexpectedValue {
            expected: format!("{name} in 0..=i64::MAX"),
            actual: value.to_string(),
        })?;
        input.push_tiny_int(value);
    }
    input.push_tiny_int(if is_masterchain { -1 } else { 0 });

    // DROP the method selector, then GETFORWARDFEE.
    let mut code = TonCell::builder();
    code.write_num(&0x30f838_u32, 24)?;
    let state = ContractState::new(TonAddress::ZERO, TxLTHash::new(0, TonHash::ZERO), 0)
        .with_code_boc(Arc::new(code.build()?.to_boc()?))
        .with_data_boc(Arc::new(TonCell::empty().to_boc()?));
    let request = EmulatorGetMethodRequest::new_with_state(Arc::new(state), 0, Arc::new(input.to_boc()?));
    let response = client.emulate_get_method(request).await?;
    let mut output = TVMStack::from_boc(response.stack_boc)?;
    let fee = Coins::from_stack(&mut output)?;
    output.ensure_empty()?;
    Ok(fee)
}

#[cfg(test)]
#[path = "tests/test_tvm_methods.rs"]
mod tests;
