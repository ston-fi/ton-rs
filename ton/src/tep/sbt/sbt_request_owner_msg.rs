use ton_core::cell::TonCell;
use ton_core::types::tlb_core::{MsgAddress, TLBRef};
use ton_core::TLB;

/// ```raw
/// request_owner#d0c3bfea
///   query_id:uint64
///   dest:MsgAddress
///   forward_payload:^Cell
///   with_content:Bool
/// = InternalMsgBody;
/// ```
#[derive(Clone, Debug, PartialEq, TLB)]
#[tlb(prefix = 0xd0c3bfea, bits_len = 32, ensure_empty = true)]
pub struct SbtRequestOwnerMsg {
    pub query_id: u64,
    pub dst: MsgAddress, // address of the contract to which the ownership of SBT should be proven
    pub forward_payload: TLBRef<TonCell>, // arbitrary data required by target contract
    pub with_content: bool, // if true, SBT's content cell will be included in message to contract.
}
