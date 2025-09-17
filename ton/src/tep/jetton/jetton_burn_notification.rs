use crate::block_tlb::Coins;
use ton_lib_core::types::tlb_core::MsgAddress;
use ton_lib_core::TLB;

/// ```raw
/// burn_notification#7bdd97de query_id:uint64 amount:(VarUInteger 16)
/// sender:MsgAddress
/// response_destination:MsgAddress
/// = InternalMsgBody;
/// ```
// TODO is not tested
#[derive(Debug, Clone, PartialEq, TLB)]
#[tlb(prefix = 0x7bdd97de, bits_len = 32, ensure_empty = true)]
pub struct JettonBurnNotification {
    pub query_id: u64,
    pub amount: Coins,
    pub sender: MsgAddress,
    pub response_dst: MsgAddress,
}
