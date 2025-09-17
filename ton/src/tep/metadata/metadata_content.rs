use crate::tep::snake_data::SnakeData;
use crate::tlb_adapters::DictKeyAdapterTonHash;
use crate::tlb_adapters::DictValAdapterTLBRef;
use crate::tlb_adapters::TLBHashMapE;
use std::collections::HashMap;
use std::fmt::Debug;
use ton_lib_core::cell::{TonCell, TonHash};
use ton_lib_core::TLB;

#[derive(PartialEq, Eq, Debug, Clone, TLB)]
pub enum MetadataContent {
    Internal(MetadataInternal),
    External(MetadataExternal),
    Unsupported(MetadataUnsupported),
}

#[derive(PartialEq, Eq, Debug, Clone, TLB)]
#[tlb(prefix = 0x0, bits_len = 8)]
pub struct MetadataInternal {
    #[tlb(adapter = "TLBHashMapE::<DictKeyAdapterTonHash, DictValAdapterTLBRef, _, _>::new(256)")]
    pub data: HashMap<TonHash, SnakeData>,
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
