use crate::block_tlb::Coins;
use crate::tlb_adapters::ConstLen;
use crate::tlb_adapters::DictKeyAdapterAddress;
use crate::tlb_adapters::DictValAdapterTLB;
use crate::tlb_adapters::TLBHashMap;
use std::collections::HashMap;
use ton_lib_core::types::TonAddress;
use ton_lib_core::TLB;
// TODO not tested
#[derive(Clone, Debug, TLB)]
pub struct MintlessAirdropDict {
    #[tlb(adapter = "TLBHashMap::<DictKeyAdapterAddress, DictValAdapterTLB, _, _>::new(267)")]
    pub data: HashMap<TonAddress, MintlessAirdropData>,
}

#[derive(Clone, Debug, PartialEq, TLB)]
pub struct MintlessAirdropData {
    pub amount: Coins,
    #[tlb(bits_len = 48)]
    pub start_from: u64,
    #[tlb(bits_len = 48)]
    pub expired_at: u64,
}
