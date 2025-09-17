use crate::block_tlb::block_types::block_info::ExtBlockRef;
use crate::tlb_adapters::TLBRef;
use ton_lib_core::TLB;

#[derive(Debug, Clone, PartialEq, TLB)]
pub enum PrevBlockInfo {
    Regular(ExtBlockRef),
    AfterMerge(BlockPrevInfoAfterMerge), // is not tested
}

#[derive(Debug, Clone, PartialEq, TLB)]
pub struct BlockPrevInfoAfterMerge {
    #[tlb(adapter = "TLBRef")]
    pub prev1: ExtBlockRef,
    #[tlb(adapter = "TLBRef")]
    pub prev2: ExtBlockRef,
}

impl Default for PrevBlockInfo {
    fn default() -> Self {
        PrevBlockInfo::Regular(ExtBlockRef {
            end_lt: 0,
            seqno: 0,
            root_hash: Default::default(),
            file_hash: Default::default(),
        })
    }
}
