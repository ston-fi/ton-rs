use crate::tlb_adapters::{DictKeyAdapter, DictValAdapter, TLBHashMap};
use std::collections::HashMap;
use std::hash::Hash;
use ton_core::cell::CellBuilder;
use ton_core::cell::CellParser;
use ton_core::cell::TonCell;
use ton_core::errors::TonCoreError;

// https://github.com/ton-blockchain/ton/blame/72056a2261cbb11f7cf0f20b389bcbffe018b1a8/crypto/block/block.tlb#L37
/// Write present marker (0|1 bit) to root cell, and then Dict data to first ref cell.
/// Usage: `#[tlb_derive(adapter = "TLBHashMapE::<DictKeyAdapterTonHash, DictValAdapterTLB, _, _>::new(256)")]` instead
pub struct TLBHashMapE<KA: DictKeyAdapter, VA: DictValAdapter>(TLBHashMap<KA, VA>);

impl<KA, VA> TLBHashMapE<KA, VA>
where
    KA: DictKeyAdapter,
    VA: DictValAdapter,
    KA::KeyType: Eq + Hash + Ord,
{
    /// Creates an optional dictionary adapter with the specified fixed key length.
    pub fn new(key_bits_len: u32) -> Self {
        Self(TLBHashMap::new(key_bits_len))
    }

    pub fn read(&self, parser: &mut CellParser) -> Result<HashMap<KA::KeyType, VA::ValType>, TonCoreError> {
        if !parser.read_bit()? {
            return Ok(HashMap::new());
        }
        self.0.read(&mut parser.read_next_ref()?.parser())
    }

    pub fn write(
        &self,
        builder: &mut CellBuilder,
        data: &HashMap<KA::KeyType, VA::ValType>,
    ) -> Result<(), TonCoreError> {
        if data.is_empty() {
            builder.write_bit(false)?;
            return Ok(());
        }
        builder.write_bit(true)?;
        let mut dict_data_builder = TonCell::builder();
        self.0.write(&mut dict_data_builder, data)?;
        builder.write_ref(dict_data_builder.build()?)
    }
}

#[cfg(test)]
#[path = "../../tests/tlb_adapters/dict/test_tlb_hash_map_e.rs"]
mod tests;
