use ton_lib_core::TLB;

#[derive(Debug, Clone, PartialEq, TLB)]
#[tlb(prefix = 0xc4, bits_len = 8)]
pub struct GlobalVersion {
    pub version: u32,
    pub capabilities: u64,
}
