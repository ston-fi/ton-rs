use crate::cell::{CellBitReader, CellBitWriter};
use bitstream_io::{BitRead, BitWrite};
use num_bigint::{BigInt, BigUint};
use num_traits::{One, Signed, Zero};
use std::fmt::Display;

use crate::bail_ton_core_data;
use crate::errors::TonCoreError;
use fastnum::{I1024, I128, I256, I512};
use fastnum::{U1024, U128, U256, U512};

macro_rules! primitive_convert_to_unsigned {
    ($val:expr,$T:ty) => {{
        let mut uval: $T = $val.unsigned_abs();
        uval <<= 1u8;
        if $val < 0 {
            uval += <$T>::one();
        }
        uval
    }};
}
macro_rules! primitive_convert_to_signed {
    ($uval:expr,$I:ty) => {{
        let mut val: $I = ($uval >> 1) as $I;
        if ($uval & 1) != 0 {
            val *= -1 as $I;
        }
        val
    }};
}

macro_rules! primitive_highest_bit_pos {
    ($val:expr,$T:ty,true) => {{
        let max_bit_id = (std::mem::size_of::<$T>() * 8 - 1) as u32;
        (max_bit_id - $val.abs().leading_zeros())
    }};
    ($val:expr,$T:ty,false) => {{
        let max_bit_id = (std::mem::size_of::<$T>() * 8 - 1) as u32;
        (max_bit_id - $val.leading_zeros())
    }};
}

/// Allows generic read/write operation for any numeric type
///
/// Questions
/// Split on Primitive and not Primitive?
pub trait TonCellNum: Display + Sized + Clone {
    fn tcn_write_bits(&self, writer: &mut CellBitWriter, bits_len: u32) -> Result<(), TonCoreError>;

    fn tcn_read_bits(reader: &mut CellBitReader, bits_len: u32) -> Result<Self, TonCoreError>;

    fn tcn_is_zero(&self) -> bool;

    fn tcn_shr(&self, bits: usize) -> Self;

    fn tcn_min_bits_len(&self) -> u32;
}
macro_rules! ton_cell_num_primitive_unsigned_impl {
    ($src:ty) => {
        impl TonCellNum for $src {
            fn tcn_write_bits(&self, writer: &mut CellBitWriter, bits_len: u32) -> Result<(), TonCoreError> {
                if self.tcn_min_bits_len() > bits_len {
                    bail_ton_core_data!(
                        "Not enough bits for write num {} in {} bits signed, min len {}",
                        *self,
                        bits_len,
                        self.tcn_min_bits_len()
                    );
                }
                writer.write_var(bits_len, *self)?;
                Ok(())
            }
            fn tcn_read_bits(reader: &mut CellBitReader, bits_len: u32) -> Result<Self, TonCoreError> {
                if (bits_len != 0) {
                    let val: Self = reader.read_var(bits_len)?;
                    Ok(val)
                } else {
                    Ok(0)
                }
            }
            fn tcn_is_zero(&self) -> bool { *self == 0 }
            fn tcn_shr(&self, _bits: usize) -> Self { *self >> _bits }
            fn tcn_min_bits_len(&self) -> u32 {
                if *self == 0 {
                    0u32
                } else {
                    (primitive_highest_bit_pos!(*self, Self, false) + 1u32)
                }
            }
        }
    };
}

macro_rules! ton_cell_num_primitive_signed_impl {
    ($src:ty,$u_src:ty) => {
        impl TonCellNum for $src {
            fn tcn_write_bits(&self, writer: &mut CellBitWriter, bits_len: u32) -> Result<(), TonCoreError> {
                let val: $u_src = primitive_convert_to_unsigned!(*self, $u_src);
                writer.write_var(bits_len, val)?;
                Ok(())
            }

            fn tcn_read_bits(reader: &mut CellBitReader, bits_len: u32) -> Result<Self, TonCoreError> {
                if bits_len != 0 {
                    let uval: $u_src = reader.read_var(bits_len)?;
                    let ret: Self = primitive_convert_to_signed!(uval, Self);
                    Ok(ret)
                } else {
                    Ok(0)
                }
            }

            fn tcn_is_zero(&self) -> bool { *self == 0 }
            fn tcn_shr(&self, _bits: usize) -> Self { *self >> _bits }
            fn tcn_min_bits_len(&self) -> u32 {
                if *self == 0 {
                    0u32
                } else {
                    primitive_highest_bit_pos!(*self, Self, true) + 2u32
                }
            }
        }
    };
}

ton_cell_num_primitive_unsigned_impl!(u8);
ton_cell_num_primitive_unsigned_impl!(u16);
ton_cell_num_primitive_unsigned_impl!(u32);
ton_cell_num_primitive_unsigned_impl!(u64);
ton_cell_num_primitive_unsigned_impl!(u128);

ton_cell_num_primitive_signed_impl!(i16, u16);
ton_cell_num_primitive_signed_impl!(i32, u32);
ton_cell_num_primitive_signed_impl!(i64, u64);
ton_cell_num_primitive_signed_impl!(i128, u128);
ton_cell_num_primitive_signed_impl!(i8, u8);

// Implementation for BigUint
// Note: BigUint is used for BigInt sign encoding
// Must left-align values for non-byte-aligned sizes to match write_bits expectations
impl TonCellNum for usize {
    fn tcn_write_bits(&self, writer: &mut CellBitWriter, bits_len: u32) -> Result<(), TonCoreError> {
        (*self as u64).tcn_write_bits(writer, bits_len)
    }

    fn tcn_read_bits(reader: &mut CellBitReader, bits_len: u32) -> Result<Self, TonCoreError> {
        let val: u64 = u64::tcn_read_bits(reader, bits_len)?;
        Ok(val as usize)
    }
    fn tcn_is_zero(&self) -> bool { *self == 0 }
    fn tcn_shr(&self, bits: usize) -> Self { *self >> bits }

    fn tcn_min_bits_len(&self) -> u32 {
        if *self == 0 {
            0u32
        } else {
            (primitive_highest_bit_pos!(*self, Self, false) + 1u32)
        }
    }
}

impl TonCellNum for BigUint {
    fn tcn_write_bits(&self, writer: &mut CellBitWriter, bits_len: u32) -> Result<(), TonCoreError> {
        if bits_len == 0 {
            return Ok(());
        }

        // Left-align the value if not byte-aligned
        let value_to_write = if bits_len % 8 != 0 {
            self << (8 - bits_len % 8)
        } else {
            self.clone()
        };

        // Get big-endian bytes
        let mut bytes = value_to_write.to_bytes_be();

        let num_bytes = bits_len.div_ceil(8) as usize;
        let full_bytes = (bits_len / 8) as usize;
        let remaining_bits = bits_len % 8;

        // Pad with leading zeros if needed
        while bytes.len() < num_bytes as usize {
            bytes.insert(0, 0);
        }

        // Trim if we have extra bytes
        if bytes.len() > num_bytes {
            bytes = bytes[(bytes.len() - num_bytes)..].to_vec();
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

    fn tcn_read_bits(reader: &mut CellBitReader, bits_len: u32) -> Result<Self, TonCoreError> {
        if bits_len == 0 {
            return Ok(BigUint::zero());
        }

        let full_bytes = bits_len / 8;
        let remaining_bits = bits_len % 8;
        let mut result = BigUint::zero();

        // Read full bytes
        for _ in 0..full_bytes {
            let byte = reader.read::<8, u8>()?;
            result = (result << 8) | BigUint::from(byte);
        }

        // Read remaining bits if any
        if remaining_bits > 0 {
            let last_bits = reader.read_var::<u8>(remaining_bits as u32)?;
            result = (result << remaining_bits) | BigUint::from(last_bits);
        }

        Ok(result)
    }

    fn tcn_is_zero(&self) -> bool { Zero::is_zero(self) }

    fn tcn_shr(&self, bits: usize) -> Self { self >> bits }

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

        // Use two's complement encoding (standard for TVM int257 and signed integers)
        let mut bytes = self.to_signed_bytes_be();

        // Calculate required bytes
        let num_bytes = bits_len.div_ceil(8) as usize;
        let full_bytes = (bits_len / 8) as usize;
        let remaining_bits = bits_len % 8;

        // Pad with sign extension if needed
        let pad_byte = if self.is_negative() { 0xFF } else { 0x00 };
        while bytes.len() < num_bytes {
            bytes.insert(0, pad_byte);
        }

        // Trim if too many bytes
        if bytes.len() > num_bytes {
            bytes = bytes[(bytes.len() - num_bytes)..].to_vec();
        }

        // Left-align if not byte-aligned
        if bits_len % 8 != 0 {
            let shift = 8 - (bits_len % 8);
            let mut carry = 0u16;
            for i in (0..bytes.len()).rev() {
                let val = (bytes[i] as u16) << shift;
                bytes[i] = (val | carry) as u8;
                carry = val >> 8;
            }
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

    fn tcn_read_bits(reader: &mut CellBitReader, bits_len: u32) -> Result<Self, TonCoreError> {
        if bits_len == 0 {
            return Ok(BigInt::zero());
        }

        let full_bytes = bits_len / 8;
        let remaining_bits = bits_len % 8;
        let mut result = BigInt::zero();

        // Read full bytes
        for _ in 0..full_bytes {
            let byte = reader.read::<8, u8>()?;
            result = (result << 8) | BigInt::from(byte);
        }

        // Read remaining bits if any
        if remaining_bits > 0 {
            let last_bits = reader.read_var::<u8>(remaining_bits as u32)?;
            result = (result << remaining_bits) | BigInt::from(last_bits);
        }

        // Check if the sign bit is set
        let sign_bit_mask = BigInt::from(1) << (bits_len - 1);
        if &result & &sign_bit_mask != BigInt::zero() {
            // Negative number - sign extend by subtracting 2^bits_len
            let modulus = BigInt::from(1) << bits_len;
            result -= modulus;
        }

        Ok(result)
    }
    fn tcn_is_zero(&self) -> bool { *self == Self::from(0u32) }
    fn tcn_shr(&self, _bits: usize) -> Self { self >> _bits }
    fn tcn_min_bits_len(&self) -> u32 {
        if self.tcn_is_zero() {
            0u32
        } else {
            (self.bits() as u32) + 1u32
        }
    }
}

macro_rules! ton_cell_num_fastnum_unsigned_impl {
    ($src:ty) => {
        impl TonCellNum for $src {
            fn tcn_write_bits(&self, writer: &mut CellBitWriter, bits_len: u32) -> Result<(), TonCoreError> {
                if bits_len == 0 {
                    return Ok(());
                }
                let bits_len = bits_len as usize;
                if ((bits_len as usize) > size_of::<$src>() * 8) {
                    bail_ton_core_data!("Requested bits {} more that sizeof  {}", bits_len, size_of::<$src>() * 8);
                }
                if (bits_len < self.tcn_min_bits_len() as usize) {
                    bail_ton_core_data!(
                        "Not enouth bits for write num {} in {} bits unsigned  min len {}",
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

            fn tcn_read_bits(reader: &mut CellBitReader, bits_len: u32) -> Result<Self, TonCoreError> {
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

            fn tcn_shr(&self, bits: usize) -> Self { *self >> bits }
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
            fn tcn_write_bits(&self, writer: &mut CellBitWriter, bits_len: u32) -> Result<(), TonCoreError> {
                // Encode signed value: (abs(value) << 1) | (sign ? 1 : 0)
                let zero: $src = Self::from(0u32);
                let sign = *self < zero;
                let magnitude = self.abs();

                // Convert magnitude to unsigned and encode sign in LSB
                // We need to manually convert Int to UInt by building it byte by byte
                let bytes_count = std::mem::size_of::<$src>();
                let mut bytes_vec = Vec::with_capacity(bytes_count);
                let mut temp = magnitude;

                for _ in 0..bytes_count {
                    let byte_val = (temp.clone() & Self::from(0xFFu32)).to_string().parse::<u8>().unwrap_or(0);
                    bytes_vec.push(byte_val);
                    temp >>= 8;
                }
                bytes_vec.reverse(); // Make it big-endian

                // Now construct the UInt from these bytes
                let mut uval = <$u_src>::from(0u32);
                for byte in bytes_vec {
                    uval = (uval << 8) | <$u_src>::from(byte);
                }

                // Encode sign
                uval <<= 1u32;
                if sign {
                    uval += <$u_src>::ONE;
                }

                // Use the unsigned implementation to serialize
                uval.tcn_write_bits(writer, bits_len)
            }

            fn tcn_read_bits(reader: &mut CellBitReader, bits_len: u32) -> Result<Self, TonCoreError> {
                let unsigned_val = <$u_src>::tcn_read_bits(reader, bits_len)?;

                // Decode sign from LSB
                let sign_bit = (unsigned_val.clone() & <$u_src>::ONE) == <$u_src>::ONE;
                let mut magnitude_uint = unsigned_val >> 1u32;

                // Convert UInt magnitude back to Int
                let bytes_count = std::mem::size_of::<$src>();
                let mut bytes_vec = Vec::with_capacity(bytes_count);

                for _ in 0..bytes_count {
                    let byte_val =
                        (magnitude_uint.clone() & <$u_src>::from(0xFFu32)).to_string().parse::<u8>().unwrap_or(0);
                    bytes_vec.push(byte_val);
                    magnitude_uint >>= 8;
                }
                bytes_vec.reverse(); // Make it big-endian

                // Construct Int from bytes
                let mut result = Self::from(0u32);
                for byte in bytes_vec {
                    result = (result << 8) | Self::from(byte);
                }

                if sign_bit {
                    result = -result;
                }

                Ok(result)
            }
            fn tcn_is_zero(&self) -> bool { *self == Self::from(0u32) }
            fn tcn_shr(&self, _bits: usize) -> Self { *self >> _bits }
            fn tcn_min_bits_len(&self) -> u32 {
                if self.tcn_is_zero() {
                    0u32
                } else {
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
    use crate::cell::{CellParser, TonCell};
    use fastnum::{I512, U512};
    use num_bigint::{BigInt, BigUint};
    #[test]
    fn test_toncellnum_store_and_parse_uint16() -> anyhow::Result<()> {
        // Create a builder and store an int16 value
        let mut builder = TonCell::builder();
        let test_value: u16 = 12;

        let test_bit = 5;
        builder.write_num(&test_value, test_bit)?;

        // Build the cell
        let cell = builder.build()?;

        // Create a parser and read back the int16 value
        let mut parser = CellParser::new(&cell);
        let parsed_value = parser.read_num::<u16>(test_bit)?;

        // Verify the value matches
        assert_eq!(parsed_value, test_value);
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
    fn test_toncellnum_store_and_parse_i512() -> anyhow::Result<()> {
        // Create a builder and store a I512 value
        let mut builder = TonCell::builder();
        let test_value = I512::from(1234u32);

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
    fn test_toncellnum_biguint_150_bits() -> anyhow::Result<()> {
        // Test BigUint with 150 bits (the size used in test_dict_key_bits_len_bigger_than_key)
        use num_bigint::BigUint;

        let mut builder = TonCell::builder();
        let test_value = BigUint::from(4u32);

        let bits_len = 150;

        builder.write_num(&test_value, bits_len)?;
        let cell = builder.build()?;

        let mut parser = CellParser::new(&cell);
        let parsed_value = parser.read_num::<BigUint>(bits_len)?;

        assert_eq!(parsed_value, test_value, "BigUint round-trip failed for 150 bits");

        Ok(())
    }

    #[test]
    fn test_toncellnum_u256_simple_non_byte_aligned() -> anyhow::Result<()> {
        // Test U256 (fastnum) with non-byte-aligned bits
        use fastnum::U256;

        let mut builder = TonCell::builder();
        let test_value = U256::from(42u32);

        let bits_len = 9; // Not byte-aligned

        builder.write_num(&test_value, bits_len)?;
        let cell = builder.build()?;

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
        assert_eq!(cell.data_bits_len, 0, "Cell should have 0 data bits");

        Ok(())
    }
}
