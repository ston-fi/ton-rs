use crate::tlb_adapters::ConstLen;
use num_bigint::BigUint;
use ton_lib_core::cell::TonCellRef;
use ton_lib_core::types::tlb_core::MsgAddress;
use ton_lib_core::TLB;

/// ```raw
/// owner_info#0dd607e3
///   query_id:uint64
///   item_id:uint256
///   initiator:MsgAddress
///   owner:MsgAddress
///   data:^Cell
///   revoked_at:uint64
///   content:(Maybe ^Cell)
/// = InternalMsgBody;
/// ```
#[derive(Clone, Debug, PartialEq, TLB)]
#[tlb(prefix = 0x0dd607e3, bits_len = 32, ensure_empty = true)]
pub struct SbtOwnerInfoMsg {
    pub query_id: u64,
    #[tlb(bits_len = 256)]
    pub item_id: BigUint,
    pub initiator: MsgAddress, // address of request initiator
    pub owner: MsgAddress,
    pub data: TonCellRef,            // data cell passed in prove_ownership.
    pub revoked_at: u64,             // unixtime
    pub content: Option<TonCellRef>, // NFT's content, it is passed if with_content was true in prove_ownership.
}
