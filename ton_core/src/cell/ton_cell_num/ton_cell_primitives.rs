use crate::bail_ton_core_data;
use crate::cell::TonCellNum;
use crate::cell::{CellBuilder, CellParser};
use crate::unsinged_highest_bit_pos;

use crate::errors::{TonCoreError, TonCoreResult};
use crate::toncellnum_use_type_as;

macro_rules! primitive_convert_to_unsigned {
    ($val:expr,$T:ty,$bit_count:expr) => {{
        // Two's complement: cast to unsigned and mask to bit_count
        let uval = $val as $T;
        let bit_count = $bit_count as usize;
        let type_bits = std::mem::size_of::<$T>() * 8;

        if bit_count >= type_bits {
            // Full width or larger - no masking needed
            uval
        } else {
            // Mask to bit_count bits
            let mask = ((1 as $T) << bit_count) - 1;
            uval & mask
        }
    }};
}

macro_rules! primitive_convert_to_signed {
    ($uval:expr,$I:ty,$U:ty,$bit_count:expr) => {{
        // Two's complement decoding with sign extension
        let uval = $uval;
        let bit_count = $bit_count as usize;
        let type_bits = std::mem::size_of::<$I>() * 8;

        if bit_count >= type_bits {
            // Full width or larger - just cast
            uval as $I
        } else {
            // Need to sign-extend from bit_count to full width
            let sign_bit = 1 << (bit_count - 1);
            if (uval & sign_bit) != 0 {
                // Negative number - extend with 1s
                let extension_mask = (<$U>::MAX << bit_count);
                (uval | extension_mask) as $I
            } else {
                // Positive number - just cast
                uval as $I
            }
        }
    }};
}

macro_rules! primitive_highest_bit_pos_signed {
    ($val:expr,$T:ty) => {{
        let max_bit_id = (std::mem::size_of::<$T>() * 8 - 1) as u32;
        let val = $val;
        if val < -1 {
            let abs_val = (val + 1).abs();
            let pos_leading = abs_val.leading_zeros();
            max_bit_id - pos_leading
        } else if val >= 0 {
            let pos_leading = val.leading_zeros();
            let pos_result = if pos_leading > 0 { max_bit_id - pos_leading } else { 0 };
            pos_result
        } else {
            0
        }
    }};
}

macro_rules! ton_cell_num_primitive_unsigned_impl {
    ($src:ty) => {
        impl TonCellNum for $src {
            fn tcn_write_bits(&self, writer: &mut CellBuilder, bits_len: u32) -> Result<(), TonCoreError> {
                if self.tcn_min_bits_len() > bits_len {
                    bail_ton_core_data!(
                        "Not enough bits for write num {} in {} bits unsigned, min len {}",
                        *self,
                        bits_len,
                        self.tcn_min_bits_len()
                    );
                }

                writer.write_primitive(bits_len, *self)?;

                Ok(())
            }
            fn tcn_read_bits(reader: &mut CellParser, bits_len: u32) -> TonCoreResult<Self> {
                if bits_len == 0 {
                    return Ok(0);
                }
                let val: $src = reader.read_primitive(bits_len)?;
                Ok(val)
            }
            fn tcn_is_zero(&self) -> bool { *self == 0 }

            fn tcn_min_bits_len(&self) -> u32 {
                if *self == 0 {
                    0u32
                } else {
                    unsinged_highest_bit_pos!(*self, Self) + 1u32
                }
            }

            fn tcn_sizeof_bytes() -> u32 { (std::mem::size_of::<$src>()) as u32 }
        }
    };
}

macro_rules! ton_cell_num_primitive_signed_impl {
    ($src:ty,$u_src:ty) => {
        impl TonCellNum for $src {
            fn tcn_write_bits(&self, writer: &mut CellBuilder, bits_len: u32) -> Result<(), TonCoreError> {
                if self.tcn_min_bits_len() > bits_len {
                    bail_ton_core_data!(
                        "Not enough bits for write num {} in {} bits, min len {}",
                        *self,
                        bits_len,
                        self.tcn_min_bits_len()
                    );
                }

                let uval: $u_src = primitive_convert_to_unsigned!(*self, $u_src, bits_len);
                uval.tcn_write_bits(writer, bits_len)
            }

            fn tcn_read_bits(reader: &mut CellParser, bits_len: u32) -> Result<Self, TonCoreError> {
                if bits_len == 0 {
                    return Ok(0 as Self);
                }
                let uval = <$u_src>::tcn_read_bits(reader, bits_len)?;
                let ret: Self = primitive_convert_to_signed!(uval, Self, $u_src, bits_len);
                Ok(ret)
            }

            fn tcn_is_zero(&self) -> bool { *self == 0 }

            fn tcn_min_bits_len(&self) -> u32 {
                if *self == 0 {
                    0u32
                } else {
                    let type_bits = (std::mem::size_of::<Self>() * 8) as u32;
                    // For MIN values, we need the full bit width
                    if *self == Self::MIN {
                        type_bits
                    } else {
                        primitive_highest_bit_pos_signed!(*self, Self) + 2u32
                    }
                }
            }
            fn tcn_sizeof_bytes() -> u32 { (std::mem::size_of::<$src>()) as u32 }
        }
    };
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

toncellnum_use_type_as!(usize, u64, |v: &usize| -> Result<u64, TonCoreError> { Ok(*v as u64) }, |v: u64| -> Result<
    usize,
    TonCoreError,
> {
    if v > usize::MAX as u64 {
        Err(TonCoreError::data("usize conversion", "Value too large for usize"))
    } else {
        Ok(v as usize)
    }
});

toncellnum_use_type_as!(isize, i64, |v: &isize| -> Result<i64, TonCoreError> { Ok(*v as i64) }, |v: i64| -> Result<
    isize,
    TonCoreError,
> {
    if v > isize::MAX as i64 || v < isize::MIN as i64 {
        Err(TonCoreError::data("isize conversion", "Value out of range for isize"))
    } else {
        Ok(v as isize)
    }
});

#[cfg(test)]
mod tests {

    use crate::cell::ton_cell_num::tests::test_num_read_write;

    #[test]
    fn test_toncellnum_convert_sign_unsign_int16() -> anyhow::Result<()> {
        // Test with 15 bits
        let bits_len = 15;
        let val = -3i16;
        let u_val: u16 = primitive_convert_to_unsigned!(val, u16, bits_len);
        let res_val = primitive_convert_to_signed!(u_val, i16, u16, bits_len);
        assert_eq!(val, res_val);

        // Test with various bit lengths
        for bits_len in [8, 10, 15, 16] {
            let val = -3i16;
            let u_val: u16 = primitive_convert_to_unsigned!(val, u16, bits_len);
            let res_val = primitive_convert_to_signed!(u_val, i16, u16, bits_len);
            assert_eq!(val, res_val, "Failed round-trip for bits_len={}", bits_len);
        }

        // Test edge cases
        let bits_len = 16;
        let val = i16::MIN;
        let u_val: u16 = primitive_convert_to_unsigned!(val, u16, bits_len);
        let res_val = primitive_convert_to_signed!(u_val, i16, u16, bits_len);
        assert_eq!(val, res_val, "Failed for i16::MIN");

        let val = i16::MAX;
        let u_val: u16 = primitive_convert_to_unsigned!(val, u16, bits_len);
        let res_val = primitive_convert_to_signed!(u_val, i16, u16, bits_len);
        assert_eq!(val, res_val, "Failed for i16::MAX");

        Ok(())
    }

    #[test]
    fn test_toncellnum_store_and_parse_usize() -> anyhow::Result<()> { Ok(()) }
    #[test]
    fn test_toncellnum_primitive_higest_bit_pos() -> anyhow::Result<()> {
        assert_eq!(std::mem::size_of::<i8>() as u32 * 8u32, primitive_highest_bit_pos_signed!(i8::MIN, i8) + 2);
        assert_eq!(7, primitive_highest_bit_pos_signed!(-64i8, i8) + 2);

        assert_eq!(std::mem::size_of::<i8>() as u32 * 8u32, primitive_highest_bit_pos_signed!(i8::MAX, i8) + 2);
        Ok(())
    }

    #[test]
    fn test_toncellnum_primitives_corner_cases() {
        // primitive unsigned

        test_num_read_write(vec![(0u8, 8), (u8::MAX, 8)], "u8").unwrap();
        test_num_read_write(vec![(0u16, 16), (u16::MAX, 16)], "u16").unwrap();
        test_num_read_write(vec![(0u32, 32), (u32::MAX, 32)], "u32").unwrap();
        test_num_read_write(vec![(0u64, 64), (u64::MAX, 64)], "u64").unwrap();
        test_num_read_write(vec![(0u128, 128), (u128::MAX, 128)], "u128").unwrap();
        let size_usize = std::mem::size_of::<usize>() as u32 * 8u32;
        // assert_eq!(size_usize,std::mem::size_of::<u64>() as u32 * 8u32);
        test_num_read_write(vec![(0usize, size_usize), (usize::MAX, size_usize)], "usize").unwrap();

        // primitive signed
        test_num_read_write(vec![(0i8, 8), (i8::MAX, 8), (i8::MIN, 8), (i8::MIN / 2, 8)], "i8").unwrap();
        test_num_read_write(vec![(0i16, 16), (i16::MAX, 16), (i16::MIN, 16), (i16::MIN / 2, 16)], "i16").unwrap();
        test_num_read_write(vec![(0i32, 32), (i32::MAX, 32), (i32::MIN, 32), (i32::MIN / 2, 32)], "i32").unwrap();
        test_num_read_write(vec![(0i64, 64), (i64::MAX, 64), (i64::MIN, 64), (i64::MIN / 2, 64)], "i64").unwrap();
        test_num_read_write(vec![(0i128, 128), (i128::MAX, 128), (i128::MIN, 128), (i128::MIN / 2, 128)], "i128")
            .unwrap();

        let size_usize = std::mem::size_of::<isize>() as u32 * 8u32;
        // assert_eq!(size_usize,std::mem::size_of::<u64>() as u32 * 8u32);

        test_num_read_write(
            vec![
                (0isize, size_usize),
                (isize::MAX, size_usize),
                (isize::MIN, size_usize),
                ((isize::MIN / 2), size_usize),
            ],
            "isize",
        )
        .unwrap();
    }
    #[test]
    fn test_toncellnum_primitives_overbits_usage() {
        // primitive unsigned
        test_num_read_write(vec![(255u8, 16), (u8::MAX, 16)], "u8").unwrap();
        test_num_read_write(vec![(65535u16, 32), (u16::MAX, 32)], "u16").unwrap();
        test_num_read_write(vec![(4294967295u32, 64), (u32::MAX, 64)], "u32").unwrap();
        test_num_read_write(vec![(18446744073709551615u64, 128), (u64::MAX, 128)], "u64").unwrap();
        test_num_read_write(vec![(340282366920938463463374607431768211455u128, 256), (u128::MAX, 256)], "u128")
            .unwrap();

        // primitive signed
        test_num_read_write(vec![(127i8, 16), (i8::MAX, 16), (i8::MIN, 16), (i8::MIN / 2, 16)], "i8").unwrap();
        test_num_read_write(vec![(32767i16, 32), (i16::MAX, 32), (i16::MIN, 32), (i16::MIN / 2, 32)], "i16").unwrap();
        test_num_read_write(vec![(2147483647i32, 64), (i32::MAX, 64), (i32::MIN, 64), (i32::MIN / 2, 64)], "i32")
            .unwrap();
        test_num_read_write(
            vec![
                (9223372036854775807i64, 128),
                (i64::MAX, 128),
                (i64::MIN, 128),
                (i64::MIN / 2, 128),
            ],
            "i64",
        )
        .unwrap();
        test_num_read_write(
            vec![
                (170141183460469231731687303715884105727i128, 256),
                (i128::MAX, 256),
                (i128::MIN, 256),
                (i128::MIN / 2, 256),
            ],
            "i128",
        )
        .unwrap();
    }
}
