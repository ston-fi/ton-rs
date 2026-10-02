use std::collections::HashMap;

use super::label_type::DictLabelType;
use crate::tlb_adapters::DictValAdapter;
use num_bigint::BigUint;
use num_traits::One;
use ton_core::bail_ton_core_data;
use ton_core::cell::CellParser;
use ton_core::errors::TonCoreError;
use ton_core::traits::tlb::TLB;
use ton_core::types::tlb_core::adapters::UnaryLen;

pub struct DictDataParser {
    key_bits_len: usize,
    cur_key_prefix: BigUint, // store leading 1 to determinate len properly
}

impl DictDataParser {
    /// Creates a parser with an empty key prefix and the specified fixed key length.
    pub fn new(key_len_bits: usize) -> DictDataParser {
        DictDataParser {
            key_bits_len: key_len_bits,
            cur_key_prefix: BigUint::one(),
        }
    }

    pub fn read<VA: DictValAdapter>(
        &mut self,
        parser: &mut CellParser,
    ) -> Result<HashMap<BigUint, VA::ValType>, TonCoreError> {
        // reset state in case of reusing
        self.cur_key_prefix = BigUint::one();

        let mut result = HashMap::new();
        self.parse_impl::<VA>(parser, &mut result)?;
        Ok(result)
    }

    fn parse_impl<VA: DictValAdapter>(
        &mut self,
        parser: &mut CellParser,
        dst: &mut HashMap<BigUint, VA::ValType>,
    ) -> Result<(), TonCoreError> {
        // will rollback prefix to original value at the end of the function
        let origin_key_prefix_len = self.cur_key_prefix.bits();

        let remaining = self.key_bits_len - (origin_key_prefix_len as usize - 1);
        let len_bits = (usize::BITS - remaining.leading_zeros()) as usize;
        let (prefix_len, repeated_bit) = match self.detect_label_type(parser)? {
            DictLabelType::Same => {
                let bit = parser.read_bit()?;
                (parser.read_num::<usize>(len_bits)?, Some(bit))
            },
            DictLabelType::Short => (*UnaryLen::read(parser)?, None),
            DictLabelType::Long => (parser.read_num::<usize>(len_bits)?, None),
        };
        if prefix_len > remaining {
            bail_ton_core_data!("dictionary label length {prefix_len} exceeds remaining key width {remaining}");
        }
        if let Some(bit) = repeated_bit {
            if bit {
                self.cur_key_prefix += 1u32;
                self.cur_key_prefix <<= prefix_len;
                self.cur_key_prefix -= 1u32;
            } else {
                self.cur_key_prefix <<= prefix_len;
            }
        } else if prefix_len != 0 {
            let val = parser.read_num::<BigUint>(prefix_len)?;
            self.cur_key_prefix <<= prefix_len;
            self.cur_key_prefix |= val;
        }
        if self.cur_key_prefix.bits() as usize == (self.key_bits_len + 1) {
            let mut key = BigUint::one() << self.key_bits_len;
            key ^= &self.cur_key_prefix;
            dst.insert(key, VA::read(parser)?);
        } else {
            let left_ref = parser.read_next_ref()?;
            self.cur_key_prefix <<= 1;
            self.parse_impl::<VA>(&mut left_ref.parser(), dst)?;

            let right_ref = parser.read_next_ref()?;
            self.cur_key_prefix += BigUint::one();
            self.parse_impl::<VA>(&mut right_ref.parser(), dst)?;
        }
        self.cur_key_prefix >>= self.cur_key_prefix.bits() - origin_key_prefix_len;
        Ok(())
    }

    fn detect_label_type(&self, parser: &mut CellParser) -> Result<DictLabelType, TonCoreError> {
        let label = if parser.read_bit()? {
            if parser.read_bit()? { DictLabelType::Same } else { DictLabelType::Long }
        } else {
            DictLabelType::Short
        };
        Ok(label)
    }
}
