use crate::block_tlb::block_types::mc_block_extra::MCBlockExtra;
use crate::tlb_adapters::TLBRefOpt;
use ton_lib_core::cell::{TonCellRef, TonHash};
use ton_lib_core::TLB;

// https://github.com/ton-blockchain/ton/blame/6f745c04daf8861bb1791cffce6edb1beec62204/crypto/block/block.tlb#L467
#[derive(Debug, Clone, PartialEq, TLB)]
#[tlb(prefix = 0x4a33f6fd, bits_len = 32)]
pub struct BlockExtra {
    pub in_msg_descr: TonCellRef,   // TODO
    pub out_msg_descr: TonCellRef,  // TODO
    pub account_blocks: TonCellRef, // TODO
    pub rand_seed: TonHash,
    pub created_by: TonHash,
    #[tlb(adapter = "TLBRefOpt")]
    pub mc_block_extra: Option<MCBlockExtra>,
}
