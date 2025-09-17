use ton_lib_core::cell::TonCell;
use ton_lib_core::types::tlb_core::TLBEitherRef;
use ton_lib_core::types::TonAddress;
use ton_lib_core::TLB;

/// ```raw
/// ownership_assigned#0x05138d91
///   query_id:uint64
///   prev_owner:MsgAddress
///   forward_payload:(Either Cell ^Cell)
/// = InternalMsgBody;
/// ```
#[derive(Clone, Debug, PartialEq, TLB)]
#[tlb(prefix = 0x05138d91, bits_len = 32, ensure_empty = true)]
pub struct NFTOwnershipAssignedMsg {
    pub query_id: u64,
    pub prev_owner: TonAddress,
    pub forward_payload: TLBEitherRef<TonCell>,
}

impl NFTOwnershipAssignedMsg {
    pub fn new(prev_owner: &TonAddress) -> Self {
        Self {
            query_id: 0,
            prev_owner: prev_owner.clone(),
            forward_payload: TLBEitherRef::new(TonCell::EMPTY),
        }
    }
}
