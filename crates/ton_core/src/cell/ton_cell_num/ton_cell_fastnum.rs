use crate::bail_ton_core_data;
use crate::cell::TonCellNum;
use crate::cell::{CellBuilder, CellParser};
use std::any::type_name;

use crate::errors::{TonCoreError, TonCoreResult};
use fastnum::{I128, I256, I512, I1024};
use fastnum::{U128, U256, U512, U1024};

macro_rules! ton_cell_num_fastnum_unsigned_impl {
    ($src:ty) => {
        impl TonCellNum for $src {
            fn tcn_write_bits(&self, writer: &mut CellBuilder, bits_len: usize) -> TonCoreResult<()> {
                let slice = self.to_radix_be(256);
                let slice_len_bits = slice.len() * 8;
                let padding_bits = bits_len.saturating_sub(slice_len_bits);
                if padding_bits > 0 {
                    writer.write_bits_with_offset(&[0u8; 128], 0, padding_bits)?;
                }
                let bits_to_write = bits_len - padding_bits;
                writer.write_bits_with_offset(&slice, slice_len_bits - bits_to_write, bits_to_write)
            }

            fn tcn_read_bits(reader: &mut CellParser, bits_len: usize) -> TonCoreResult<Self> {
                let type_size_bits = size_of::<Self>() * 8;
                let padding_bits = bits_len.saturating_sub(type_size_bits);
                if padding_bits > 0 {
                    reader.read_bits_to(padding_bits, &mut [0u8; 128])?; // skip padding
                }
                let mut dst = [0u8; size_of::<Self>()];
                let bits_to_read = bits_len - padding_bits;
                reader.read_bits_to(bits_to_read, &mut dst)?;
                let value = match Self::from_be_slice(&dst) {
                    Some(v) => v,
                    None => bail_ton_core_data!("Failed to read {} from slice: {dst:?}", type_name::<Self>()),
                };
                Ok(value >> (type_size_bits - bits_to_read))
            }

            fn tcn_min_bits_len(&self) -> usize { size_of::<Self>() * 8 - self.leading_zeros() as usize }
        }
    };
}

macro_rules! ton_cell_num_fastnum_signed_impl {
    ($src:ty,$u_src:ty) => {
        impl TonCellNum for $src {
            fn tcn_write_bits(&self, writer: &mut CellBuilder, bits_len: usize) -> Result<(), TonCoreError> {
                let slice = self.to_radix_be(256);
                let slice_len_bits = slice.len() * 8;
                let padding_bits = bits_len.saturating_sub(slice_len_bits);
                if padding_bits > 0 {
                    let padding_val = if self >= &Self::ZERO { 0 } else { 255 };
                    writer.write_bits_with_offset(&[padding_val; 128], 0, padding_bits)?;
                }
                let bits_to_write = bits_len - padding_bits;
                writer.write_bits_with_offset(slice, slice_len_bits - bits_to_write, bits_to_write)
            }

            fn tcn_read_bits(reader: &mut CellParser, bits_len: usize) -> Result<Self, TonCoreError> {
                let type_size_bits = size_of::<Self>() * 8;
                let padding_bits = bits_len.saturating_sub(type_size_bits);
                if padding_bits > 0 {
                    reader.read_bits_to(padding_bits, &mut [0u8; 128])?; // skip padding
                }
                let bits_to_read = bits_len - padding_bits;
                let mut dst = [0u8; size_of::<Self>()];
                reader.read_bits_to(bits_to_read, &mut dst)?;
                let value = match Self::from_be_slice(&dst) {
                    Some(v) => v,
                    None => bail_ton_core_data!("Failed to read {} from slice: {dst:?}", type_name::<Self>()),
                };
                Ok(value >> (type_size_bits - bits_to_read))
            }

            fn tcn_min_bits_len(&self) -> usize {
                let type_size_bits = size_of::<Self>() * 8;
                if self >= &Self::ZERO {
                    return (type_size_bits - self.leading_zeros() as usize) + 1;
                }
                type_size_bits - self.leading_ones() as usize + 1
            }
        }
    };
}

ton_cell_num_fastnum_unsigned_impl!(U128);
ton_cell_num_fastnum_unsigned_impl!(U256);
ton_cell_num_fastnum_unsigned_impl!(U512);
ton_cell_num_fastnum_unsigned_impl!(U1024);

ton_cell_num_fastnum_signed_impl!(I128, U128);
ton_cell_num_fastnum_signed_impl!(I256, U256);
ton_cell_num_fastnum_signed_impl!(I512, U512);
ton_cell_num_fastnum_signed_impl!(I1024, U1024);
