use crate::block_tlb::block_types::mc_block_extra::MCBlockExtra;
use ton_core::cell::{TonCell, TonHash};
use ton_core::types::tlb_core::TLBRef;
use ton_core::TLB;

// https://github.com/ton-blockchain/ton/blame/6f745c04daf8861bb1791cffce6edb1beec62204/crypto/block/block.tlb#L467
#[derive(Debug, Clone, PartialEq, TLB)]
#[tlb(prefix = 0x4a33f6fd, bits_len = 32)]
pub struct BlockExtra {
    pub in_msg_descr: TLBRef<TonCell>,   // TODO
    pub out_msg_descr: TLBRef<TonCell>,  // TODO
    pub account_blocks: TLBRef<TonCell>, // TODO
    pub rand_seed: TonHash,
    pub created_by: TonHash,
    pub mc_block_extra: Option<TLBRef<MCBlockExtra>>,
}
