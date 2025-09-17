use ton_lib_core::TLB;

#[derive(Clone, Debug, PartialEq, TLB)]
#[tlb(prefix = 0xd53276db, bits_len = 32, ensure_empty = true)]
pub struct NFTExcessesMsg {
    pub query_id: u64,
}
