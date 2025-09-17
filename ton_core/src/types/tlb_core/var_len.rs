use crate::bail_ton_core_data;
use crate::cell::CellBuilder;
use crate::cell::CellParser;
use crate::cell::TonCellNum;
use crate::errors::TonCoreError;
use crate::traits::tlb::TLB;
use std::ops::{Deref, DerefMut};

pub type VarLenBits<T, const LEN_BITS_LEN: usize> = VarLen<T, LEN_BITS_LEN, false>;
pub type VarLenBytes<T, const LEN_BITS_LEN: usize> = VarLen<T, LEN_BITS_LEN, true>;

/// VarLen: store data len, and then data itself
///
/// BITS_LEN_LEN - number of bits used to store length
/// LEN_IN_BYTES - if true, data len is specified in bytes. Otherwise - in bits
#[derive(Debug, Clone, Eq, Hash, Ord, PartialOrd, PartialEq)]
pub struct VarLen<T, const LEN_BITS_LEN: usize, const LEN_IN_BYTES: bool> {
    pub data: T,
    pub bits_len: usize,
}

impl<T, const LEN_BITS_LEN: usize, const LEN_IN_BYTES: bool> VarLen<T, LEN_BITS_LEN, LEN_IN_BYTES> {
    pub fn new<D: Into<T>>(data: D, bits_len: usize) -> Self {
        Self {
            data: data.into(),
            bits_len,
        }
    }
}

impl<T, const L: usize, const BL: bool> Deref for VarLen<T, L, BL> {
    type Target = T;
    fn deref(&self) -> &Self::Target { &self.data }
}

impl<T, const L: usize, const BL: bool> DerefMut for VarLen<T, L, BL> {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.data }
}

// TonNum impl
impl<T: TonCellNum, const LEN_BITS_LEN: usize, const LEN_IN_BYTES: bool> TLB for VarLen<T, LEN_BITS_LEN, LEN_IN_BYTES> {
    fn read_definition(parser: &mut CellParser) -> Result<Self, TonCoreError> {
        let len = parser.read_num(LEN_BITS_LEN)?;
        let bits_len = if LEN_IN_BYTES { len * 8 } else { len };
        let data = parser.read_num(bits_len)?;
        Ok(Self { data, bits_len })
    }

    fn write_definition(&self, builder: &mut CellBuilder) -> Result<(), TonCoreError> {
        if LEN_IN_BYTES && self.bits_len % 8 != 0 {
            bail_ton_core_data!("VarLen: len in bits must be multiple of 8, but got {}", self.bits_len);
        }
        let len = if LEN_IN_BYTES { self.bits_len / 8 } else { self.bits_len };
        builder.write_num(&len, LEN_BITS_LEN)?;
        builder.write_num(&self.data, self.bits_len)?;
        Ok(())
    }
}

// Vec impl
impl<const LEN_BITS_LEN: usize, const LEN_IN_BYTES: bool> TLB for VarLen<Vec<u8>, LEN_BITS_LEN, LEN_IN_BYTES> {
    fn read_definition(parser: &mut CellParser) -> Result<Self, TonCoreError> {
        let len = parser.read_num(LEN_BITS_LEN)?;
        let bits_len = if LEN_IN_BYTES { len * 8 } else { len };
        let data = parser.read_bits(bits_len)?;
        Ok(Self { data, bits_len })
    }

    fn write_definition(&self, builder: &mut CellBuilder) -> Result<(), TonCoreError> {
        if LEN_IN_BYTES && self.bits_len % 8 != 0 {
            bail_ton_core_data!("VarLen: len in bytes must be multiple of 8, but got {}", self.bits_len);
        }
        let len = if LEN_IN_BYTES { self.bits_len / 8 } else { self.bits_len };
        builder.write_num(&len, LEN_BITS_LEN)?;
        builder.write_bits(&self.data, self.bits_len)?;
        Ok(())
    }
}

impl Copy for VarLenBytes<u128, 4> {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::TonCell;
    use num_bigint::BigUint;

    #[test]
    fn test_block_tlb_var_len_bits_num() -> anyhow::Result<()> {
        // len in bits
        let obj = VarLenBits::<u32, 8>::new(1u8, 4);
        let cell = obj.to_cell()?;
        // 8 bits of length (value = 4) + 4 bits of data (value = 1)
        assert_eq!(&cell.data, &[0b00000100, 0b00010000]);
        let parsed = VarLenBits::<u32, 8>::from_cell(&cell)?;
        assert_eq!(obj, parsed);
        Ok(())
    }

    #[test]
    fn test_block_tlb_var_len_bytes_primitive_num() -> anyhow::Result<()> {
        // len in bytes
        let obj = VarLenBytes::<u32, 16>::new(1u8, 24);
        let cell = obj.to_cell()?;
        // 16 bits of length (value = 3 bytes <==> 24 bits), and then 24 bits of data (value = 1)
        assert_eq!(&cell.data, &[0b00000000, 0b00000011, 0b00000000, 0b00000000, 0b00000001]);
        let parsed = VarLenBytes::<u32, 16>::from_cell(&cell)?;
        assert_eq!(obj, parsed);

        // the same, additionally check builder state
        let mut builder = TonCell::builder();
        obj.write(&mut builder)?;
        assert_eq!(builder.data_bits_left(), 1023 - 40);

        Ok(())
    }

    #[test]
    fn test_block_tlb_var_len_bytes_big_uint() -> anyhow::Result<()> {
        // len in bytes
        let obj = VarLenBytes::<BigUint, 16>::new(1u8, 24);
        let cell = obj.to_cell()?;
        // 16 bits of length (value = 3 bytes <==> 24 bits), and then 24 bits of data (value = 1)
        assert_eq!(&cell.data, &[0b00000000, 0b00000011, 0b00000000, 0b00000000, 0b00000001]);
        let parsed = VarLenBytes::<BigUint, 16>::from_cell(&cell)?;
        assert_eq!(obj, parsed);

        // the same, additionally check builder state
        let mut builder = TonCell::builder();
        obj.write(&mut builder)?;
        assert_eq!(builder.data_bits_left(), 1023 - 40);

        Ok(())
    }
}
