use crate::block_tlb::ShardPfx;
use crate::tlb_adapters::DictValAdapter;
use std::collections::HashMap;
use std::marker::PhantomData;
use ton_core::bail_ton_core_data;
use ton_core::cell::{CellBuilder, CellParser, TonCell};
use ton_core::constants::TON_MAX_SPLIT_DEPTH;
use ton_core::errors::TonCoreError;

// for now it's used only only with ShardPfx in keys
pub struct BinTree<VA: DictValAdapter>(PhantomData<VA>);

impl<VA: DictValAdapter> Default for BinTree<VA> {
    fn default() -> Self {
        Self::new()
    }
}

impl<VA: DictValAdapter> BinTree<VA> {
    /// Creates a stateless binary-tree adapter.
    pub fn new() -> Self {
        Self(PhantomData)
    }

    pub fn read(parser: &mut CellParser) -> Result<HashMap<ShardPfx, VA::ValType>, TonCoreError> {
        let mut val = HashMap::new();
        Self::read_impl(parser, ShardPfx::default(), &mut val)?;
        Ok(val)
    }

    fn read_impl(
        parser: &mut CellParser,
        cur_key: ShardPfx,
        cur_val: &mut HashMap<ShardPfx, VA::ValType>,
    ) -> Result<(), TonCoreError> {
        if cur_key.bits_len > TON_MAX_SPLIT_DEPTH as u32 {
            bail_ton_core_data!("[read] BinTree depth exceeded: {} > {TON_MAX_SPLIT_DEPTH}", cur_key.bits_len);
        }
        if !parser.read_bit()? {
            cur_val.insert(cur_key, VA::read(parser)?);
            return Ok(());
        }
        let new_bits_len = cur_key.bits_len + 1;

        let left_key = ShardPfx {
            value: cur_key.value,
            bits_len: new_bits_len,
        };
        Self::read_impl(&mut parser.read_next_ref()?.parser(), left_key, cur_val)?;

        let right_key = ShardPfx {
            value: cur_key.value | (1 << (64 - new_bits_len)),
            bits_len: new_bits_len,
        };
        Self::read_impl(&mut parser.read_next_ref()?.parser(), right_key, cur_val)?;
        Ok(())
    }

    pub fn write(builder: &mut CellBuilder, data: &HashMap<ShardPfx, VA::ValType>) -> Result<(), TonCoreError> {
        if data.is_empty() {
            bail_ton_core_data!("BinTree can't be empty");
        }
        Self::write_impl(builder, ShardPfx::default(), data)
    }

    fn write_impl(
        builder: &mut CellBuilder,
        cur_key: ShardPfx,
        data: &HashMap<ShardPfx, VA::ValType>,
    ) -> Result<(), TonCoreError> {
        if cur_key.bits_len > TON_MAX_SPLIT_DEPTH as u32 {
            bail_ton_core_data!("[write] BinTree depth exceeded: {} > {TON_MAX_SPLIT_DEPTH}", cur_key.bits_len);
        }
        if let Some(val) = data.get(&cur_key) {
            builder.write_bit(false)?;
            return VA::write(builder, val);
        }
        builder.write_bit(true)?;

        let new_bits_len = cur_key.bits_len + 1;
        let left_key = ShardPfx {
            value: cur_key.value,
            bits_len: new_bits_len,
        };
        let right_key = ShardPfx {
            value: cur_key.value | (1 << (64 - new_bits_len)),
            bits_len: new_bits_len,
        };

        let mut left_builder = TonCell::builder();
        Self::write_impl(&mut left_builder, left_key, data)?;
        builder.write_ref(left_builder.build()?)?;

        let mut right_builder = TonCell::builder();
        Self::write_impl(&mut right_builder, right_key, data)?;
        builder.write_ref(right_builder.build()?)?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "../tests/tlb_adapters/test_bin_tree.rs"]
mod tests;
