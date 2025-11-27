use crate::cell::{CellBuilder, CellParser};

use std::fmt::Display;
mod ton_cell_bignum;
mod ton_cell_fastnum;
mod ton_cell_primitives;

use crate::bail_ton_core_data;
use crate::errors::TonCoreError;

/// Allows generic read/write operation for any numeric type
pub trait TonCellNum: Display + Sized + Clone {
    /// CellBuilder guarantees 0 < bits_len < 1024
    fn tcn_write_bits(&self, writer: &mut CellBuilder, bits_len: u32) -> Result<(), TonCoreError>;
    /// CellWriter guarantees 0 <= bits_len < 1024
    fn tcn_read_bits(reader: &mut CellParser, bits_len: u32) -> Result<Self, TonCoreError>;
    fn tcn_is_zero(&self) -> bool;
    fn tcn_min_bits_len(&self) -> u32;
}

    /// Returns the maximum bit size for padding purposes in write_num
    /// For fixed-size types, this is sizeof(T) * 8
    /// For BigInt/BigUint, this is 1024 (same as I1024/U1024)
    fn tcn_max_bits_len() -> u32;
}
macro_rules! primitive_convert_to_signed {
    ($uval:expr,$I:ty,$U:ty,$bit_count:expr) => {{
        // Two's complement decoding with sign extension
        let uval = $uval;
        let bit_count = $bit_count as usize;
        let type_bits = std::mem::size_of::<$I>() * 8;

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
            fn tcn_read_bits(reader: &mut CellBitsReader, bits_len: u32) -> Result<Self, TonCoreError> {
                if bits_len != 0 {
                    let val: Self = reader.read_var(bits_len)?;
                    Ok(val)
                } else {
                    Ok(0)
                }
            }
            fn tcn_is_zero(&self) -> bool { *self == 0 }

            fn tcn_read_bits(reader: &mut CellParser, bits_len: u32) -> Result<Self, TonCoreError> {
                let val_as = <$Dst>::tcn_read_bits(reader, bits_len)?;
                // fallible Dst -> Src, already returns TonCoreError
                $dst_to_src(val_as)
            }
        }
    };
}

            fn tcn_is_zero(&self) -> bool {
                // If conversion here can *theoretically* fail, treat it as a logic bug.
                let val_as: $Dst =
                    $src_to_dst(self).expect("toncellnum_use_type_as: Src -> Dst conversion failed in tcn_is_zero");
                val_as.tcn_is_zero()
            }

            fn tcn_read_bits(reader: &mut CellBitsReader, bits_len: u32) -> Result<Self, TonCoreError> {
                if bits_len != 0 {
                    let uval: $u_src = reader.read_var(bits_len)?;
                    let ret: Self = primitive_convert_to_signed!(uval, Self, $u_src, bits_len);
                    Ok(ret)
                } else {
                    Ok(0)
                }
            }

            fn tcn_is_zero(&self) -> bool { *self == 0 }

            fn tcn_min_bits_len(&self) -> u32 {
                let val_as: $Dst = $src_to_dst(self)
                    .expect("toncellnum_use_type_as: Src -> Dst conversion failed in tcn_min_bits_len");
                val_as.tcn_min_bits_len()
            }
        }
    };
}

            fn tcn_max_bits_len() -> u32 { <$Dst>::tcn_max_bits_len() }
        }
    };
    Ok(result)
}

impl TonCellNum for BigUint {
    fn tcn_write_bits(&self, writer: &mut CellBitWriter, bits_len: u32) -> Result<(), TonCoreError> {
        if bits_len == 0 {
            return Ok(());
        }
        let curr_u1024 = biguint_to_u1024(self)?;

        curr_u1024.tcn_write_bits(writer, bits_len)
    }

    fn tcn_read_bits(reader: &mut CellBitsReader, bits_len: u32) -> Result<Self, TonCoreError> {
        if bits_len == 0 {
            return Ok(BigUint::zero());
        }

        let val = U1024::tcn_read_bits(reader, bits_len)?;
        let result = u1024_to_biguint(val)?;

        Ok(result)
    }

    fn tcn_is_zero(&self) -> bool { Zero::is_zero(self) }

    fn tcn_min_bits_len(&self) -> u32 {
        if self.tcn_is_zero() {
            0u32
        } else {
            self.bits() as u32
        }
    }
}

impl TonCellNum for BigInt {
    fn tcn_write_bits(&self, writer: &mut CellBitWriter, bits_len: u32) -> Result<(), TonCoreError> {
        if bits_len == 0 {
            return Ok(());
        }
        bigint_to_i1024(self)?.tcn_write_bits(writer, bits_len)
    }

    fn tcn_read_bits(reader: &mut CellBitsReader, bits_len: u32) -> Result<Self, TonCoreError> {
        if bits_len == 0 {
            return Ok(BigInt::zero());
        }
        let val = I1024::tcn_read_bits(reader, bits_len)?;
        let result = i1024_to_bigint(val)?;

        Ok(result)
    }
    fn tcn_is_zero(&self) -> bool { *self == Self::from(0u32) }

    fn tcn_min_bits_len(&self) -> u32 {
        if self.tcn_is_zero() {
            0u32
        } else {
            (self.bits() as u32) + 1u32
        }
    }
}

use fastnum::bint::{Int, UInt};
// fastnum
fn fastnum_convert_to_unsigned<const N: usize>(src: Int<N>, bits_len: u32) -> Result<UInt<N>, TonCoreError> {
    // Sanity: Int/UInt<N> hold N*64 bits.
    assert!((bits_len as usize) <= N * 64, "bits_len exceeds width");

    // Special case: when bits_len == N*64, we can't compute 2^bits_len without overflow.
    // In this case, we handle the conversion differently.
    if bits_len == (N * 64) as u32 {
        if src < Int::<N>::ZERO {
            // For negative values: compute unsigned_value = signed_value + 2^(N*64)
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

pub fn fastnum_convert_to_signed<const N: usize>(src: UInt<N>, bits_len: u32) -> Result<Int<N>, TonCoreError> {
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
            // Negative value: compute signed = unsigned - 2^(N*64)
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

macro_rules! ton_cell_num_fastnum_unsigned_impl {
    ($src:ty) => {
        impl TonCellNum for $src {
            fn tcn_write_bits(&self, writer: &mut CellBitWriter, bits_len: u32) -> Result<(), TonCoreError> {
                if bits_len == 0 {
                    return Ok(());
                }
                let bits_len = bits_len as usize;
                if bits_len > size_of::<$src>() * 8 {
                    bail_ton_core_data!("Requested bits {} more than sizeof {}", bits_len, size_of::<$src>() * 8);
                }
                if bits_len < self.tcn_min_bits_len() as usize {
                    bail_ton_core_data!(
                        "Not enough bits for write num {} in {} bits unsigned, min len {}",
                        *self,
                        bits_len,
                        self.tcn_min_bits_len()
                    );
                }
                // Left-align value if not byte-aligned
                let mut value = *self;
                if bits_len % 8 != 0 {
                    value <<= 8 - bits_len % 8;
                }

                // Calculate number of bytes needed
                let num_bytes = bits_len.div_ceil(8);
                let full_bytes = bits_len / 8;
                let remaining_bits = bits_len % 8;

                // Extract bytes in big-endian order
                let mut bytes = Vec::with_capacity(num_bytes);
                for i in (0..num_bytes).rev() {
                    let shift_amount = i * 8;
                    let byte_val = (value >> shift_amount) & Self::from(0xFFu32);

                    // Convert to u8 by going through u64
                    // For a single byte value, this is safe
                    bytes.push(byte_val.to_u64().unwrap() as u8);
                }

                // Write full bytes
                writer.write_bytes(&bytes[0..full_bytes])?;

                // Write remaining bits from TOP of last byte
                if remaining_bits > 0 {
                    let last_byte = bytes[full_bytes];
                    writer.write_var(remaining_bits as u32, last_byte >> (8 - remaining_bits))?;
                }
                Ok(())
            }

            fn tcn_read_bits(reader: &mut CellBitsReader, bits_len: u32) -> Result<Self, TonCoreError> {
                if bits_len == 0 {
                    return Ok(Self::from(0u32));
                }

                let full_bytes = bits_len / 8;
                let remaining_bits = bits_len % 8;
                let mut result = Self::from(0u32);

                // Read full bytes
                for _ in 0..full_bytes {
                    let byte = reader.read::<8, u8>()?;
                    result = (result << 8) | Self::from(byte);
                }

                // Read remaining bits if any
                if remaining_bits > 0 {
                    let last_bits = reader.read_var::<u8>(remaining_bits as u32)?;
                    result = (result << remaining_bits) | Self::from(last_bits);
                }

                Ok(result)
            }

            fn tcn_is_zero(&self) -> bool { *self == Self::from(0u32) }

            fn tcn_min_bits_len(&self) -> u32 {
                if self.tcn_is_zero() {
                    0u32
                } else {
                    primitive_highest_bit_pos!(*self, $src, false) as u32 + 1u32
                }
            }
        }
    };
}
macro_rules! ton_cell_num_fastnum_signed_impl {
    ($src:ty,$u_src:ty) => {
        impl TonCellNum for $src {
            fn tcn_read_bits(reader: &mut CellBitsReader, bits_len: u32) -> Result<Self, TonCoreError> {
                let u_sibling = <$u_src>::tcn_read_bits(reader, bits_len)?;
                let rz = fastnum_convert_to_signed(u_sibling, bits_len)?;
                Ok(rz)
            }
            fn tcn_write_bits(&self, writer: &mut CellBitWriter, bits_len: u32) -> Result<(), TonCoreError> {
                let u_sibling: $u_src = fastnum_convert_to_unsigned(*self, bits_len)?;
                u_sibling.tcn_write_bits(writer, bits_len)
            }

            fn tcn_is_zero(&self) -> bool { *self == Self::from(0u32) }

            fn tcn_min_bits_len(&self) -> u32 {
                if self.tcn_is_zero() {
                    0u32
                } else {
                    // Two's complement: same as primitives

                    primitive_highest_bit_pos!(*self, $src, true) as u32 + 2u32
                }
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

#[cfg(test)]
mod tests {

    use crate::cell::TonCellNum;
    use crate::cell::{CellParser, TonCell};
    use fastnum::*;
    use num_bigint::{BigInt, BigUint};
    use std::fmt::Debug;
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
    fn test_toncellnum_store_and_parse_int16() -> anyhow::Result<()> {
        // Create a builder and store an int16 value

        let mut builder = TonCell::builder();
        let test_value: i16 = -12;

        let test_bit = 5;
        builder.write_num(&test_value, test_bit)?;

        // Build the cell
        let cell = builder.build()?;

        // Create a parser and read back the int16 value
        let mut parser = CellParser::new(&cell);
        let parsed_value = parser.read_num::<i16>(test_bit)?;

        // Verify the value matches
        assert_eq!(parsed_value, test_value);

        Ok(())
    }
    #[test]
    fn test_toncellnum_store_and_parse_bigint() -> anyhow::Result<()> {
        // Create a builder and store an int16 value
        let mut builder = TonCell::builder();
        let test_value = BigInt::from(-900i128);

        let test_bit = 14;
        builder.write_num(&test_value, test_bit)?;

        // Build the cell
        let cell = builder.build()?;

        // Create a parser and read back the int16 value
        let mut parser = CellParser::new(&cell);
        let parsed_value = parser.read_num::<BigInt>(test_bit)?;

        // Verify the value matches
        assert_eq!(parsed_value, test_value);

        Ok(())
    }
    #[test]
    fn test_toncellnum_store_and_parse_biguint() -> anyhow::Result<()> {
        // Create a builder and store an int16 value
        let mut builder = TonCell::builder();
        let test_value: BigUint = BigUint::from(64000u64);

        let test_bit = 32;
        builder.write_num(&test_value, test_bit)?;

        // Build the cell
        let cell = builder.build()?;

        // Create a parser and read back the int16 value
        let mut parser = CellParser::new(&cell);
        let parsed_value = parser.read_num::<BigUint>(test_bit)?;

        // Verify the value matches
        assert_eq!(parsed_value, test_value);

        Ok(())
    }

    #[test]
    fn test_toncellnum_store_and_parse_usize() -> anyhow::Result<()> {
        // Create a builder and store a usize value
        let mut builder = TonCell::builder();
        let test_value: usize = 12345;

        let test_bit = 32;
        builder.write_num(&test_value, test_bit)?;

        // Build the cell
        let cell = builder.build()?;

        // Create a parser and read back the usize value
        let mut parser = CellParser::new(&cell);
        let parsed_value = parser.read_num::<usize>(test_bit)?;

        // Verify the value matches
        assert_eq!(parsed_value, test_value);

        Ok(())
    }

    #[test]
    fn test_toncellnum_store_and_parse_u512() -> anyhow::Result<()> {
        // Create a builder and store a usize value
        let mut builder = TonCell::builder();
        let test_value: U512 = 1234u64.into();

        let test_bit = 30;
        builder.write_num(&test_value, test_bit)?;

        // Build the cell
        let cell = builder.build()?;

        // Create a parser and read back the usize value
        let mut parser = CellParser::new(&cell);
        let parsed_value = parser.read_num::<U512>(test_bit)?;

        // Verify the value matches
        assert_eq!(parsed_value, test_value);

        Ok(())
    }

    #[test]
    fn test_toncellnum_fastnum_i512_negation() -> () {
        // Fastnum now correctly handles negation
        let test_value1 = -I512::from(1234i64);
        let test_value2 = I512::from(-1234i64);
        assert_eq!(test_value1, test_value2);
        ()
    }

    #[test]
    fn test_toncellnum_fastnum_i256_negation() -> () {
        // Fastnum now correctly handles negation
        let test_value1 = -I256::from(1234i64);
        let test_value2 = I256::from(-1234i64);
        assert_eq!(test_value1, test_value2);
        ()
    }

    #[test]
    fn test_toncellnum_fastnum_i128_negation() -> () {
        // Fastnum now correctly handles negation
        let test_value1 = -I128::from(1234i64);
        let test_value2 = I128::from(-1234i64);
        assert_eq!(test_value1, test_value2);
        ()
    }

    #[test]
    fn test_toncellnum_store_and_parse_i512() -> anyhow::Result<()> {
        // Create a builder and store a I512 value
        let mut builder = TonCell::builder();
        let test_value = -I512::from(1234i32);

        let test_bit = 30;
        builder.write_num(&test_value, test_bit)?;

        // Build the cell
        let cell = builder.build()?;

        // Create a parser and read back the I512 value
        let mut parser = CellParser::new(&cell);
        let parsed_value = parser.read_num::<I512>(test_bit)?;

        // Verify the value matches
        assert_eq!(parsed_value, test_value);

        Ok(())
    }
    #[test]
    fn test_toncellnum_bigint_toi1024_conv() {
        let test_big_int = -1 * BigInt::from(1234i64);
        let test_fastnum = bigint_to_i1024(&test_big_int).unwrap();
        let result_big_int = i1024_to_bigint(test_fastnum).unwrap();

        assert_eq!(test_big_int, result_big_int);
    }

    #[test]
    fn test_toncellnum_biguint_tou1024_conv() {
        // Since u1024_to_biguint and biguint_to_u1024 are private functions
        // in the same module, we can call them directly without importing

        // Test with a simple value
        let test_big_uint = BigUint::from(1234u64);
        let test_fastnum = biguint_to_u1024(&test_big_uint).unwrap();
        let result_big_uint = u1024_to_biguint(test_fastnum).unwrap();
        assert_eq!(test_big_uint, result_big_uint);

        // Test with zero
        let test_big_uint = BigUint::from(0u32);
        let test_fastnum = biguint_to_u1024(&test_big_uint).unwrap();
        let result_big_uint = u1024_to_biguint(test_fastnum).unwrap();
        assert_eq!(test_big_uint, result_big_uint);

        // Test with a large value
        let test_big_uint = BigUint::from(u128::MAX);
        let test_fastnum = biguint_to_u1024(&test_big_uint).unwrap();
        let result_big_uint = u1024_to_biguint(test_fastnum).unwrap();
        assert_eq!(test_big_uint, result_big_uint);

        // Test with a very large value (256 bits)
        let test_big_uint = (BigUint::from(1u32) << 255) + BigUint::from(12345u64);
        let test_fastnum = biguint_to_u1024(&test_big_uint).unwrap();
        let result_big_uint = u1024_to_biguint(test_fastnum).unwrap();
        assert_eq!(test_big_uint, result_big_uint);
    }

    #[test]
    fn test_toncellnum_bigint_257_bits_serialization() -> anyhow::Result<()> {
        // This test demonstrates the BigInt serialization issue with non-byte-aligned sizes
        // BigInt uses sign encoding: (magnitude << 1) | sign_bit
        // For 257 bits (33 bytes with 1 bit in last byte), the alignment matters

        use num_bigint::BigInt;
        use std::str::FromStr;

        let mut builder = TonCell::builder();

        // This is the actual value from test_get_nft_data_result
        let test_value =
            BigInt::from_str("17026683442852985036293000817890672620529067535828542797724775561309021470835")?;

        // BigInt in TVM stack uses 257 bits (int257)
        let bits_len = 257;
        builder.write_num(&test_value, bits_len)?;

        let cell = builder.build()?;

        // Read back
        let mut parser = CellParser::new(&cell);
        let parsed_value = parser.read_num::<BigInt>(bits_len)?;

        // This should pass but currently fails due to alignment issues
        assert_eq!(parsed_value, test_value, "BigInt round-trip failed for 257 bits");

        Ok(())
    }

    #[test]
    fn test_toncellnum_bigint_simple_non_byte_aligned() -> anyhow::Result<()> {
        // Simpler test: 9 bits (2 bytes with 1 bit in second byte)
        use num_bigint::BigInt;

        let mut builder = TonCell::builder();
        let test_value = BigInt::from(42);

        let bits_len = 9; // Not byte-aligned

        builder.write_num(&test_value, bits_len)?;

        let cell = builder.build()?;

        let mut parser = CellParser::new(&cell);

        // Debug: check what bytes we read back
        let _ = parser.read_bits(bits_len)?;
        parser.seek_bits(-(bits_len as i32))?;

        let parsed_value = parser.read_num::<BigInt>(bits_len)?;

        assert_eq!(parsed_value, test_value, "BigInt round-trip failed for 9 bits");

        Ok(())
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
        let parsed_value = parser.read_num::<BigUint>(bits_len)?;

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

        let mut parser = CellParser::new(&cell);
        parser.read_bits(bits_len)?;
        parser.seek_bits(-(bits_len as i32))?;
        let parsed_value = parser.read_num::<U256>(bits_len)?;

        assert_eq!(parsed_value, test_value, "U256 round-trip failed for 9 bits");

        Ok(())
    }

    #[test]
    fn test_toncellnum_i128_zero_value_zero_bits() -> anyhow::Result<()> {
        // Test I128 with zero value and 0 bits (edge case)
        use fastnum::I128;

        let mut builder = TonCell::builder();
        let test_value = I128::from(0u32);

        let bits_len = 0; // Zero bits

        builder.write_num(&test_value, bits_len)?;
        let cell = builder.build()?;

        let mut parser = CellParser::new(&cell);
        let parsed_value = parser.read_num::<I128>(bits_len)?;

        assert_eq!(parsed_value, test_value, "I128 round-trip failed for 0 bits with zero value");

        Ok(())
    }

    #[test]
    fn test_ton_cell_num_fastnum_i256_negative_256_bits() -> anyhow::Result<()> {
        let mut builder = TonCell::builder();
        let test_value = I256::from(-32);
        let bits_len = 256;

        builder.write_num(&test_value, bits_len)?;
        let cell = builder.build()?;

        let mut parser = CellParser::new(&cell);
        let parsed_value = parser.read_num::<I256>(bits_len)?;

        assert_eq!(parsed_value, test_value);
        Ok(())
    }
}
