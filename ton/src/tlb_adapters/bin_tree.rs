use crate::block_tlb::ShardPfx;
use crate::tlb_adapters::DictValAdapter;
use std::collections::HashMap;
use std::marker::PhantomData;
use ton_lib_core::bail_ton_core_data;
use ton_lib_core::cell::{CellBuilder, CellParser, TonCell};
use ton_lib_core::constants::TON_MAX_SPLIT_DEPTH;
use ton_lib_core::errors::TonCoreError;
use ton_lib_core::traits::tlb::TLB;

// for now it's used only only with ShardPfx in keys
pub struct BinTree<VA: DictValAdapter<T>, T: TLB>(PhantomData<(VA, T)>);

impl<VA: DictValAdapter<T>, T: TLB> Default for BinTree<VA, T> {
    fn default() -> Self { Self::new() }
}

impl<VA: DictValAdapter<T>, T: TLB> BinTree<VA, T> {
    pub fn new() -> Self { Self(PhantomData) }

    pub fn read(parser: &mut CellParser) -> Result<HashMap<ShardPfx, T>, TonCoreError> {
        let mut val = HashMap::new();
        Self::read_impl(parser, ShardPfx::default(), &mut val)?;
        Ok(val)
    }

    fn read_impl(
        parser: &mut CellParser,
        cur_key: ShardPfx,
        cur_val: &mut HashMap<ShardPfx, T>,
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

    pub fn write(builder: &mut CellBuilder, data: &HashMap<ShardPfx, T>) -> Result<(), TonCoreError> {
        if data.is_empty() {
            bail_ton_core_data!("BinTree can't be empty");
        }
        Self::write_impl(builder, ShardPfx::default(), data)
    }

    fn write_impl(
        builder: &mut CellBuilder,
        cur_key: ShardPfx,
        data: &HashMap<ShardPfx, T>,
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
        builder.write_ref(left_builder.build_ref()?)?;

        let mut right_builder = TonCell::builder();
        Self::write_impl(&mut right_builder, right_key, data)?;
        builder.write_ref(right_builder.build_ref()?)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tlb_adapters::DictValAdapterNum;

    #[test]
    fn test_bin_tree() -> anyhow::Result<()> {
        //                                                * (0000,len=0)
        //                 * (0100,len=1)                                        * (1100, len=1)
        //        *(0010,len=2)          * [0100,len=2]=3                   * (1010,len=2)          * [1100,len=2] = 6
        // * [0000,len=3]=1   * [0010,len=3]=2                 * [1000,len=3]=4     * [1010,len=3]=5
        let data = HashMap::from([
            (
                ShardPfx {
                    value: 0x0,
                    bits_len: 3,
                },
                1,
            ),
            (
                ShardPfx {
                    value: 0x2_000000000000000,
                    bits_len: 3,
                },
                2,
            ),
            (
                ShardPfx {
                    value: 0x4_000000000000000,
                    bits_len: 2,
                },
                3,
            ),
            (
                ShardPfx {
                    value: 0x8_000000000000000,
                    bits_len: 3,
                },
                4,
            ),
            (
                ShardPfx {
                    value: 0xA_000000000000000,
                    bits_len: 3,
                },
                5,
            ),
            (
                ShardPfx {
                    value: 0xC_000000000000000,
                    bits_len: 2,
                },
                6,
            ),
        ]);
        let mut builder = TonCell::builder();
        BinTree::<DictValAdapterNum<32>, u32>::write(&mut builder, &data)?;
        let cell = builder.build()?;
        let mut parser = cell.parser();
        let parsed_data = BinTree::<DictValAdapterNum<32>, u32>::read(&mut parser)?;
        assert_eq!(data, parsed_data);
        Ok(())
    }
}
