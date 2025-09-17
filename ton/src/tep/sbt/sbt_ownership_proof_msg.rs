use crate::tlb_adapters::ConstLen;
use num_bigint::BigUint;
use ton_lib_core::cell::TonCellRef;
use ton_lib_core::types::tlb_core::MsgAddress;
use ton_lib_core::TLB;

/// ```raw
/// ownership_proof#0524c7ae
///   query_id:uint64
///   item_id:uint256
///   owner:MsgAddress
///   data:^Cell
///   revoked_at:uint64
///   content:(Maybe ^Cell)
/// = InternalMsgBody;
/// ```
#[derive(Clone, Debug, PartialEq, TLB)]
#[tlb(prefix = 0x0524c7ae, bits_len = 32, ensure_empty = true)]
pub struct SbtOwnershipProofMsg {
    pub query_id: u64,
    #[tlb(bits_len = 256)]
    pub item_id: BigUint,
    pub owner: MsgAddress,
    pub data: TonCellRef,
    pub revoked_at: u64,
    pub content: Option<TonCellRef>,
}
