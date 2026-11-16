use ton_core::TLB;

#[derive(Debug, Clone, PartialEq, Eq, TLB)]
#[tlb(prefix = 0xc4, bits_len = 8)]
pub struct GlobalVersion {
    pub version: u32,
    pub capabilities: u64,
}
