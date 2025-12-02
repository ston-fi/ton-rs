use crate::bail_ton_core_data;
use crate::cell::TonCellNum;
use crate::cell::{CellBuilder, CellParser};
use crate::cell::{toncellnum_bigendian_bit_reader, toncellnum_bigendian_bit_writer};

use crate::errors::{TonCoreError, TonCoreResult};
use crate::unsinged_highest_bit_pos;
use fastnum::bint::{Int, UInt};
use fastnum::{I128, I256, I512, I1024};
use fastnum::{U128, U256, U512, U1024};

macro_rules! fastnum_highest_bit_pos_signed {
    ($val:expr,$T:ty) => {{
        let max_bit_id = (std::mem::size_of::<$T>() * 8 - 1) as u32;
        let val = $val;
        if val < <$T>::from(-1) {
            let abs_val = (val + <$T>::from(1)).abs();
            let pos_leading = abs_val.leading_zeros();
            max_bit_id - pos_leading
        } else if val >= <$T>::from(0) {
            let pos_leading = val.leading_zeros();
            let pos_result = if pos_leading > 0 { max_bit_id - pos_leading } else { 0 };
            pos_result
        } else {
            0
        }
    }};
}
#[inline]
fn fastnum_from_big_endian_bytes_unsigned<const N: usize>(bytes_array: &[u8]) -> TonCoreResult<UInt<N>> {
    let total_bits = (N * 64) as u32;

    let available_bits = (bytes_array.len() * 8) as u32;
    assert_eq!(available_bits, total_bits);
    let mut answer = UInt::<N>::ZERO;
    for bit_pos in 0..total_bits {
        let out_bit_index = total_bits - 1 - bit_pos; // 0..bits_len-1
        let byte_index = (out_bit_index / 8) as usize;
        let bit_in_byte = 7 - (out_bit_index % 8); // MSB-first inside byte

        let byte = bytes_array[byte_index];
        let bit_is_one = ((byte >> bit_in_byte) & 1) != 0;

        if bit_is_one {
            answer = answer | (UInt::<N>::ONE << bit_pos);
        }
    }
    Ok(answer)
}
#[inline]
fn fastnum_to_big_endian_bytes_unsigned<const N: usize>(src: UInt<N>) -> TonCoreResult<Vec<u8>> {
    let total_bits = (N * 64) as u32;
    let num_bytes = N * 8;
    let mut out = vec![0u8; num_bytes];

    for bit_pos in 0..total_bits {
        let mask = UInt::<N>::ONE << bit_pos;
        let bit_is_one = (src & mask) != UInt::<N>::ZERO;

        if bit_is_one {
            let out_bit_index = total_bits - 1 - bit_pos; // 0..bits_len-1
            let byte_index = (out_bit_index / 8) as usize;
            let bit_in_byte = 7 - (out_bit_index % 8); // MSB-first in byte

            out[byte_index] |= 1u8 << bit_in_byte;
        }
    }

    Ok(out)
}
#[inline]
fn fastnum_from_big_endian_bytes_signed<const N: usize>(bytes_array: &[u8]) -> TonCoreResult<Int<N>> {
    let total_bits = (N * 64) as u32;
    assert_eq!(bytes_array.len(), N * 8);
    if total_bits == 0 {
        return Ok(Int::<N>::ZERO);
    }

    // Interpret bytes_array as a full-width two's-complement big-endian integer.
    // Bit 0 is LSB, bit total_bits-1 is the sign bit.
    let bits_len = total_bits;
    let sign_bit_pos = bits_len - 1;

    // Check if sign bit is set
    // The sign bit is at bit_pos = bits_len - 1, which maps to out_bit_index = 0 in the byte array
    // (because write function uses: out_bit_index = total_bits - 1 - bit_pos)
    let sign_out_bit_index = total_bits - 1 - sign_bit_pos; // This equals 0
    let sign_byte_index = (sign_out_bit_index / 8) as usize;
    let sign_bit_in_byte = 7 - (sign_out_bit_index % 8);
    let sign_bit_is_set = ((bytes_array[sign_byte_index] >> sign_bit_in_byte) & 1) != 0;

    // Build the value: if sign bit is set, start with -2^(n-1), otherwise start with 0
    let mut answer = if sign_bit_is_set {
        // Start with MIN value (-2^(n-1)) for two's complement
        Int::<N>::MIN
    } else {
        Int::<N>::ZERO
    };

    // Add all other bits (excluding the sign bit)
    // bit_pos: 0 = LSB, 1, 2, ..., bits_len-2 (exclude sign bit at bits_len-1)
    // Use reverse mapping to match the write function: out_bit_index = total_bits - 1 - bit_pos
    for bit_pos in 0..(bits_len - 1) {
        // Map bit_pos to the actual bit index in the byte array (reverse mapping)
        // bit_pos 0 → out_bit_index bits_len-1 (MSB position in array)
        // bit_pos bits_len-2 → out_bit_index 1 (bit before sign bit)
        let out_bit_index = total_bits - 1 - bit_pos;
        let byte_index = (out_bit_index / 8) as usize;
        let bit_in_byte = 7 - (out_bit_index % 8); // MSB-first inside the byte

        let byte = bytes_array[byte_index];
        let bit_is_one = ((byte >> bit_in_byte) & 1) != 0;

        if bit_is_one {
            let term = Int::<N>::ONE << bit_pos;
            answer += term;
        }
    }

    Ok(answer)
}

#[inline]
fn fastnum_to_big_endian_bytes_signed<const N: usize>(src: Int<N>) -> TonCoreResult<Vec<u8>> {
    let total_bits = (N * 64) as u32;
    let num_bytes = N * 8;
    let mut out = vec![0u8; num_bytes];
    for bit_pos in 0..total_bits {
        let mask = Int::<N>::ONE << bit_pos;
        let bit_is_one = (src & mask) != Int::<N>::ZERO;

        if bit_is_one {
            let out_bit_index = total_bits - 1 - bit_pos; // MSB-first
            let byte_index = (out_bit_index / 8) as usize;
            let bit_in_byte = 7 - (out_bit_index % 8); // MSB-first in byte
            out[byte_index] |= 1u8 << bit_in_byte;
        }
    }
    Ok(out)
}

macro_rules! ton_cell_num_fastnum_unsigned_impl {
    ($src:ty) => {
        impl TonCellNum for $src {
            fn tcn_write_bits(&self, writer: &mut CellBuilder, bits_len: u32) -> TonCoreResult<()> {
                if bits_len == 0 {
                    return Ok(());
                }
                assert!(bits_len <= Self::tcn_sizeof_bytes() * 8);
                if bits_len < self.tcn_min_bits_len() {
                    bail_ton_core_data!(
                        "Not enough bits for write num {} in {} bits unsigned, min len {}",
                        *self,
                        bits_len,
                        self.tcn_min_bits_len()
                    );
                }

                let bytes = fastnum_to_big_endian_bytes_unsigned(*self)?;

                toncellnum_bigendian_bit_writer(writer, &bytes, bits_len)?;

                Ok(())
            }

            fn tcn_read_bits(reader: &mut CellParser, bits_len: u32) -> TonCoreResult<Self> {
                if bits_len == 0 {
                    return Ok(Self::from(0u32));
                }

                let restored_forward =
                    toncellnum_bigendian_bit_reader(reader, bits_len, Self::tcn_sizeof_bytes() as u32, false)?;

                fastnum_from_big_endian_bytes_unsigned(&restored_forward)
            }

            fn tcn_is_zero(&self) -> bool { *self == Self::from(0u32) }

            fn tcn_min_bits_len(&self) -> u32 {
                if self.tcn_is_zero() {
                    0u32
                } else {
                    unsinged_highest_bit_pos!(*self, $src) as u32 + 1u32
                }
            }

            fn tcn_sizeof_bytes() -> u32 { (std::mem::size_of::<$src>()) as u32 }
        }
    };
}
macro_rules! ton_cell_num_fastnum_signed_impl {
    ($src:ty,$u_src:ty) => {
        impl TonCellNum for $src {
            fn tcn_write_bits(&self, writer: &mut CellBuilder, bits_len: u32) -> Result<(), TonCoreError> {
                if bits_len == 0 {
                    return Ok(());
                }
                assert!(bits_len <= Self::tcn_sizeof_bytes() * 8);
                if bits_len < self.tcn_min_bits_len() {
                    bail_ton_core_data!(
                        "Not enough bits for write num {} in {} bits unsigned, min len {}",
                        *self,
                        bits_len,
                        self.tcn_min_bits_len()
                    );
                }

                let bytes = fastnum_to_big_endian_bytes_signed(*self)?;
                toncellnum_bigendian_bit_writer(writer, &bytes, bits_len)?;
                Ok(())
            }
            fn tcn_read_bits(reader: &mut CellParser, bits_len: u32) -> Result<Self, TonCoreError> {
                let restored_forward =
                    toncellnum_bigendian_bit_reader(reader, bits_len, Self::tcn_sizeof_bytes() as u32, true)?;
                fastnum_from_big_endian_bytes_signed(&restored_forward)
            }

            fn tcn_is_zero(&self) -> bool { *self == Self::from(0u32) }

            fn tcn_min_bits_len(&self) -> u32 {
                if self.tcn_is_zero() {
                    0u32
                } else {
                    // Two's complement: same as primitives
                    let type_bits = (std::mem::size_of::<$src>() * 8) as u32;
                    // For MIN values, we need the full bit width
                    let self_val = *self;
                    if self_val == <$src>::MIN {
                        type_bits
                    } else {
                        fastnum_highest_bit_pos_signed!(self_val, $src) as u32 + 2u32
                    }
                }
            }

            fn tcn_sizeof_bytes() -> u32 { (std::mem::size_of::<$src>()) as u32 }
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

#[cfg(test)]
mod tests {
    use crate::cell::ton_cell_num::ton_cell_fastnum::{
        fastnum_from_big_endian_bytes_signed, fastnum_from_big_endian_bytes_unsigned,
        fastnum_to_big_endian_bytes_signed, fastnum_to_big_endian_bytes_unsigned,
    };

    use crate::cell::ton_cell_num::tests::test_num_read_write;
    use fastnum::*;

    #[test]
    fn test_toncellnum_conversation_bytes_array() {
        let val = -1i128;
        let fn_val = I128::from(val as i64);
        assert_eq!(fastnum_to_big_endian_bytes_signed(fn_val).unwrap(), val.to_be_bytes());

        let base = U128::from(U128::MAX);
        let bytes = fastnum_to_big_endian_bytes_unsigned(base).unwrap();
        let restored: U128 = fastnum_from_big_endian_bytes_unsigned(&bytes).unwrap();
        assert_eq!(base, restored);
        let base = U128::from(U128::MIN);
        let bytes = fastnum_to_big_endian_bytes_unsigned(base).unwrap();
        let restored: U128 = fastnum_from_big_endian_bytes_unsigned(&bytes).unwrap();
        assert_eq!(base, restored);

        let base = I128::from(I128::from(-1));
        let bytes = fastnum_to_big_endian_bytes_signed(base).unwrap();
        let restored: I128 = fastnum_from_big_endian_bytes_signed(&bytes).unwrap();
        assert_eq!(base, restored);
        let base = I128::from(I128::MIN);
        let bytes = fastnum_to_big_endian_bytes_signed(base).unwrap();
        let restored: I128 = fastnum_from_big_endian_bytes_signed(&bytes).unwrap();
        assert_eq!(base, restored);
        let base = I128::from(I128::MAX);
        let bytes = fastnum_to_big_endian_bytes_signed(base).unwrap();
        let restored: I128 = fastnum_from_big_endian_bytes_signed(&bytes).unwrap();
        assert_eq!(base, restored);
        let base = I128::from(I128::from(1));
        let bytes = fastnum_to_big_endian_bytes_signed(base).unwrap();
        let restored: I128 = fastnum_from_big_endian_bytes_signed(&bytes).unwrap();
        assert_eq!(base, restored);
    }

    #[test]
    fn test_toncellnum_fastnum_higest_bit_pos() -> anyhow::Result<()> {
        assert_eq!(std::mem::size_of::<I128>() as u32 * 8u32, fastnum_highest_bit_pos_signed!(I128::MIN, I128) + 2);
        assert_eq!(8u32, fastnum_highest_bit_pos_signed!(I128::from(-128), I128) + 2);
        Ok(())
    }

    #[test]
    fn test_toncellnum_fastnum_store_and_parse() -> anyhow::Result<()> {
        test_num_read_write(vec![(I128::from(-1234i64), 30)], "store_and_parse_I128").unwrap();
        test_num_read_write(vec![(I256::from(-1234i64), 30)], "store_and_parse_I256").unwrap();
        test_num_read_write(vec![(I512::from(-1234i64), 30)], "store_and_parse_I512").unwrap();
        test_num_read_write(vec![(I1024::from(-1234i64), 30)], "store_and_parse_I1024").unwrap();
        Ok(())
    }

    #[test]
    fn test_toncellnum_fastnum_lib_negations() -> () {
        // Fastnum now correctly handles negation
        let test_value1 = -I128::from(1234i64);
        let test_value2 = I128::from(-1234i64);
        assert_eq!(test_value1, test_value2);

        let test_value1 = -I256::from(1234i64);
        let test_value2 = I256::from(-1234i64);
        assert_eq!(test_value1, test_value2);
        // Fastnum now correctly handles negation
        let test_value1 = -I512::from(1234i64);
        let test_value2 = I512::from(-1234i64);
        assert_eq!(test_value1, test_value2);
        let test_value1 = -I1024::from(1234i64);
        let test_value2 = I1024::from(-1234i64);
        assert_eq!(test_value1, test_value2);
        ()
    }
    #[test]
    fn test_toncellnum_primitives_overbits_usage() {
        // primitive overbits unsigned
        test_num_read_write(vec![(U128::from(1u8), 256), (U128::MAX, 129)], "overbits_U128").unwrap();
        test_num_read_write(vec![(U256::from(1u8), 512), (U256::MAX, 257)], "overbits_U256").unwrap();
        test_num_read_write(vec![(U512::from(1u8), 1023), (U512::MAX, 513)], "overbits_U512").unwrap();

        // primitive overbits signed
        test_num_read_write(vec![(I128::from(1i8), 256), (I128::MAX, 129), (I128::MIN, 129)], "overbits_I128").unwrap();
        test_num_read_write(vec![(I256::from(1i8), 512), (I256::MAX, 257), (I256::MIN, 257)], "overbits_I256").unwrap();
        test_num_read_write(vec![(I512::from(1i8), 1023), (I512::MAX, 513), (I512::MIN, 513)], "overbits_I512")
            .unwrap();
    }

    #[test]
    fn test_toncellnum_fastnum_corner_cases() {
        // fastnum unsigned
        test_num_read_write(vec![(U128::from(0u8), 128), (U128::MAX, 128)], "U128").unwrap();
        test_num_read_write(vec![(U256::from(0u8), 256), (U256::MAX, 256)], "U256").unwrap();
        test_num_read_write(vec![(U512::from(0u8), 512), (U512::MAX, 512)], "U512").unwrap();
        let max_1023b = U1024::MAX >> 1;
        test_num_read_write(vec![(U1024::from(0u8), 1023), (max_1023b, 1023)], "U1024").unwrap();

        test_num_read_write(
            vec![
                (I128::from(0i8), 128),
                (I128::MAX, 128),
                (I128::MIN, 128),
                (I128::MIN / I128::from(2), 128),
            ],
            "I128",
        )
        .unwrap();
        test_num_read_write(
            vec![
                (I256::from(0i8), 256),
                (I256::MAX, 256),
                (I256::MIN, 256),
                (I256::MIN / I256::from(2), 256),
            ],
            "I256",
        )
        .unwrap();
        test_num_read_write(
            vec![
                (I512::from(0i8), 512),
                (I512::MAX, 512),
                (I512::MIN, 512),
                (I512::MIN / I512::from(2), 512),
                (I512::MIN / I512::from(2), 511),
            ],
            "I512",
        )
        .unwrap();
        let max_1023b: I1024 = I1024::MAX >> 1;
        let min_1023b = -max_1023b - I1024::from(1u8);
        test_num_read_write(vec![(I1024::from(0i8), 1023), (max_1023b, 1023), (min_1023b, 1023)], "I1024").unwrap();
    }
}
