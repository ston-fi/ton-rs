use crate::errors::TonError;
use crate::tlb_adapters::dict::data_builder::DictDataBuilder;
use crate::tlb_adapters::dict::data_parser::DictDataParser;
use crate::tlb_adapters::{DictKeyAdapter, DictValAdapter};
use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;
use ton_lib_core::bail_ton_core_data;
use ton_lib_core::cell::CellBuilder;
use ton_lib_core::cell::CellParser;
use ton_lib_core::errors::TonCoreError;

// https://github.com/ton-blockchain/ton/blame/72056a2261cbb11f7cf0f20b389bcbffe018b1a8/crypto/block/block.tlb#L22
/// Adapter to write HashMap with arbitrary key/values into a cell
/// Doesn't write 'present' marker to root cell. Generally, is not supposed to be used in TLB structs
/// Usage example: `#[tlb_derive(adapter = "TLBHashMap::<DictKeyAdapterTonHash, DictValAdapterTLB, _, _>::new(256)")]` instead
pub struct TLBHashMap<KA: DictKeyAdapter<K>, VA: DictValAdapter<V>, K, V> {
    key_bits_len: u32,
    _phantom: PhantomData<(KA, VA, K, V)>,
}

impl<KA, VA, K, V> TLBHashMap<KA, VA, K, V>
where
    KA: DictKeyAdapter<K>,
    VA: DictValAdapter<V>,
    K: Eq + Hash,
{
    pub fn new(key_bits_len: u32) -> Self {
        Self {
            key_bits_len,
            _phantom: PhantomData,
        }
    }

    pub fn read(&self, parser: &mut CellParser) -> Result<HashMap<K, V>, TonCoreError> {
        let mut data_parser = DictDataParser::new(self.key_bits_len as usize);
        let data_raw = data_parser.read::<V, VA>(parser)?;
        let data = data_raw
            .into_iter()
            .map(|(k, v)| Ok::<_, TonCoreError>((KA::extract_key(&k)?, v)))
            .collect::<Result<HashMap<K, V>, _>>()?;
        Ok(data)
    }

    pub fn write(&self, builder: &mut CellBuilder, data: &HashMap<K, V>) -> Result<(), TonCoreError> {
        if data.is_empty() {
            bail_ton_core_data!("empty HashMap can't be written");
        }
        let mut key_value_pairs =
            data.iter().map(|(k, v)| Ok::<_, TonError>((KA::make_key(k)?, v))).collect::<Result<Vec<_>, _>>()?;
        key_value_pairs.sort_by_key(|(x, _)| x.clone());

        let mut keys_sorted = Vec::with_capacity(key_value_pairs.len());
        let mut values_sorted = Vec::with_capacity(key_value_pairs.len());
        for (key, value) in key_value_pairs {
            keys_sorted.push(key);
            values_sorted.push(value);
        }
        let data_builder = DictDataBuilder::<V, VA>::new(self.key_bits_len as usize, keys_sorted, &values_sorted)?;
        let dict_data_cell = data_builder.build()?;
        builder.write_cell(&dict_data_cell)
    }
}
