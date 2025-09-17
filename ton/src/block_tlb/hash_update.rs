use ton_lib_core::cell::TonHash;
use ton_lib_core::TLB;

// https://github.com/ton-blockchain/ton/blob/ed4682066978f69ffa38dd98912ca77d4f660f66/crypto/block/block.tlb#L302
#[derive(Default, Debug, PartialEq, Clone, TLB)]
#[tlb(prefix = 0x72, bits_len = 8)]
pub struct HashUpdate {
    pub old: TonHash,
    pub new: TonHash,
}
