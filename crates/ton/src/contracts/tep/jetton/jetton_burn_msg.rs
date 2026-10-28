//! Jetton burn request message.

use ton_core::TLB;
use ton_core::cell::TonCell;
use ton_core::traits::tlb::TLB;
use ton_core::types::tlb_core::TLBCoins;
use ton_core::types::tlb_core::{MsgAddress, TLBRef};

/// ```raw
/// burn#595f07bc query_id:uint64 amount:(VarUInteger 16)
/// response_destination:MsgAddress
/// custom_payload:(Maybe ^Cell)
/// = InternalMsgBody;
/// ```
#[derive(Clone, Debug, PartialEq, TLB)]
#[tlb(prefix = 0x595f07bc, bits_len = 32, ensure_empty = true)]
pub struct JettonBurnMsg<CustomPayloadT: TLB = TonCell> {
    pub query_id: u64,            // arbitrary request number
    pub amount: TLBCoins,         // amount to burn
    pub response_dst: MsgAddress, // address to send confirmation
    pub custom_payload: Option<TLBRef<CustomPayloadT>>,
}

impl JettonBurnMsg {
    /// Creates a burn request with query ID zero, no response address, and no custom payload.
    pub fn new<T: Into<TLBCoins>>(amount: T) -> Self {
        JettonBurnMsg {
            query_id: 0,
            amount: amount.into(),
            response_dst: MsgAddress::NONE,
            custom_payload: None,
        }
    }
}

#[cfg(test)]
#[path = "../../../tests/contracts/tep/jetton/test_jetton_burn_msg.rs"]
mod tests;
