use crate::block_tlb::{BlockExtra, BlockInfo, MerkleUpdate, ValueFlow};
use ton_core::TLB;
use ton_core::cell::TonCell;
use ton_core::types::tlb_core::TLBRef;

// https://github.com/ton-blockchain/ton/blob/6f745c04daf8861bb1791cffce6edb1beec62204/crypto/block/block.tlb#L462
#[derive(Debug, Clone, PartialEq, TLB)]
#[tlb(prefix = 0x11ef55aa, bits_len = 32)]
pub struct Block {
    pub global_id: i32,
    pub info: TLBRef<BlockInfo>,
    pub value_flow: TLBRef<ValueFlow>,
    pub state_update: TLBRef<MerkleUpdate<TonCell>>,
    pub extra: TLBRef<BlockExtra>,
}

#[cfg(test)]
#[path = "../../tests/block_tlb/block_types/test_block.rs"]
mod tests;
