use num_bigint::{BigInt, BigUint};
use num_traits::{Signed, Zero};

use std::fmt::Display;

use crate::bail_ton_core_data;
use crate::errors::TonCoreError;
use fastnum::{I1024, I128, I256, I512};
use fastnum::{U1024, U128, U256, U512};

/// Allows generic read/write operation for any numeric type
///
/// Questions
/// Split on Primitive and not Primitive?
pub trait TonCellNum: Display + Sized + Clone {
    // deprecated
    fn tcn_to_bytes(&self, bits_len: usize) -> Result<Vec<u8>, TonCoreError>;
    fn tcn_from_bytes(data: Vec<u8>, bits_len: usize) -> Result<Self, TonCoreError>;

    fn highest_bit_pos_ignore_sign(&self) -> Option<u32>;
    fn tcn_is_zero(&self) -> bool;

    fn tcn_shr(&self, bits: usize) -> Self;

    fn tcn_min_bits_len(&self) -> u32;
}

macro_rules! ton_cell_num_primitive_signed_impl {
    ($src:ty) => {
        impl TonCellNum for $src {
            fn tcn_to_bytes(&self, bits_len: usize) -> Result<Vec<u8>, TonCoreError> {
                // Signed integers use standard two's complement representation in TON
                // Just serialize the raw bytes like unsigned integers
                if bits_len == 0 {
                    return Ok(vec![]);
                }
                if (bits_len > std::mem::size_of::<$src>() * 8) {
                    bail_ton_core_data!(
                        "Requested bits {} more than sizeof {}",
                        bits_len,
                        std::mem::size_of::<$src>() * 8
                    );
                }
                if (bits_len < self.tcn_min_bits_len() as usize) {
                    bail_ton_core_data!(
                        "Not enough bits for write num {} in {} bits signed, min len {}",
                        *self,
                        bits_len,
                        self.tcn_min_bits_len()
                    );
                }

                // Calculate number of bytes needed
                let num_bytes = bits_len.div_ceil(8);

                // Adjust value if bits_len is not byte-aligned
                let mut value = *self;
                if bits_len % 8 != 0 {
                    value <<= 8 - bits_len % 8;
                }

                // Extract bytes in big-endian order
                let all_bytes = value.to_be_bytes();
                let type_bytes = std::mem::size_of::<$src>();

                // Return only the needed bytes from the end (big-endian)
                Ok(all_bytes[(type_bytes - num_bytes)..].to_vec())
            }

            fn tcn_from_bytes(data: Vec<u8>, bits_len: usize) -> Result<Self, TonCoreError> {
                if bits_len == 0 {
                    return Ok(0);
                }

                // Reconstruct number from bytes as unsigned first
                let mut result: $src = 0;
                let type_bits = std::mem::size_of::<$src>() * 8;

                for (i, &byte) in data.iter().enumerate() {
                    let shift_amount = (data.len() - 1 - i) * 8;
                    // Only shift if it won't overflow
                    if shift_amount < type_bits {
                        result |= (byte as $src) << shift_amount;
                    }
                }

                // Shift right if bits_len is not byte-aligned
                if bits_len % 8 != 0 {
                    result >>= 8 - bits_len % 8;
                }

                // Sign-extend if the MSB of the read bits is set
                if bits_len < type_bits {
                    let sign_bit_pos = bits_len - 1;
                    let sign_bit_mask = 1 << sign_bit_pos;
                    if (result & sign_bit_mask) != 0 {
                        // Negative number - sign extend by setting all higher bits to 1
                        let extension_mask = !((1 << bits_len) - 1);
                        result |= extension_mask;
                    }
                }

                Ok(result)
            }

            fn highest_bit_pos_ignore_sign(&self) -> Option<u32> {
                let val = self.unsigned_abs();
                val.highest_bit_pos_ignore_sign()
            }

            fn tcn_is_zero(&self) -> bool { *self == 0 }
            fn tcn_shr(&self, _bits: usize) -> Self { *self >> _bits }
            fn tcn_min_bits_len(&self) -> u32 {
                let type_bits = (std::mem::size_of::<$src>() * 8) as u32;

                if *self >= 0 {
                    // For non-negative values, need bits for value + 1 for sign bit but 0  needs 0 bit
                    if *self == 0 {
                        0
                    } else {
                        let bits_for_value = type_bits - self.leading_zeros();
                        bits_for_value + 1 // +1 for sign bit
                    }
                } else {
                    let magnitude = self.unsigned_abs();
                    if magnitude == 1 {
                        1 // -1 needs just 1 bit (1)
                    } else {
                        let bits_needed = type_bits - (magnitude - 1).leading_zeros();
                        bits_needed + 1 // +1 for sign bit
                    }
                }
            }
        }
    };
}

macro_rules! ton_cell_num_primitive_unsigned_impl {
    ($src:ty) => {
        impl TonCellNum for $src {
            fn tcn_to_bytes(&self, bits_len: usize) -> Result<Vec<u8>, TonCoreError> {
                if bits_len == 0 {
                    return Ok(vec![]);
                }
                if (bits_len > std::mem::size_of::<$src>() * 8) {
                    bail_ton_core_data!(
                        "Requested bits {} more than sizeof {}",
                        bits_len,
                        std::mem::size_of::<$src>() * 8
                    );
                }
                if (bits_len < self.tcn_min_bits_len() as usize) {
                    bail_ton_core_data!(
                        "Not enouth bits for write num {} in {} bits unsigned  min len {}",
                        *self,
                        bits_len,
                        self.tcn_min_bits_len()
                    );
                }

                // Calculate number of bytes needed
                let num_bytes = bits_len.div_ceil(8);

                // Adjust value if bits_len is not byte-aligned
                let mut value = *self;
                if bits_len % 8 != 0 {
                    value <<= 8 - bits_len % 8;
                }

                // Extract bytes in big-endian order
                let all_bytes = value.to_be_bytes();
                let type_bytes = std::mem::size_of::<$src>();

                // Return only the needed bytes from the end (big-endian)
                Ok(all_bytes[(type_bytes - num_bytes)..].to_vec())
            }

            fn highest_bit_pos_ignore_sign(&self) -> Option<u32> {
                if self.tcn_is_zero() {
                    return None;
                }
                let max_bit_id = (std::mem::size_of::<Self>() * 8 - 1) as u32;
                Some(max_bit_id - self.leading_zeros())
            }

            fn tcn_from_bytes(data: Vec<u8>, bits_len: usize) -> Result<Self, TonCoreError> {
                if bits_len == 0 {
                    return Ok(Self::zero());
                }

                // Reconstruct number from bytes
                let mut result: $src = 0;
                let type_bits = std::mem::size_of::<$src>() * 8;

                for (i, &byte) in data.iter().enumerate() {
                    let shift_amount = (data.len() - 1 - i) * 8;
                    // Only shift if it won't overflow
                    if shift_amount < type_bits {
                        result |= (byte as $src) << shift_amount;
                    }
                }

                // Shift right if bits_len is not byte-aligned
                if bits_len % 8 != 0 {
                    result >>= 8 - bits_len % 8;
                }
                Ok(result)
            }

            fn tcn_is_zero(&self) -> bool { *self == 0 }

            fn tcn_shr(&self, bits: usize) -> Self { *self >> bits }

            fn tcn_min_bits_len(&self) -> u32 {
                if let Some(mut value) = self.highest_bit_pos_ignore_sign() {
                    value += 1u32; // bit pos to bit size
                    value
                } else {
                    0u32
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
ton_cell_num_primitive_unsigned_impl!(usize);

ton_cell_num_primitive_signed_impl!(i8);
ton_cell_num_primitive_signed_impl!(i16);
ton_cell_num_primitive_signed_impl!(i32);
ton_cell_num_primitive_signed_impl!(i64);
ton_cell_num_primitive_signed_impl!(i128);

// Implementation for BigUint
// Note: BigUint is used for BigInt sign encoding
// Must left-align values for non-byte-aligned sizes to match write_bits expectations
impl TonCellNum for BigUint {
    fn tcn_to_bytes(&self, bits_len: usize) -> Result<Vec<u8>, TonCoreError> {
        if bits_len == 0 {
            return Ok(vec![]);
        }

        // Calculate how many bytes we need for bits_len
        let required_bytes = bits_len.div_ceil(8);

        // Left-align the value if not byte-aligned (to match write_bits expectations)
        let value_to_serialize = if bits_len % 8 != 0 {
            self << (8 - bits_len % 8)
        } else {
            self.clone()
        };

        // Get big-endian bytes
        let mut bytes = value_to_serialize.to_bytes_be();

        // Pad with leading zeros if needed
        while bytes.len() < required_bytes {
            bytes.insert(0, 0);
        }

        // Trim if we have extra bytes from the shift operation
        if bytes.len() > required_bytes {
            bytes = bytes[(bytes.len() - required_bytes)..].to_vec();
        }

        Ok(bytes)
    }

    fn tcn_from_bytes(data: Vec<u8>, bits_len: usize) -> Result<Self, TonCoreError> {
        if bits_len == 0 {
            return Ok(BigUint::zero());
        }

        let mut result = BigUint::from_bytes_be(&data);

        // Compensate for read_bits left-aligning the last partial byte
        // read_bits shifts left by (8 - bits_len % 8) when bits_len % 8 != 0
        if bits_len % 8 != 0 {
            result >>= 8 - bits_len % 8;
        }

        Ok(result)
    }

    fn tcn_is_zero(&self) -> bool { Zero::is_zero(self) }

    fn highest_bit_pos_ignore_sign(&self) -> Option<u32> {
        if self.tcn_is_zero() {
            return None;
        }
        // For BigUint, use bits() which returns the number of bits needed
        // The highest bit position is bits - 1
        let bits = self.bits();
        Some((bits - 1) as u32)
    }
    fn tcn_shr(&self, bits: usize) -> Self { self >> bits }

    fn tcn_min_bits_len(&self) -> u32 {
        if let Some(mut value) = self.highest_bit_pos_ignore_sign() {
            value += 1u32; // bit pos to bit size
            value
        } else {
            0u32
        }
    }
}

impl TonCellNum for BigInt {
    fn tcn_to_bytes(&self, bits_len: usize) -> Result<Vec<u8>, TonCoreError> {
        if bits_len == 0 {
            return Ok(vec![]);
        }

        // Use two's complement encoding (standard for TVM int257 and signed integers)
        let mut bytes = self.to_signed_bytes_be();

        // Calculate required bytes
        let required_bytes = bits_len.div_ceil(8);

        // Pad with sign extension if needed
        let pad_byte = if self.is_negative() { 0xFF } else { 0x00 };
        while bytes.len() < required_bytes {
            bytes.insert(0, pad_byte);
        }

        // Trim if too many bytes
        if bytes.len() > required_bytes {
            bytes = bytes[(bytes.len() - required_bytes)..].to_vec();
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

        Ok(bytes)
    }

    fn tcn_from_bytes(data: Vec<u8>, bits_len: usize) -> Result<Self, TonCoreError> {
        if bits_len == 0 {
            return Ok(BigInt::zero());
        }

        let mut bytes = data;

        // Undo left-alignment if not byte-aligned
        if bits_len % 8 != 0 {
            let shift = 8 - (bits_len % 8);
            let mut carry = 0u16;
            for byte in &mut bytes {
                let val = (*byte as u16) | (carry << 8);
                *byte = (val >> shift) as u8;
                carry = val & ((1 << shift) - 1);
            }
        }

        // Check the sign bit at the correct position for bits_len
        let sign_bit_byte_idx = 0;
        let sign_bit_pos_in_byte = if bits_len % 8 == 0 {
            7 // MSB of first byte
        } else {
            (bits_len % 8) - 1
        };

        let is_negative = (bytes[sign_bit_byte_idx] & (1 << sign_bit_pos_in_byte)) != 0;

        // Sign-extend if necessary
        if is_negative && bits_len % 8 != 0 {
            // Set all bits above the significant bits to 1
            let mask = 0xFF << (bits_len % 8);
            bytes[0] |= mask;
        }

        // Use two's complement deserialization
        Ok(BigInt::from_signed_bytes_be(&bytes))
    }
    fn tcn_is_zero(&self) -> bool { *self == Self::from(0u32) }
    fn tcn_shr(&self, _bits: usize) -> Self { self >> _bits }
    fn tcn_min_bits_len(&self) -> u32 {
        if let Some(value) = self.highest_bit_pos_ignore_sign() {
            value + 2
        } else {
            0
        }
    }
    fn highest_bit_pos_ignore_sign(&self) -> Option<u32> {
        if self.tcn_is_zero() {
            return None;
        }
        let max_bit_id = (std::mem::size_of::<Self>() * 8 - 1) as u32;
        Some(max_bit_id - self.bits() as u32)
    }
}

macro_rules! ton_cell_num_fastnum_unsigned_impl {
    ($src:ty) => {
        impl TonCellNum for $src {
            fn tcn_to_bytes(&self, bits_len: usize) -> Result<Vec<u8>, TonCoreError> {
                if bits_len == 0 {
                    return Ok(vec![]);
                }
                if (bits_len > size_of::<$src>() * 8) {
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
                // Calculate number of bytes needed
                let num_bytes = bits_len.div_ceil(8);

                // Adjust value if bits_len is not byte-aligned
                let mut value = *self;
                if bits_len % 8 != 0 {
                    value <<= 8 - bits_len % 8;
                }

                // Extract bytes in big-endian order
                let mut bytes = Vec::with_capacity(num_bytes);
                for i in (0..num_bytes).rev() {
                    let shift_amount = i * 8;
                    let byte_val = (value >> shift_amount) & Self::from(0xFFu32);

                    // Convert to u8 by going through u64
                    // For a single byte value, this is safe
                    bytes.push(byte_val.to_u64().unwrap() as u8);
                }

                Ok(bytes)
            }

            fn tcn_from_bytes(data: Vec<u8>, bits_len: usize) -> Result<Self, TonCoreError> {
                if bits_len == 0 {
                    return Ok(Self::from(0u32));
                }

                // Reconstruct number from bytes
                let mut result = Self::from(0u32);
                for &byte in &data {
                    result = (result << 8) | Self::from(byte);
                }

                // Shift right if bits_len is not byte-aligned
                if bits_len % 8 != 0 {
                    result >>= 8 - bits_len % 8;
                }
                Ok(result)
            }

            fn highest_bit_pos_ignore_sign(&self) -> Option<u32> {
                if self.tcn_is_zero() {
                    return None;
                }
                let max_bit_id = (std::mem::size_of::<Self>() * 8 - 1) as u32;
                Some(max_bit_id - self.leading_zeros())
            }

            fn tcn_is_zero(&self) -> bool { *self == Self::from(0u32) }

            fn tcn_shr(&self, bits: usize) -> Self { *self >> bits }
            fn tcn_min_bits_len(&self) -> u32 {
                if let Some(mut value) = self.highest_bit_pos_ignore_sign() {
                    value += 1u32; // bit pos to bit size
                    value
                } else {
                    0u32
                }
            }
        }
    };
}
macro_rules! ton_cell_num_fastnum_signed_impl {
    ($src:ty,$u_src:ty) => {
        impl TonCellNum for $src {
            fn tcn_to_bytes(&self, bits_len: usize) -> Result<Vec<u8>, TonCoreError> {
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
                uval.tcn_to_bytes(bits_len)
            }

            fn tcn_from_bytes(data: Vec<u8>, bits_len: usize) -> Result<Self, TonCoreError> {
                let unsigned_val = <$u_src>::tcn_from_bytes(data, bits_len)?;

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
                if let Some(value) = self.highest_bit_pos_ignore_sign() {
                    value + 2
                } else {
                    0
                }
            }
            fn highest_bit_pos_ignore_sign(&self) -> Option<u32> {
                if self.tcn_is_zero() {
                    return None;
                }
                let max_bit_id = (std::mem::size_of::<Self>() * 8 - 1) as u32;
                Some(max_bit_id - self.abs().leading_zeros())
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
    use crate::cell::{CellParser, TonCell, TonCellNum};
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
    fn test_bigint_257_bits_serialization() -> anyhow::Result<()> {
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

        println!("Original:  {}", test_value);
        println!("Parsed:    {}", parsed_value);
        println!("Match: {}", test_value == parsed_value);

        // This should pass but currently fails due to alignment issues
        assert_eq!(parsed_value, test_value, "BigInt round-trip failed for 257 bits");

        Ok(())
    }

    #[test]
    fn test_bigint_simple_non_byte_aligned() -> anyhow::Result<()> {
        // Simpler test: 9 bits (2 bytes with 1 bit in second byte)
        use num_bigint::BigInt;

        let mut builder = TonCell::builder();
        let test_value = BigInt::from(42);

        let bits_len = 9; // Not byte-aligned

        // Debug: check what bytes are generated
        let encoded_bytes = test_value.tcn_to_bytes(bits_len)?;
        println!("Value 42 encoded as BigInt:");
        println!("  Bytes: {:02x?}", encoded_bytes);
        println!("  Length: {} bytes for {} bits", encoded_bytes.len(), bits_len);

        builder.write_num(&test_value, bits_len)?;

        let cell = builder.build()?;

        let mut parser = CellParser::new(&cell);

        // Debug: check what bytes we read back
        let read_bytes = parser.read_bits(bits_len)?;
        println!("Read back bytes: {:02x?}", read_bytes);
        parser.seek_bits(-(bits_len as i32))?;

        let parsed_value = parser.read_num::<BigInt>(bits_len)?;

        println!("Original (9 bits):  {}", test_value);
        println!("Parsed (9 bits):    {}", parsed_value);

        assert_eq!(parsed_value, test_value, "BigInt round-trip failed for 9 bits");

        Ok(())
    }

    #[test]
    fn test_biguint_150_bits() -> anyhow::Result<()> {
        // Test BigUint with 150 bits (the size used in test_dict_key_bits_len_bigger_than_key)
        use num_bigint::BigUint;

        let mut builder = TonCell::builder();
        let test_value = BigUint::from(4u32);

        let bits_len = 150;

        println!("Testing BigUint value 4 with 150 bits:");
        builder.write_num(&test_value, bits_len)?;
        let cell = builder.build()?;

        let mut parser = CellParser::new(&cell);
        let parsed_value = parser.read_num::<BigUint>(bits_len)?;

        println!("  Original: {}", test_value);
        println!("  Parsed:   {}", parsed_value);

        assert_eq!(parsed_value, test_value, "BigUint round-trip failed for 150 bits");

        Ok(())
    }
}
