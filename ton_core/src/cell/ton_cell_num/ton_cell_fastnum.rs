use crate::bail_ton_core_data;
use crate::cell::TonCellNum;
use crate::cell::{CellBuilder, CellParser};
use crate::errors::{TonCoreError, TonCoreResult};
use crate::unsinged_highest_bit_pos;
use fastnum::bint::{Int, UInt};
use fastnum::{TryCast, I1024, I128, I256, I512};
use fastnum::{U1024, U128, U256, U512};

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

fn fastnum_from_big_endian_bytes<const N: usize>(bytes_array: &[u8], bits_len: u32) -> TonCoreResult<UInt<N>> {
    let total_bits = (N * 64) as u32;

    if bits_len > total_bits {
        return Err(TonCoreError::DataError {
            producer: "fastnum_from_big_endian_bytes".to_string(),
            msg: format!("bits_len {} exceeds maximum {} for UInt<{}>", bits_len, total_bits, N),
        });
    }
    let available_bits = (bytes_array.len() * 8) as u32;
    if bits_len > available_bits {
        return Err(TonCoreError::DataError {
            producer: "fastnum_from_big_endian_bytes".to_string(),
            msg: format!(
                "bits_len {} exceeds available bits {} ({} bytes)",
                bits_len,
                available_bits,
                bytes_array.len()
            ),
        });
    }
    if bits_len == 0 {
        return Ok(UInt::<N>::ZERO);
    }
    let mut answer = UInt::<N>::ZERO;
    for bit_pos in 0..bits_len {
        let out_bit_index = bits_len - 1 - bit_pos; // 0..bits_len-1
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

pub(crate) fn fastnum_to_big_endian_bytes<const N: usize>(src: UInt<N>, bits_len: u32) -> TonCoreResult<Vec<u8>> {
    let total_bits = (N * 64) as u32;
    if bits_len > total_bits {
        return Err(TonCoreError::DataError {
            producer: "fastnum_to_big_endian_bytes".to_string(),
            msg: format!("bits_len {} exceeds maximum {} for UInt<{}>", bits_len, total_bits, N),
        });
    }

    if bits_len == 0 {
        return Ok(Vec::new());
    }

    // how many bytes we need to store bits_len bits
    let num_bytes = bits_len.div_ceil(8) as usize;
    let mut out = vec![0u8; num_bytes];

    // We treat bit 0 as the least-significant bit of src.
    // We want a big-endian bitstring:
    // - bit (bits_len - 1) -> MSB of out[0]
    // - bit 0              -> LSB of out[last]
    for bit_pos in 0..bits_len {
        let mask = UInt::<N>::ONE << bit_pos;
        let bit_is_one = (src & mask) != UInt::<N>::ZERO;

        if bit_is_one {
            let out_bit_index = bits_len - 1 - bit_pos; // 0..bits_len-1
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
                assert!(bits_len <= Self::tcn_max_bits_len());

                if bits_len < self.tcn_min_bits_len() {
                    bail_ton_core_data!(
                        "Not enough bits for write num {} in {} bits unsigned, min len {}",
                        *self,
                        bits_len,
                        self.tcn_min_bits_len()
                    );
                }

                let bytes = fastnum_to_big_endian_bytes(*self, bits_len)?;
                writer.write_bits(&bytes, bits_len as usize)?;

                Ok(())
            }

            fn tcn_read_bits(reader: &mut CellParser, bits_len: u32) -> TonCoreResult<Self> {
                if bits_len == 0 {
                    return Ok(Self::from(0u32));
                }
                let bits_array = reader.read_bits(bits_len as usize)?;
                fastnum_from_big_endian_bytes(&bits_array, bits_len)
            }

            fn tcn_is_zero(&self) -> bool { *self == Self::from(0u32) }

            fn tcn_min_bits_len(&self) -> u32 {
                if self.tcn_is_zero() {
                    0u32
                } else {
                    unsinged_highest_bit_pos!(*self, $src) as u32 + 1u32
                }
            }

            fn tcn_max_bits_len() -> u32 { (std::mem::size_of::<$src>() * 8) as u32 }
        }
    };
}
macro_rules! ton_cell_num_fastnum_signed_impl {
    ($src:ty,$u_src:ty) => {
        impl TonCellNum for $src {
            fn tcn_read_bits(reader: &mut CellParser, bits_len: u32) -> Result<Self, TonCoreError> {
                let u_sibling = <$u_src>::tcn_read_bits(reader, bits_len)?;
                let rz = fastnum_convert_to_signed(u_sibling, bits_len)?;
                Ok(rz)
            }
            fn tcn_write_bits(&self, writer: &mut CellBuilder, bits_len: u32) -> Result<(), TonCoreError> {
                let u_sibling: $u_src = fastnum_convert_to_unsigned(*self, bits_len)?;
                u_sibling.tcn_write_bits(writer, bits_len)
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

            fn tcn_max_bits_len() -> u32 { (std::mem::size_of::<$src>() * 8) as u32 }
        }
    };
}

// fastnum
fn fastnum_convert_to_unsigned<const N: usize>(src: Int<N>, bits_len: u32) -> Result<UInt<N>, TonCoreError> {
    if bits_len > (N * 64) as u32 {
        bail_ton_core_data!("bits_len exceeds width");
    }
    if bits_len == (N * 64) as u32 {
        if src < Int::<N>::ZERO {
            // Handle MIN values specially to avoid overflow in negation
            if src == Int::<N>::MIN {
                // For MIN, the unsigned representation is the sign bit set: 1 << (N*64 - 1)
                let sign_bit_pos = (N * 64 - 1) as u32;
                return Ok(UInt::<N>::ONE << sign_bit_pos);
            }
            // For other negative values: compute unsigned_value = signed_value + 2^(N*64)
            // Since 2^(N*64) can't be represented in UInt<N>, we work around it by
            // using the fact that UInt<N>::MAX + 1 wraps to 0, so we can compute
            // the complement. For a negative value n, the unsigned representation is
            // -n as UInt, but in two's complement it's UInt::MAX - |n| + 1.

            // Get the absolute value as an unsigned type
            let abs_val: UInt<N> = (-src)
                .try_cast()
                .map_err(|_| TonCoreError::data("fastnum_convert_to_unsigned", "abs conversion failed"))?;

            // Compute two's complement: UInt::MAX - abs + 1 = -abs (in two's complement)
            return Ok(UInt::<N>::MAX - abs_val + UInt::<N>::ONE);
        } else {
            // For non-negative values, direct cast works
            let u_value: UInt<N> = src
                .try_cast()
                .map_err(|_| TonCoreError::data("fastnum_convert_to_unsigned", "full-width conversion failed"))?;
            return Ok(u_value);
        }
    }
    // Special case: when bits_len == N*64 - 1, 2^bits_len might not fit in Int<N>
    // For example, 2^1023 doesn't fit in Int<16> (I1024)
    if bits_len == (N * 64 - 1) as u32 {
        // For (N*64 - 1) bits, the modulus is 2^(N*64 - 1) = sign_bit << 1
        // where sign_bit = 2^(N*64 - 2)
        let sign_bit_pos = (N * 64 - 2) as u32;
        let sign_bit = UInt::<N>::ONE << sign_bit_pos;
        let modulus = sign_bit << 1; // This is 2^(N*64 - 1)

        if src < Int::<N>::ZERO {
            // For negative values: unsigned = signed + 2^(N*64 - 1) = modulus - |src|
            if src == Int::<N>::MIN {
                // MIN can't be negated, but for (N*64 - 1) bits, the most negative value
                // is -2^(N*64 - 2), which is represented as sign_bit in unsigned
                return Ok(sign_bit);
            }
            // For other negatives: unsigned = modulus - |src|
            let abs_val: UInt<N> = (-src).try_cast().map_err(|_| {
                TonCoreError::data("fastnum_convert_to_unsigned", "abs conversion failed for near-full-width")
            })?;
            return Ok(modulus - abs_val);
        } else {
            // For non-negative values, direct cast works (they're already < 2^(N*64 - 1))
            let u_value: UInt<N> = src
                .try_cast()
                .map_err(|_| TonCoreError::data("fastnum_convert_to_unsigned", "near-full-width conversion failed"))?;
            return Ok(u_value);
        }
    }

    // 2^bits_len as UInt<N>
    let modulus_u = UInt::<N>::ONE << bits_len;

    // Cast modulus to Int<N> so we can use rem_euclid on the signed value.
    let modulus_i: Int<N> = modulus_u
        .try_cast()
        .map_err(|_| TonCoreError::data("fastnum_convert_to_unsigned", "2^bits_len fits into Int<N>"))?;

    // (a mod m) in the mathematical sense (always >= 0), even for negatives.
    let reduced_i = src.rem_euclid(modulus_i);

    // Cast the non-negative remainder back to UInt<N>.
    reduced_i
        .try_cast()
        .map_err(|_| TonCoreError::data("fastnum_convert_to_unsigned", "non-negative remainder fits into UInt<N>"))
}

fn fastnum_convert_to_signed<const N: usize>(src: UInt<N>, bits_len: u32) -> Result<Int<N>, TonCoreError> {
    if bits_len > (N * 64) as u32 {
        bail_ton_core_data!("bits_len exceeds width");
    }
    // Special-case 0 bits: by convention return 0.
    if bits_len == 0 {
        return Ok(Int::<N>::from(0u8));
    }

    // Special case: when bits_len == N*64, we can't compute 2^bits_len without overflow.
    if bits_len == (N * 64) as u32 {
        // For full-width values, the unsigned value IS the bit pattern.
        // To convert to signed two's complement: signed = unsigned - 2^(N*64)
        // Since 2^(N*64) can't be represented in UInt<N>, we use: signed = unsigned - (MAX + 1)
        // Where MAX + 1 wraps to 0, so signed = unsigned - MAX - 1
        // Simplifying: signed = -(MAX - unsigned + 1)

        // Check if this represents a negative value (sign bit set)
        let sign_bit_pos = (N * 64 - 1) as u32;
        let sign_bit = UInt::<N>::ONE << sign_bit_pos;

        if src >= sign_bit {
            // Handle MIN value specially (sign bit is the only bit set)
            if src == sign_bit {
                return Ok(Int::<N>::MIN);
            }
            // Other negative values: compute signed = unsigned - 2^(N*64)
            // Since 2^(N*64) = UInt::MAX + 1, and MAX + 1 wraps to 0:
            // signed = unsigned - (MAX + 1) = unsigned + (!MAX) = unsigned - MAX - 1
            let diff = UInt::<N>::MAX - src;
            let i_value = (diff + UInt::<N>::ONE).try_cast().map_err(|_| {
                TonCoreError::data("fastnum_convert_to_signed", "Failed to convert negative full-width value")
            })?;
            return Ok(-i_value);
        } else {
            // Positive value: direct cast works
            let i_value: Int<N> = src.try_cast().map_err(|_| {
                TonCoreError::data("fastnum_convert_to_signed", "Failed to convert positive full-width value")
            })?;
            return Ok(i_value);
        }
    }

    // Special case: when bits_len == N*64 - 1, 2^bits_len might not fit in Int<N>

    if bits_len == (N * 64 - 1) as u32 {
        let sign_bit_pos = (N * 64 - 2) as u32;
        let sign_bit = UInt::<N>::ONE << sign_bit_pos;
        let modulus = sign_bit << 1;

        // Check if this represents a negative value (sign bit at position N*64 - 2 is set)
        if src >= sign_bit {
            // Negative value: signed = unsigned - 2^(N*64 - 1) = unsigned - modulus
            let diff: UInt<N> = modulus - src;
            let i_value: Int<N> = diff.try_cast().map_err(|_| {
                TonCoreError::data("fastnum_convert_to_signed", "Failed to convert negative near-full-width value")
            })?;
            return Ok(-i_value);
        } else {
            // Positive value: direct cast works
            let i_value: Int<N> = src.try_cast().map_err(|_| {
                TonCoreError::data("fastnum_convert_to_signed", "Failed to convert positive near-full-width value")
            })?;
            return Ok(i_value);
        }
    }

    // 2^bits_len
    let two_pow_bits_u = UInt::<N>::ONE << bits_len;
    let two_pow_bits_i: Int<N> = two_pow_bits_u
        .try_cast()
        .map_err(|_| TonCoreError::data("fastnum_convert_to_signed", "Failed to cast 2^bits_len to Int"))?;

    // Mask to exactly `bits_len` low bits (in case higher bits are set)
    let mask = two_pow_bits_u - UInt::<N>::ONE;
    let v = src & mask;

    // Sign bit (2^(bits_len-1)): if set -> negative branch
    let sign_bit = UInt::<N>::ONE << (bits_len - 1);

    // Cast the (masked) magnitude into Int
    let mut as_i: Int<N> = v
        .try_cast()
        .map_err(|_| TonCoreError::data("fastnum_convert_to_signed", "Failed to cast masked value to Int"))?;

    if v >= sign_bit {
        // Negative value: subtract 2^bits_len to get the proper two's-complement Int
        as_i -= two_pow_bits_i;
    }

    Ok(as_i)
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
    use crate::cell::ton_cell_num::tests::test_num_read_write;
    use crate::cell::TonCell;
    use fastnum::*;

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
    fn test_toncellnum_fastnum_custom() {
        let mut builder = TonCell::builder();
        let val = U512::MAX;
        builder.write_num(&val, 512).unwrap();
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
