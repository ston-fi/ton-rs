use crate::cell::{CellBuilder, CellParser};

use std::fmt::Display;
mod ton_cell_bignum;
mod ton_cell_fastnum;
mod ton_cell_primitives;

use crate::errors::{TonCoreError, TonCoreResult};

/// Allows generic read/write operation for any numeric type
pub trait TonCellNum: Display + Sized + Clone {
    fn tcn_write_bits(&self, writer: &mut CellBuilder, bits_len: u32) -> Result<(), TonCoreError>;
    fn tcn_read_bits(reader: &mut CellParser, bits_len: u32) -> Result<Self, TonCoreError>;
    fn tcn_is_zero(&self) -> bool;
    fn tcn_min_bits_len(&self) -> u32;

    fn tcn_sizeof_bytes() -> u32;
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
                $dst_to_src(val_as)
            }

            fn tcn_is_zero(&self) -> bool {
                let Ok(val_as) = $src_to_dst(self) else {
                    return false;
                };
                val_as.tcn_is_zero()
            }

            fn tcn_min_bits_len(&self) -> u32 {
                let Ok(val_as) = $src_to_dst(self) else {
                    return 0;
                };
                val_as.tcn_min_bits_len()
            }
            fn tcn_sizeof_bytes() -> u32 { <$Dst>::tcn_sizeof_bytes() }
        }
    };
}
#[inline(always)]
fn set_bit_bigendian(bits_array: &mut Vec<u8>, bit_pos: u32, bit_val: bool) {
    let total_bytes = bits_array.len();
    // In big-endian, bit_pos 0 is the LSB (last byte, bit 0)
    // bit_pos increases towards MSB (first byte, bit 7)
    let byte_index = total_bytes - 1 - (bit_pos / 8) as usize;
    let bit_in_byte = (bit_pos % 8) as usize; // LSB-first inside byte (bit 0 is LSB)
    let mask = 1u8 << bit_in_byte;

    if bit_val {
        bits_array[byte_index] |= mask; // set bit to 1
    } else {
        bits_array[byte_index] &= !mask; // set bit to 0
    }
}
#[inline(always)]
fn get_bit_bigendian(bits_array: &Vec<u8>, bit_pos: u32) -> bool {
    let total_bytes = bits_array.len();
    // In big-endian, bit_pos 0 is the LSB (last byte, bit 0)
    // bit_pos increases towards MSB (first byte, bit 7)
    let byte_index = total_bytes - 1 - (bit_pos / 8) as usize;
    let bit_in_byte = (bit_pos % 8) as usize; // LSB-first inside byte (bit 0 is LSB)
    let mask = 1u8 << bit_in_byte;

    (bits_array[byte_index] & mask) != 0
}

pub(crate) fn toncellnum_bigendian_bit_reader(
    reader: &mut CellParser,
    bits_count: u32,
    out_array_bytes: u32,
    is_sigend: bool,
) -> TonCoreResult<Vec<u8>> {
    let total_out_bits = out_array_bytes * 8;
    let mut out = vec![0u8; out_array_bytes as usize];
    // Handle zero bits case
    if bits_count == 0 {
        return Ok(out);
    }

    for bit_index in 0..bits_count {
        let bit_index = bits_count - 1 - bit_index;
        let bit_val = reader.read_bit()?;
        set_bit_bigendian(&mut out, bit_index, bit_val);
    }

    if is_sigend {
        // sign extend
        let sign_bit = get_bit_bigendian(&out, bits_count - 1);
        if sign_bit {
            for bit_index in bits_count..total_out_bits {
                set_bit_bigendian(&mut out, bit_index, true);
            }
        }
    }
    assert_eq!(out.len() as u32, out_array_bytes);
    Ok(out)
}

pub(crate) fn toncellnum_bigendian_bit_writer(
    writer: &mut CellBuilder,
    bytes_array: &Vec<u8>,
    bits_count: u32,
) -> TonCoreResult<()> {
    for bit_index in 0..bits_count {
        let bit_index = bits_count - 1 - bit_index;
        writer.write_bit(get_bit_bigendian(bytes_array, bit_index)).unwrap();
    }
    Ok(())
}

#[cfg(test)]
mod tests {

    use crate::cell::TonCellNum;
    use crate::cell::{CellParser, TonCell};
    use fastnum::*;
    use num_bigint::{BigInt, BigUint};
    use std::fmt::Debug;

    use crate::cell::ton_cell_num::{get_bit_bigendian, set_bit_bigendian};

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
    fn test_toncellnum_bytes_read_write() {
        let uval = 123u16;
        let array = uval.to_be_bytes().to_vec();
        let mut out_arr = vec![0u8; array.len()];

        for i in 0..array.len() * 8 {
            let from_api = get_bit_bigendian(&array, i as u32);
            set_bit_bigendian(&mut out_arr, i as u32, from_api);
        }
        for i in 0..array.len() {
            let from_val = uval & (1u16 << i) != 0;
            let from_api = get_bit_bigendian(&array, i as u32);
            assert_eq!(from_val, from_api);
        }
        assert_eq!(array, out_arr);
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
