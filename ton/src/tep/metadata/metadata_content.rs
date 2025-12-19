use crate::block_tlb::{TVMStack, TVMType};
use crate::errors::TonResult;
use crate::tep::snake_data::SnakeData;
use crate::tlb_adapters::DictKeyAdapterTonHash;
use crate::tlb_adapters::DictValAdapterTLB;
use crate::tlb_adapters::TLBHashMapE;
use std::collections::HashMap;
use std::fmt::Debug;
use ton_core::TLB;
use ton_core::cell::{TonCell, TonHash};
use ton_core::traits::tlb::TLB;
use ton_core::types::tlb_core::TLBRef;

pub type MetadataDict = HashMap<TonHash, TLBRef<SnakeData>>;

#[derive(PartialEq, Debug, Clone, TLB)]
pub enum MetadataContent {
    Internal(MetadataInternal),
    External(MetadataExternal),
    Unsupported(MetadataUnsupported),
}

#[derive(PartialEq, Debug, Clone, TLB)]
#[tlb(prefix = 0x0, bits_len = 8)]
pub struct MetadataInternal {
    #[tlb(adapter = "TLBHashMapE::<DictKeyAdapterTonHash, DictValAdapterTLB<_>>::new(256)")]
    pub data: MetadataDict,
}

#[derive(PartialEq, Eq, Debug, Clone, TLB)]
#[tlb(prefix = 0x1, bits_len = 8)]
pub struct MetadataExternal {
    pub uri: SnakeData,
}

#[derive(PartialEq, Eq, Debug, Clone, TLB)]
pub struct MetadataUnsupported {
    pub cell: TonCell,
}

impl TVMType for MetadataContent {
    fn from_stack(stack: &mut TVMStack) -> TonResult<Self> { Ok(MetadataContent::from_cell(&stack.pop_cell()?)?) }
}
