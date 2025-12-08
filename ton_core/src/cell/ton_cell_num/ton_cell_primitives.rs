use crate::cell::TonCellNum;
use crate::cell::{CellBuilder, CellParser};

use crate::errors::{TonCoreError, TonCoreResult};

/// Uses native bitstream operations
macro_rules! ton_cell_num_primitive_unsigned_impl {
    ($src:ty) => {
        impl TonCellNum for $src {
            fn tcn_write_bits(&self, writer: &mut CellBuilder, bits_len: usize) -> TonCoreResult<()> {
                let padding_bits = bits_len.saturating_sub(Self::BITS as usize);
                if padding_bits > 0 {
                    writer.write_bits_with_offset(&[0u8; 128], 0, padding_bits)?;
                }
                writer.write_unsigned_primitive(*self, bits_len - padding_bits) // the most optimized way
            }

            fn tcn_read_bits(reader: &mut CellParser, bits_len: usize) -> TonCoreResult<Self> {
                let padding_bits = bits_len.saturating_sub(Self::BITS as usize);
                if padding_bits > 0 {
                    reader.read_bits_to(padding_bits, &mut [0u8; 128])?; // skip padding
                }
                reader.read_unsigned_primitive(bits_len - padding_bits)
            }

            fn tcn_min_bits_len(&self) -> usize { (Self::BITS - self.leading_zeros()) as usize }
        }
    };
}

macro_rules! ton_cell_num_primitive_signed_impl {
    ($src:ty,$u_src:ty) => {
        impl TonCellNum for $src {
            fn tcn_write_bits(&self, writer: &mut CellBuilder, bits_len: usize) -> Result<(), TonCoreError> {
                let padding_bits = bits_len.saturating_sub(Self::BITS as usize);
                if padding_bits > 0 {
                    let padding_value = if *self >= 0 { 0 } else { 255 };
                    writer.write_bits_with_offset(&[padding_value; 128], 0, padding_bits)?;
                }
                let bits_to_write = bits_len - padding_bits;
                // unsigned extension - set non-relevant bits to zero
                let shift_bits = Self::BITS as usize - bits_to_write;
                let unsigned = (*self as $u_src) << shift_bits >> shift_bits;
                unsigned.tcn_write_bits(writer, bits_to_write)
            }

            fn tcn_read_bits(reader: &mut CellParser, bits_len: usize) -> Result<Self, TonCoreError> {
                let unsigned = <$u_src>::tcn_read_bits(reader, bits_len)?;
                if bits_len >= Self::BITS as usize {
                    return Ok(unsigned as $src);
                }
                // signed extension - set non-relevant bits to sign bit
                let shift_bits = Self::BITS as usize - bits_len;
                let signed = (unsigned as $src) << shift_bits >> shift_bits;
                Ok(signed)
            }

            fn tcn_min_bits_len(&self) -> usize {
                let type_size_bits = size_of::<Self>() * 8;
                if *self >= 0 {
                    return (type_size_bits - self.leading_zeros() as usize) + 1;
                }
                type_size_bits - self.leading_ones() as usize + 1
            }
        }
    };
}

#[rustfmt::skip]
impl TonCellNum for usize {
    fn tcn_write_bits(&self, writer: &mut CellBuilder, bits_len: usize) -> TonCoreResult<()> { (*self as u64).tcn_write_bits(writer, bits_len) }
    fn tcn_read_bits(reader: &mut CellParser, bits_len: usize) -> TonCoreResult<Self> { Ok(u64::tcn_read_bits(reader, bits_len)? as usize) }
    fn tcn_min_bits_len(&self) -> usize { (Self::BITS - self.leading_zeros()) as usize }
}

ton_cell_num_primitive_unsigned_impl!(u8);
ton_cell_num_primitive_unsigned_impl!(u16);
ton_cell_num_primitive_unsigned_impl!(u32);
ton_cell_num_primitive_unsigned_impl!(u64);
ton_cell_num_primitive_unsigned_impl!(u128);

ton_cell_num_primitive_signed_impl!(i8, u8);
ton_cell_num_primitive_signed_impl!(i16, u16);
ton_cell_num_primitive_signed_impl!(i32, u32);
ton_cell_num_primitive_signed_impl!(i64, u64);
ton_cell_num_primitive_signed_impl!(i128, u128);
ton_cell_num_primitive_signed_impl!(isize, u64);
