use num_bigint::{BigInt, BigUint, Sign};
use num_traits::{One, Signed, Zero};

use bitstream_io::Integer;
use std::fmt::Display;
// fastnum support temporarily disabled due to API compatibility issues

use crate::bail_ton_core_data;
use crate::errors::TonCoreError;
use fastnum::bint::{Int, UInt};
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

macro_rules! unsigned_to_signed {
    ($src_ty:ty, $dst_ty:ty, $val:expr) => {{
        let mut uval: $src_ty = $val;
        let sign_bit = (uval & 1) != 0;
        uval >>= 1;
        let mut sval = uval as $dst_ty;
        if sign_bit {
            sval = -sval;
        }
        sval
    }};
}

macro_rules! primitive_signed_to_unsigned {
    ($src_ty:ty, $dst_ty:ty, $val:expr) => {{
        let value: $src_ty = $val;
        let sign = value < 0;
        let mut uval = value.unsigned_abs();
        uval <<= 1;
        if sign {
            uval += 1;
        }
        uval as $dst_ty
    }};
}

macro_rules! primitive_unsigned_to_signed {
    ($src_ty:ty, $dst_ty:ty, $val:expr) => {{
        let mut uval: $src_ty = $val;
        let sign_bit = (uval & 1) != 0;
        uval >>= 1;
        let mut sval = uval as $dst_ty;
        if sign_bit {
            sval = -sval;
        }
        sval
    }};
}

macro_rules! ton_cell_num_primitive_signed_from_unsigned_impl {
    ($src:ty, $src_usinged:ty) => {
        impl TonCellNum for $src {
            fn tcn_to_bytes(&self, bits_len: usize) -> Result<Vec<u8>, TonCoreError> {
                let uval: $src_usinged = primitive_signed_to_unsigned!($src, $src_usinged, *self);
                let bytes = uval.tcn_to_bytes(bits_len)?;
                Ok(bytes)
            }

            fn tcn_from_bytes(data: Vec<u8>, bits_len: usize) -> Result<Self, TonCoreError> {
                let mut unsigned_val = <$src_usinged>::tcn_from_bytes(data, bits_len)?;

                Ok(primitive_unsigned_to_signed!($src_usinged, $src, unsigned_val))
            }
            fn highest_bit_pos_ignore_sign(&self) -> Option<u32> {
                let val = self.unsigned_abs();
                val.highest_bit_pos_ignore_sign()
            }

            fn tcn_is_zero(&self) -> bool { *self == 0 }
            fn tcn_shr(&self, _bits: usize) -> Self { *self >> _bits }
            fn tcn_min_bits_len(&self) -> u32 {
                let rz = self.unsigned_abs().tcn_min_bits_len() - 1;
                assert_ne!(rz, 0);
                rz
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
                let num_bytes = (bits_len + 7) / 8;

                // Adjust value if bits_len is not byte-aligned
                let mut value = *self;
                if bits_len % 8 != 0 {
                    value = value << (8 - bits_len % 8);
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
                        result = result | ((byte as $src) << shift_amount);
                    }
                }

                // Shift right if bits_len is not byte-aligned
                if bits_len % 8 != 0 {
                    result = result >> (8 - bits_len % 8);
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

ton_cell_num_primitive_signed_from_unsigned_impl!(i8, u8);
ton_cell_num_primitive_signed_from_unsigned_impl!(i16, u16);
ton_cell_num_primitive_signed_from_unsigned_impl!(i32, u32);
ton_cell_num_primitive_signed_from_unsigned_impl!(i64, u64);
ton_cell_num_primitive_signed_from_unsigned_impl!(i128, u128);

fn bigint_signed_to_unsigned(value: &BigInt) -> BigUint {
    let sign = value.is_negative();
    let mut uval = value.magnitude().clone(); // get |value|

    // Shift left 1 bit to make room for sign
    uval <<= 1u8;

    // Add 1 if negative
    if sign {
        uval += BigUint::one();
    }

    uval
}

/// Convert `BigUint` → `BigInt`
/// Decodes sign from least significant bit (LSB = 1 if negative).
pub fn bigint_unsigned_to_signed(value: &BigUint) -> BigInt {
    let sign_bit = value.bit(0);
    let mut mag = value >> 1u8; // shift right to remove sign bit

    if sign_bit {
        BigInt::from_biguint(Sign::Minus, mag)
    } else {
        BigInt::from_biguint(Sign::Plus, mag)
    }
}

// Implementation for BigUint
impl TonCellNum for BigUint {
    fn tcn_to_bytes(&self, bits_len: usize) -> Result<Vec<u8>, TonCoreError> {
        if bits_len == 0 {
            return Ok(vec![]);
        }

        // Calculate number of bytes needed
        let num_bytes = (bits_len + 7) / 8;

        // Get the big-endian bytes
        let mut bytes = self.to_bytes_be();

        // If not byte-aligned, shift left
        if bits_len % 8 != 0 {
            let shift = 8 - (bits_len % 8);
            let mut carry = 0u16;
            for byte in bytes.iter_mut().rev() {
                let temp = (*byte as u16) << shift;
                *byte = (temp | carry) as u8;
                carry = temp >> 8;
            }
            if carry > 0 {
                bytes.insert(0, carry as u8);
            }
        }

        // Pad with leading zeros if needed
        while bytes.len() < num_bytes {
            bytes.insert(0, 0);
        }

        Ok(bytes)
    }

    fn tcn_from_bytes(data: Vec<u8>, bits_len: usize) -> Result<Self, TonCoreError> {
        if bits_len == 0 {
            return Ok(BigUint::zero());
        }

        let mut result = BigUint::from_bytes_be(&data);

        // Shift right if bits_len is not byte-aligned
        if bits_len % 8 != 0 {
            result = result >> (8 - bits_len % 8);
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
        let zero = BigInt::from(0i8);
        let sign = *self < zero;
        let uval = bigint_signed_to_unsigned(self);
        let bytes = uval.tcn_to_bytes(bits_len)?;
        Ok(bytes)
    }

    fn tcn_from_bytes(data: Vec<u8>, bits_len: usize) -> Result<Self, TonCoreError> {
        let mut unsigned_val = BigUint::tcn_from_bytes(data, bits_len)?;
        let result = bigint_unsigned_to_signed(&unsigned_val);

        Ok(result)
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
                let num_bytes = (bits_len + 7) / 8;

                // Adjust value if bits_len is not byte-aligned
                let mut value = *self;
                if bits_len % 8 != 0 {
                    value = value << (8 - bits_len % 8);
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
                    result = result >> (8 - bits_len % 8);
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
                    temp = temp >> 8;
                }
                bytes_vec.reverse(); // Make it big-endian

                // Now construct the UInt from these bytes
                let mut uval = <$u_src>::from(0u32);
                for byte in bytes_vec {
                    uval = (uval << 8) | <$u_src>::from(byte);
                }

                // Encode sign
                uval = uval << 1u32;
                if sign {
                    uval = uval + <$u_src>::ONE;
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
                    magnitude_uint = magnitude_uint >> 8;
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
    use fastnum::U128;

    use crate::cell::{CellParser, TonCell, TonCellNum};
    use fastnum::{I512, U512};
    use num_bigint::{BigInt, BigUint, Sign};
    use num_traits::{One, Signed, Zero};
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
}
