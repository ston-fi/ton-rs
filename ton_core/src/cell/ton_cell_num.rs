use crate::cell::{CellBuilder, CellParser};

use std::fmt::Display;
mod ton_cell_bignum;
mod ton_cell_fastnum;
mod ton_cell_primitives;

use crate::errors::{TonCoreError, TonCoreResult};

/// Allows generic read/write operation for any numeric type
pub trait TonCellNum: Display + Sized + Clone {
    /// CellBuilder guarantees 0 < bits_len < 1024
    fn tcn_write_bits(&self, writer: &mut CellBuilder, bits_len: u32) -> Result<(), TonCoreError>;
    /// CellWriter guarantees 0 <= bits_len < 1024
    fn tcn_read_bits(reader: &mut CellParser, bits_len: u32) -> Result<Self, TonCoreError>;
    fn tcn_is_zero(&self) -> bool;
    fn tcn_min_bits_len(&self) -> u32;

    /// Returns the maximum bit size for padding purposes in write_num
    /// For fixed-size types, this is sizeof(T) * 8
    /// For BigInt/BigUint, this is 1024 (same as I1024/U1024)
    fn tcn_max_bits_len() -> u32;
}

#[macro_export]
macro_rules! unsinged_highest_bit_pos {
    ($val:expr,$T:ty) => {{
        let max_bit_id = (std::mem::size_of::<$T>() * 8 - 1) as u32;
        (max_bit_id - $val.leading_zeros())
    }};
}

#[macro_export]
macro_rules! toncellnum_use_type_as {
    (
        $Src:ty,
        $Dst:ty,
        $src_to_dst:expr,   // fn(Src) -> TonCoreResult<Dst>
        $dst_to_src:expr   // fn(Dst) -> TonCoreResult<Src>
    ) => {
        impl TonCellNum for $Src {
            fn tcn_write_bits(&self, writer: &mut CellBuilder, bits_len: u32) -> Result<(), TonCoreError> {
                // fallible Src -> Dst
                let val_as: $Dst = $src_to_dst(self)?;
                val_as.tcn_write_bits(writer, bits_len)
            }

            fn tcn_read_bits(reader: &mut CellParser, bits_len: u32) -> Result<Self, TonCoreError> {
                let val_as = <$Dst>::tcn_read_bits(reader, bits_len)?;
                // fallible Dst -> Src, already returns TonCoreError
                $dst_to_src(val_as)
            }

            fn tcn_is_zero(&self) -> bool {
                // If conversion here can *theoretically* fail, treat it as a logic bug.
                let val_as: $Dst =
                    $src_to_dst(self).expect("toncellnum_use_type_as: Src -> Dst conversion failed in tcn_is_zero");
                val_as.tcn_is_zero()
            }

            fn tcn_min_bits_len(&self) -> u32 {
                let val_as: $Dst = $src_to_dst(self)
                    .expect("toncellnum_use_type_as: Src -> Dst conversion failed in tcn_min_bits_len");
                val_as.tcn_min_bits_len()
            }

            fn tcn_max_bits_len() -> u32 { <$Dst>::tcn_max_bits_len() }
        }
    };
}

pub(crate) fn toncellnum_bigendian_bit_cutter(mut target: Vec<u8>, bits_count: u32) -> Vec<u8> {
    let total_bits = (target.len() * 8) as u32;
    assert!(bits_count <= total_bits, "bits_count {} > total_bits {}", bits_count, total_bits); // remove after asking needed behavior

    let bits_to_cut = total_bits - bits_count;
    if bits_to_cut == 0 {
        return target;
    }

    // Cut full bytes from the right.
    let full_bytes_to_cut = (bits_to_cut / 8) as usize;
    let bits_in_last_byte_to_cut = (bits_to_cut % 8) as u8;

    if full_bytes_to_cut > 0 {
        let new_len = target.len() - full_bytes_to_cut;
        target.truncate(new_len);
    }

    // Zero the lowest `bits_in_last_byte_to_cut` bits in the last byte.
    if bits_in_last_byte_to_cut > 0 {
        let last_idx = target.len() - 1;
        let mask: u8 = 0xFF << bits_in_last_byte_to_cut; // keep high bits, zero low
        target[last_idx] &= mask;
    }
    target
}
pub(crate) fn toncellnum_bigendian_bit_restorator(
    in_array: Vec<u8>,
    bits_count: u32,
    out_array_bytes: u32,
    fill_bit_value: bool,
) -> Vec<u8> {
    let out_total_bits = out_array_bytes * 8;
    assert!(bits_count <= out_total_bits, "bits_count {} > out_total_bits {}", bits_count, out_total_bits);

    let in_bytes = in_array.len();
    let out_bytes = out_array_bytes as usize;
    assert!(in_bytes <= out_bytes, "input has more bytes ({}) than output ({})", in_bytes, out_bytes);

    // Start with zeroed output.
    let mut out = if fill_bit_value {
        vec![0xFFu8; out_bytes]
    } else {
        vec![0u8; out_bytes]
    };

    // Copy existing high-order bytes to the left side (big-endian).
    // Example: in=2 bytes, out=4 bytes -> copy to indices [0,1], new bytes at [2,3].
    out[..in_bytes].copy_from_slice(&in_array);

    // Now fill restored bits (from bits_count to end) with fill_bit_value.
    for bit_idx in bits_count..out_total_bits {
        let byte_index = (bit_idx / 8) as usize;
        let bit_in_byte = 7 - (bit_idx % 8); // MSB-first inside byte
        let mask = 1u8 << bit_in_byte;

        if fill_bit_value {
            out[byte_index] |= mask; // set bit to 1
        } else {
            out[byte_index] &= !mask; // set bit to 0
        }
    }

    out
}

pub(crate) fn toncellnum_restore_bits_as_signed(in_array: Vec<u8>, bits_count: u32, out_array_bytes: u32) -> Vec<u8> {
    let sign_bit_index = bits_count - 1;
    let byte_index = (sign_bit_index / 8) as usize;
    let bit_in_byte = 7 - (sign_bit_index % 8); // MSB-first inside byte
    let sign_bit = (in_array[byte_index] >> bit_in_byte) & 1 == 1;

    toncellnum_bigendian_bit_restorator(in_array, bits_count, out_array_bytes, sign_bit)
}

#[cfg(test)]
mod tests {

    use crate::cell::{CellParser, TonCell};
    use crate::cell::{TonCellNum, toncellnum_restore_bits_as_signed};
    use fastnum::*;
    use num_bigint::{BigInt, BigUint};
    use std::fmt::Debug;

    use crate::cell::ton_cell_num::ton_cell_fastnum::fastnum_to_big_endian_bytes_unsigned;
    use bitstream_io::{BigEndian, BitRead, BitReader, BitWriter};

    //
    pub(crate) fn test_num_read_write<T: TonCellNum + PartialEq + Debug>(
        test_cases: Vec<(T, u32)>,
        type_name: &str,
    ) -> anyhow::Result<()> {
        for (value, bits_len) in test_cases {
            let mut builder = TonCell::builder();
            builder.write_num(&value, bits_len as usize).unwrap();
            let cell = builder.build().unwrap();
            let mut parser = CellParser::new(&cell);
            let parsed = parser.read_num::<T>(bits_len as usize).unwrap();
            assert_eq!(parsed, value, "Failed for {} value {:?} with {} bits", type_name, value, bits_len);
        }
        Ok(())
    }

    #[test]
    fn test_toncellnum_bit_cutter_and_restorator() {
        for i in 2..64 {
            let data_store = vec![0xff; 8];

            let mut builder = TonCell::builder();
            builder.write_bits(&data_store, i).unwrap();
            let cell = builder.build().unwrap();
            let mut parser = CellParser::new(&cell);

            let data_vec = parser.read_bits(i).unwrap();
            let data_vec = toncellnum_restore_bits_as_signed(data_vec, i as u32, 8);
            assert_eq!(data_store, data_vec);
        }
    }

    #[test]
    fn test_toncellnum_write_num() -> anyhow::Result<()> {
        let num_val = -9i32;
        let bigint_val = BigInt::from(-9i32);
        let i512_val = -I512::from(9i32);

        let mut num_builder = TonCell::builder();
        let mut bigint_builder = TonCell::builder();
        let mut i512_builder = TonCell::builder();

        num_builder.write_num(&num_val, 10).unwrap();
        bigint_builder.write_num(&bigint_val, 10).unwrap();
        let num_cell = num_builder.build().unwrap();
        i512_builder.write_num(&i512_val, 10).unwrap();

        // Two's complement: -9 in 10 bits = 1111110111
        // As bytes (10 bits = 2 bytes): 11111101 11000000 = [253, 192]
        assert_eq!(*num_cell.cell_data.data_storage, vec![0b1111_1101u8, 0b1100_0000u8]);
        assert_eq!(*num_cell.cell_data.data_storage, *bigint_builder.build()?.cell_data.data_storage);
        assert_eq!(*num_cell.cell_data.data_storage, *i512_builder.build()?.cell_data.data_storage);
        Ok(())
    }

    // Test reading and writing zero bits for all supported numeric types
    #[test]
    fn test_toncellnum_zero_bits_all_types() -> anyhow::Result<()> {
        // Test that all types return 0 when reading/writing 0 bits
        let builder = TonCell::builder();
        let cell = builder.build()?;
        let mut parser = CellParser::new(&cell);

        // Test unsigned primitives
        assert_eq!(parser.read_num::<u8>(0)?, 0u8);
        assert_eq!(parser.read_num::<u16>(0)?, 0u16);
        assert_eq!(parser.read_num::<u32>(0)?, 0u32);
        assert_eq!(parser.read_num::<u64>(0)?, 0u64);
        assert_eq!(parser.read_num::<u128>(0)?, 0u128);

        // Test signed primitives
        assert_eq!(parser.read_num::<i8>(0)?, 0i8);
        assert_eq!(parser.read_num::<i16>(0)?, 0i16);
        assert_eq!(parser.read_num::<i32>(0)?, 0i32);
        assert_eq!(parser.read_num::<i64>(0)?, 0i64);
        assert_eq!(parser.read_num::<i128>(0)?, 0i128);

        // Test usize
        assert_eq!(parser.read_num::<usize>(0)?, 0usize);

        // Test BigUint and BigInt
        assert_eq!(parser.read_num::<BigUint>(0)?, BigUint::from(0u32));
        assert_eq!(parser.read_num::<BigInt>(0)?, BigInt::from(0i32));

        // Test fastnum unsigned
        assert_eq!(parser.read_num::<U128>(0)?, U128::from(0u32));
        assert_eq!(parser.read_num::<U256>(0)?, U256::from(0u32));
        assert_eq!(parser.read_num::<U512>(0)?, U512::from(0u32));
        assert_eq!(parser.read_num::<U1024>(0)?, U1024::from(0u32));

        // Test fastnum signed
        assert_eq!(parser.read_num::<I128>(0)?, I128::from(0u32));
        assert_eq!(parser.read_num::<I256>(0)?, I256::from(0u32));
        assert_eq!(parser.read_num::<I512>(0)?, I512::from(0u32));
        assert_eq!(parser.read_num::<I1024>(0)?, I1024::from(0u32));

        Ok(())
    }
}
