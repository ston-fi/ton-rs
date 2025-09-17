use std::collections::HashMap;

use super::label_type::DictLabelType;
use crate::tlb_adapters::DictValAdapter;
use num_bigint::BigUint;
use num_traits::One;
use ton_lib_core::cell::CellParser;
use ton_lib_core::errors::TonCoreError;
use ton_lib_core::traits::tlb::TLB;
use ton_lib_core::types::tlb_core::UnaryLen;

pub struct DictDataParser {
    key_bits_len: usize,
    cur_key_prefix: BigUint, // store leading 1 to determinate len properly
}

impl DictDataParser {
    pub fn new(key_len_bits: usize) -> DictDataParser {
        DictDataParser {
            key_bits_len: key_len_bits,
            cur_key_prefix: BigUint::one(),
        }
    }

    pub fn read<T, VA: DictValAdapter<T>>(
        &mut self,
        parser: &mut CellParser,
    ) -> Result<HashMap<BigUint, T>, TonCoreError> {
        // reset state in case of reusing
        self.cur_key_prefix = BigUint::one();

        let mut result = HashMap::new();
        self.parse_impl::<T, VA>(parser, &mut result)?;
        Ok(result)
    }

    fn parse_impl<T, VA: DictValAdapter<T>>(
        &mut self,
        parser: &mut CellParser,
        dst: &mut HashMap<BigUint, T>,
    ) -> Result<(), TonCoreError> {
        // will rollback prefix to original value at the end of the function
        let origin_key_prefix_len = self.cur_key_prefix.bits();

        let label_type = self.detect_label_type(parser)?;
        match label_type {
            DictLabelType::Same => {
                let prefix_val = parser.read_bit()?;
                let prefix_len_len = self.remain_suffix_bit_len();
                let prefix_len = parser.read_num::<usize>(prefix_len_len)?;
                if prefix_val {
                    self.cur_key_prefix += 1u32;
                    self.cur_key_prefix <<= prefix_len;
                    self.cur_key_prefix -= 1u32;
                } else {
                    self.cur_key_prefix <<= prefix_len;
                }
            }
            DictLabelType::Short => {
                let prefix_len = UnaryLen::read(parser)?;
                if *prefix_len != 0 {
                    let val = parser.read_num::<BigUint>(*prefix_len)?;
                    self.cur_key_prefix <<= *prefix_len;
                    self.cur_key_prefix |= val;
                }
            }
            DictLabelType::Long => {
                let prefix_len_len = self.remain_suffix_bit_len();
                let prefix_len: usize = parser.read_num(prefix_len_len)?;
                if prefix_len_len != 0 {
                    let val: BigUint = parser.read_num(prefix_len)?;
                    self.cur_key_prefix <<= prefix_len;
                    self.cur_key_prefix |= val;
                }
            }
        }
        if self.cur_key_prefix.bits() as usize == (self.key_bits_len + 1) {
            let mut key = BigUint::one() << self.key_bits_len;
            key ^= &self.cur_key_prefix;
            dst.insert(key, VA::read(parser)?);
        } else {
            let left_ref = parser.read_next_ref()?;
            self.cur_key_prefix <<= 1;
            self.parse_impl::<T, VA>(&mut left_ref.parser(), dst)?;

            let right_ref = parser.read_next_ref()?;
            self.cur_key_prefix += BigUint::one();
            self.parse_impl::<T, VA>(&mut right_ref.parser(), dst)?;
        }
        self.cur_key_prefix >>= self.cur_key_prefix.bits() - origin_key_prefix_len;
        Ok(())
    }

    fn detect_label_type(&self, parser: &mut CellParser) -> Result<DictLabelType, TonCoreError> {
        let label = if parser.read_bit()? {
            if parser.read_bit()? {
                DictLabelType::Same
            } else {
                DictLabelType::Long
            }
        } else {
            DictLabelType::Short
        };
        Ok(label)
    }

    fn remain_suffix_bit_len(&self) -> usize {
        // add 2 because cur_prefix contains leading bit
        let prefix_len_left = self.key_bits_len - self.cur_key_prefix.bits() as usize + 2;
        (prefix_len_left as f32).log2().ceil() as usize
    }
}
