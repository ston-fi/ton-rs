//! Jetton internal-transfer message.

use ton_core::TLB;
use ton_core::cell::TonCell;
use ton_core::traits::tlb::TLB;
use ton_core::types::tlb_core::TLBCoins;
use ton_core::types::tlb_core::{MsgAddress, TLBEitherRef};

/// ```raw
/// internal_transfer#178d4519  query_id:uint64 amount:(VarUInteger 16)
/// from:MsgAddress
/// response_address:MsgAddress
/// forward_ton_amount:(VarUInteger 16)
/// forward_payload:(Either Cell ^Cell)
/// = InternalMsgBody;
/// ```
#[derive(Debug, Clone, PartialEq, TLB)]
#[tlb(prefix = 0x178d4519, bits_len = 32, ensure_empty = true)]
pub struct JettonInternalTransferMsg<ForwardPayloadT: TLB = TonCell> {
    pub query_id: u64,
    pub amount: TLBCoins,
    pub from_address: MsgAddress,
    pub response_address: MsgAddress,
    pub forward_amount: TLBCoins,
    pub forward_payload: TLBEitherRef<ForwardPayloadT>,
}

#[cfg(test)]
#[path = "../../../tests/contracts/tep/jetton/test_jetton_internal_transfer_msg.rs"]
mod tests;
