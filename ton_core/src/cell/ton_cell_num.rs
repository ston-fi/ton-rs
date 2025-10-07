
use num_bigint::{BigInt, BigUint};
use num_traits::{Signed, Zero};
use std::fmt::Display;

// fastnum support temporarily disabled due to API compatibility issues

use crate::errors::TonCoreError;
use crate::{bail_ton_core, bail_ton_core_data};
use fastnum::bint::{Int, UInt};
use fastnum::{I1024, I128, I256, I512};
use fastnum::{U1024, U128, U256, U512};



fn toncell_data_set_bit(data: &mut Vec<u8>, bit_id: usize, value: bool) -> Result<bool, TonCoreError> {
    // Find the bit in data array in big-endian format and set it to value
    // Returns the previous value of the bit

    // Calculate which byte contains this bit
    let byte_idx = bit_id / 8;
    let bit_in_byte = 7 - (bit_id % 8); // In big-endian, MSB is bit 7, LSB is bit 0

    if byte_idx >= data.len() {

        bail_ton_core_data!("Bit index {} out of range for {} bytes", bit_id, data.len());
    }

    // Get the current bit value
    let current_value = (data[byte_idx] >> bit_in_byte) & 1 == 1;

    // Set or clear the bit
    if value {
        data[byte_idx] |= 1 << bit_in_byte;
    } else {
        data[byte_idx] &= !(1 << bit_in_byte);
    }

    Ok(current_value)
}

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

macro_rules! ton_cell_num_primitive_signed_from_unsigned_impl {
    ($src:ty, $src_usinged:ty) => {
        impl TonCellNum for $src {
            fn tcn_from_bytes(mut data: Vec<u8>, bits_len: usize) -> Result<Self, TonCoreError> {
                // Get and clear the sign bit
                // toncell_data_set_bit returns the previous value of the bit
                // If it was 1 (true), the number is negative
                let was_negative = toncell_data_set_bit(&mut data, bits_len - 1, false)?;

                // Parse the data with sign bit cleared, using full bits_len
                let unsigned_val = <$src_usinged>::tcn_from_bytes(data, bits_len)?;
                let mut result: Self = unsigned_val as $src;

                if was_negative {
                    let zero: $src = unsafe { std::mem::zeroed() };
                    result = zero - result;
                }
                Ok(result)
            }
            fn highest_bit_pos_ignore_sign(&self) -> Option<u32> {
                let val = self.unsigned_abs();
                val.highest_bit_pos_ignore_sign()
            }
            fn tcn_to_bytes(&self, bits_len: usize) -> Result<Vec<u8>, TonCoreError> {
                let zero: $src = unsafe { std::mem::zeroed() };
                let sign = *self < zero;

                let mut bytes = self.unsigned_abs().tcn_to_bytes(bits_len)?;
                println!("Unsigned_bytes {:?}", bytes);
                let _ret_val = toncell_data_set_bit(&mut bytes, bits_len - 1, sign)?;

                Ok(bytes)
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

// Implementation for primitive types
macro_rules! ton_cell_num_primitive_unsigned_impl {
    ($src:ty) => {
        impl TonCellNum for $src {
            fn highest_bit_pos_ignore_sign(&self) -> Option<u32> {
                if self.tcn_is_zero() {
                    return None;
                }
                let max_bit_id = (std::mem::size_of::<Self>() * 8 - 1) as u32;
                Some(max_bit_id - self.leading_zeros())
            }

            fn tcn_to_bytes(&self, bits_len: usize) -> Result<Vec<u8>, TonCoreError> {
                if bits_len == 0 {
                    return Ok(vec![]);
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
                    let byte_val = ((value >> shift_amount) & (0xFF as $src)) as u8;
                    bytes.push(byte_val);
                }

                Ok(bytes)
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

ton_cell_num_primitive_signed_from_unsigned_impl!(i8, u8);
ton_cell_num_primitive_signed_from_unsigned_impl!(i16, u16);
ton_cell_num_primitive_signed_from_unsigned_impl!(i32, u32);
ton_cell_num_primitive_signed_from_unsigned_impl!(i64, u64);
ton_cell_num_primitive_signed_from_unsigned_impl!(i128, u128);

// Implementation for usize
impl TonCellNum for usize {
    fn tcn_to_bytes(&self, bits_len: usize) -> Result<Vec<u8>, TonCoreError> {
        if bits_len == 0 {
            return Ok(vec![]);
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
            let byte_val = ((value >> shift_amount) & 0xFF) as u8;
            bytes.push(byte_val);
        }

        Ok(bytes)
    }
    fn highest_bit_pos_ignore_sign(&self) -> Option<u32> {
        if self.tcn_is_zero() {
            return None;
        }
        let max_bit_id = (std::mem::size_of::<Self>() * 8 - 1) as u32;
        Some(max_bit_id - self.leading_zeros())
    }

    fn tcn_min_bits_len(&self) -> u32 {
        if let Some(mut value) = self.highest_bit_pos_ignore_sign() {
            value += 1u32; // bit pos to bit size
            value
        } else {
            0u32
        }
    }

    fn tcn_from_bytes(data: Vec<u8>, bits_len: usize) -> Result<Self, TonCoreError> {
        if bits_len == 0 {
            return Ok(0);
        }

        // Reconstruct number from bytes
        let mut result: usize = 0;
        let type_bits = std::mem::size_of::<usize>() * 8;

        for (i, &byte) in data.iter().enumerate() {
            let shift_amount = (data.len() - 1 - i) * 8;
            // Only shift if it won't overflow
            if shift_amount < type_bits {
                result = result | ((byte as usize) << shift_amount);
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
}

// Implementation for BigUint
impl TonCellNum for BigUint {
    fn tcn_to_bytes(&self, _bits_len: usize) -> Result<Vec<u8>, TonCoreError> { Ok(BigUint::to_bytes_be(self)) }

    fn tcn_from_bytes(data: Vec<u8>, _bits_len: usize) -> Result<Self, TonCoreError> { Ok(BigUint::from_bytes_be(&data)) }

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

// Custom implementation for BigInt (doesn't have unsigned_abs method like primitives)
impl TonCellNum for BigInt {
    fn tcn_from_bytes(mut data: Vec<u8>, bits_len: usize) -> Result<Self, TonCoreError> {
        let is_positive = toncell_data_set_bit(&mut data, bits_len - 1, false)?;
        let unsigned_val = BigUint::tcn_from_bytes(data, bits_len - 1)?;
        let mut result: BigInt = unsigned_val.into();
        if !is_positive {
            result *= -1;
        }
        Ok(result)
    }

    fn highest_bit_pos_ignore_sign(&self) -> Option<u32> {
        if self.tcn_is_zero() {
            return None;
        }
        let bits = self.bits();
        Some((bits - 1) as u32)
    }
    fn tcn_min_bits_len(&self) -> u32 {
        let big_uint_val: BigUint = if self.is_negative() {
            let val: BigInt = self.clone() * -1;
            val.to_biguint().unwrap()
        } else {
            self.to_biguint().unwrap()
        };
        big_uint_val.tcn_min_bits_len() - 1
    }
    fn tcn_to_bytes(&self, bits_len: usize) -> Result<Vec<u8>, TonCoreError> {
        let sign = self.is_negative();
        let magnitude = self.magnitude();
        let mut bytes = magnitude.tcn_to_bytes(bits_len)?;
        let ret_val = toncell_data_set_bit(&mut bytes, bits_len - 1, sign)?;
        if sign {
            assert!(ret_val == false, "to_error");
        } else {
            assert!(ret_val == true, "to_error");
        }
        Ok(bytes)
    }

    fn tcn_is_zero(&self) -> bool { Zero::is_zero(self) }

    fn tcn_shr(&self, bits: usize) -> Self { self >> bits }
}

macro_rules! ton_cell_num_fastnum_unsigned_impl {
    ($src:ty) => {
        impl TonCellNum for $src {
            fn tcn_to_bytes(&self, bits_len: usize) -> Result<Vec<u8>, TonCoreError> {
                if bits_len == 0 {
                    return Ok(vec![]);
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

pub fn fastnum_to_unsigned_abs<const N: usize>(value: Int<N>) -> UInt<N> {
    if value.is_negative() {
        let abs_val = -value;
        UInt::<N>::from(abs_val.to_bits())
    } else {
        UInt::<N>::from(value.to_bits())
    }
}

pub fn fastnum_unsigned_to_signed<const N: usize>(value: UInt<N>) -> Int<N> {
    // Convert unsigned value to signed, preserving the numeric value
    // This is similar to casting unsigned to signed in primitives (e.g., 1234u32 as i32)
    // from_bits reinterprets the UInt as an Int with the same bit pattern
    Int::<N>::from_bits(value)
}

macro_rules! ton_cell_num_fastnum_signed_impl {
    ($src:ty,$u_src:ty) => {
        impl TonCellNum for $src {
            fn tcn_from_bytes(mut data: Vec<u8>, bits_len: usize) -> Result<Self, TonCoreError> {
                // Get and clear the sign bit at position bits_len - 1
                let was_negative = toncell_data_set_bit(&mut data, bits_len - 1, false)?;

                // Build the signed value directly from bytes
                if bits_len == 0 {
                    return Ok(<$src>::from(0u32));
                }

                // Reconstruct number from bytes
                let mut result = <$src>::from(0u32);
                for &byte in &data {
                    result = (result << 8) | <$src>::from(byte);
                }

                // Shift right if bits_len is not byte-aligned
                if bits_len % 8 != 0 {
                    result = result >> (8 - bits_len % 8);
                }

                if was_negative {
                    let zero = <$src>::from(0u32);
                    result = zero - result;
                }

                Ok(result)
            }

            fn tcn_to_bytes(&self, bits_len: usize) -> Result<Vec<u8>, TonCoreError> {
                let is_neg = self.is_negative();
                let mut bytes = fastnum_to_unsigned_abs(self.clone()).tcn_to_bytes(bits_len)?;

                if is_neg {
                    let _rv = toncell_data_set_bit(&mut bytes, bits_len - 1, true)?;
                }

                Ok(bytes)
            }
            fn tcn_is_zero(&self) -> bool { *self == Self::from(0u32) }
            fn tcn_shr(&self, _bits: usize) -> Self { *self >> _bits }
            fn tcn_min_bits_len(&self) -> u32 {
                if let Some( value) = self.highest_bit_pos_ignore_sign() {
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
    use crate::cell::ton_cell_num::{ toncell_data_set_bit};
    use crate::cell::{CellParser, TonCell, TonCellNum};
    use fastnum::{I512, U512};
    use num_bigint::BigInt;
    use num_bigint::BigUint;

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
        let test_value = BigInt::from(-900);

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
    fn test_toncellnum_data_set_bit32() -> anyhow::Result<()> {
        let bits_in_val = 32;

        let mut bytes = i32::from(3).tcn_to_bytes(bits_in_val)?;
        println!("Initial bytes: {:?}", bytes);
        println!("Setting bit {} (bits_in_val - 1)", bits_in_val - 1);
        let bit_val = toncell_data_set_bit(&mut bytes, bits_in_val - 1, true)?;
        println!("Previous bit value: {}", bit_val);
        println!("Bytes after set_bit: {:?}", bytes);
        assert_eq!(bit_val, false);
        let result = i32::tcn_from_bytes(bytes, bits_in_val)?;
        println!("Parsed result: {}", result);
        assert!(result < i32::from(0i8), "should be negative");
        
        Ok(())
    }
    #[test]
    fn test_toncellnum_data_set_bit512() -> anyhow::Result<()> {
        let bits_in_val = 512;
        let mut bytes = I512::from(1).tcn_to_bytes(bits_in_val)?;
        let bit_val = toncell_data_set_bit(&mut bytes, bits_in_val - 1, true)?;
        assert_eq!(bit_val, false);
        let result = I512::tcn_from_bytes(bytes, bits_in_val)?;
        assert!(result < I512::from(0i8), "should be negative");

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
