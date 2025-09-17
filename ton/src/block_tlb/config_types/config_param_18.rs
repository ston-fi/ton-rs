use crate::tlb_adapters::DictKeyAdapterInto;
use crate::tlb_adapters::DictValAdapterTLB;
use crate::tlb_adapters::TLBHashMap;
use std::collections::HashMap;
use ton_lib_core::errors::TonCoreError;
use ton_lib_core::{bail_ton_core_data, TLB};

// https://github.com/ton-blockchain/ton/blame/6f745c04daf8861bb1791cffce6edb1beec62204/crypto/block/block.tlb#L698
#[derive(Debug, Clone, PartialEq, TLB)]
pub struct ConfigParam18 {
    #[tlb(adapter = "TLBHashMap::<DictKeyAdapterInto, DictValAdapterTLB, _, _>::new(32)")]
    pub storage_prices: HashMap<u32, StoragePrices>,
}

impl ConfigParam18 {
    pub fn get_first(&self) -> Result<&StoragePrices, TonCoreError> {
        match self.storage_prices.values().next() {
            Some(v) => Ok(v),
            None => bail_ton_core_data!("No values in storage_prices"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, TLB)]
#[tlb(prefix = 0xcc, bits_len = 8)]
pub struct StoragePrices {
    pub utime_since: u32,
    pub bit_price_ps: u64,
    pub cell_price_ps: u64,
    pub mc_bit_price_ps: u64,
    pub mc_cell_price_ps: u64,
}
