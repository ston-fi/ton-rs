use crate::block_tlb::BlockIdExt;
use ton_lib_core::cell::TonHash;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BlockId {
    pub wc: i32,
    pub shard: u64,
    pub seqno: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ZeroStateIdExt {
    pub workchain: i32,
    pub root_hash: TonHash,
    pub file_hash: TonHash,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MasterchainInfo {
    pub last: BlockIdExt,
    pub state_root_hash: TonHash,
    pub init: ZeroStateIdExt,
}
