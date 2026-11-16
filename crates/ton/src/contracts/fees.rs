//! Network forwarding prices through the configured get-method provider.

use std::sync::Arc;

use crate::block_tlb::{FromTVMStack, TVMStack};
use crate::contracts::ContractClient;
use crate::errors::{TonError, TonResult};
use ton_core::cell::{TonCell, TonHash};
use ton_core::traits::emulation_provider::EmulatorGetMethodRequest;
use ton_core::traits::state_provider::ContractState;
use ton_core::traits::tlb::TLB;
use ton_core::types::{Coins, TonAddress, TxLTHash};

/// Calculates a message's total forward fee, before the action-fee split.
///
/// `cells` and `bits` are billable statistics excluding the message root cell
/// and its bits. `is_masterchain` selects masterchain rather than basechain prices.
/// This function does not count message cells or predict their wire layout.
///
/// Uses TVM `GETFORWARDFEE` through `client`'s existing emulation provider and
/// configured timeout, without account-state reads, deployment, or native features.
/// Configuration selection and freshness remain provider-owned; the quote does not
/// promise a fresh or historical block snapshot. No quote is cached here.
///
/// ```no_run
/// use ton::contracts::{ContractClient, fees::calculate_forward_fee};
/// use ton::errors::TonResult;
/// use ton::ton_core::types::Coins;
///
/// async fn notification_fee(client: &ContractClient) -> TonResult<Coins> {
///     // A conservative bound for a referenced notification body and payload.
///     calculate_forward_fee(client, 2, 787, false).await
/// }
/// ```
///
/// # Errors
///
/// Returns an error if either size exceeds `i64::MAX`, the TVM configuration
/// does not support the instruction (global version 6 or later), the provider
/// fails or times out, or the result is not exactly one nonnegative integer
/// fitting [`Coins`]. Provider and VM errors retain their existing typed details.
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

    // DROP the get-method selector; GETFORWARDFEE consumes cells/bits/is_mc.
    // https://docs.ton.org/tvm/instructions#f838-getforwardfee
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
#[path = "../tests/contracts/test_fees.rs"]
mod tests;
