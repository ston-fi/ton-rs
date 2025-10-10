use crate::cell::{CellBitReader, CellBitWriter};
use bitstream_io::{BitRead, BitWrite};
use fastnum::{I1024, I128, I256, I512};
use fastnum::{U1024, U128, U256, U512};
use num_bigint::{BigInt, BigUint};
use num_traits::{Signed, Zero};
use std::fmt::Display;

use crate::bail_ton_core_data;
use crate::errors::TonCoreError;

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
                if self.tcn_min_bits_len() > bits_len {
                    bail_ton_core_data!(
                        "Not enough bits for write num {} in {} bits, min len {}",
                        *self,
                        bits_len,
                        self.tcn_min_bits_len()
                    );
                }
                let val: $u_src = primitive_convert_to_unsigned!(*self, $u_src, bits_len);
                writer.write_var(bits_len, val)?;
                Ok(())
            }

            fn tcn_read_bits(reader: &mut CellBitReader, bits_len: u32) -> Result<Self, TonCoreError> {
                if bits_len != 0 {
                    let uval: $u_src = reader.read_var(bits_len)?;
                    let ret: Self = primitive_convert_to_signed!(uval, Self, $u_src, bits_len);
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
                    // Two's complement: abs(val).bit_length() + 1
                    // Which is: primitive_highest_bit_pos + 2
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

ton_cell_num_primitive_signed_impl!(i8, u8);
ton_cell_num_primitive_signed_impl!(i16, u16);
ton_cell_num_primitive_signed_impl!(i32, u32);
ton_cell_num_primitive_signed_impl!(i64, u64);
ton_cell_num_primitive_signed_impl!(i128, u128);

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
                if self.tcn_min_bits_len() > bits_len {
                    bail_ton_core_data!(
                        "Not enough bits for write num {} in {} bits, min len {}",
                        *self,
                        bits_len,
                        self.tcn_min_bits_len()
                    );
                }

                // Two's complement encoding: convert signed to unsigned preserving bit pattern
                // Extract bytes from signed value
                let bytes_count = std::mem::size_of::<$src>();
                let mut bytes_vec = Vec::with_capacity(bytes_count);
                let mut temp = self.clone();

                for _ in 0..bytes_count {
                    let byte_val = (temp.clone() & Self::from(0xFFu32)).to_string().parse::<u8>().unwrap_or(0);
                    bytes_vec.push(byte_val);
                    temp >>= 8;
                }
                bytes_vec.reverse(); // Make it big-endian

                // Construct UInt from bytes (preserves two's complement bit pattern)
                let mut uval = <$u_src>::from(0u32);
                for byte in bytes_vec {
                    uval = (uval << 8) | <$u_src>::from(byte);
                }

                // Mask to bits_len (same as primitives) - MUST do this before tcn_write_bits!
                let bit_count = bits_len as usize;
                let type_bits = std::mem::size_of::<$src>() * 8;

                if bit_count < type_bits {
                    // Mask to bit_count bits
                    let mask = ((<$u_src>::ONE << bits_len) - <$u_src>::ONE);
                    uval = uval & mask;
                }

                // Write directly using the unsigned logic (skip validation since we already validated)
                // This avoids double-validation issue
                if bits_len == 0 {
                    return Ok(());
                }
                let bits_len_usize = bits_len as usize;
                if bits_len_usize > std::mem::size_of::<$u_src>() * 8 {
                    bail_ton_core_data!(
                        "Requested bits {} more than sizeof {}",
                        bits_len,
                        std::mem::size_of::<$u_src>() * 8
                    );
                }

                // Left-align value if not byte-aligned
                let mut value = uval;
                if bits_len % 8 != 0 {
                    value <<= 8 - bits_len % 8;
                }

                // Calculate number of bytes needed
                let num_bytes = (bits_len as usize).div_ceil(8);
                let full_bytes = (bits_len / 8) as usize;
                let remaining_bits = bits_len % 8;

                // Extract bytes in big-endian order
                let mut bytes = Vec::with_capacity(num_bytes);
                for i in (0..num_bytes).rev() {
                    let shift_amount = i * 8;
                    let byte_val = (value.clone() >> shift_amount) & <$u_src>::from(0xFFu32);
                    bytes.push(byte_val.to_string().parse::<u8>().unwrap_or(0));
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

                let unsigned_val = <$u_src>::tcn_read_bits(reader, bits_len)?;

                // Two's complement decoding: convert unsigned to signed preserving bit pattern
                let bytes_count = std::mem::size_of::<$src>();
                let mut bytes_vec = Vec::with_capacity(bytes_count);
                let mut temp_uint = unsigned_val.clone();

                for _ in 0..bytes_count {
                    let byte_val = (temp_uint.clone() & <$u_src>::from(0xFFu32)).to_string().parse::<u8>().unwrap_or(0);
                    bytes_vec.push(byte_val);
                    temp_uint >>= 8;
                }
                bytes_vec.reverse(); // Make it big-endian

                // Construct Int from bytes (preserves two's complement)
                let mut result = Self::from(0u32);
                for byte in bytes_vec {
                    result = (result << 8) | Self::from(byte);
                }

                // Sign extend if needed (when bits_len < full width)
                if bits_len < (std::mem::size_of::<$src>() * 8) as u32 {
                    let sign_bit_pos = bits_len - 1;
                    let sign_bit = <$u_src>::ONE << sign_bit_pos;
                    if (unsigned_val & sign_bit) != <$u_src>::from(0u32) {
                        // Sign bit is set, extend with 1s
                        let mask_uint = !((<$u_src>::ONE << bits_len) - <$u_src>::ONE);
                        // Convert mask to signed by going through bytes
                        let mut mask_bytes = Vec::with_capacity(bytes_count);
                        let mut temp_mask = mask_uint;
                        for _ in 0..bytes_count {
                            let byte_val =
                                (temp_mask.clone() & <$u_src>::from(0xFFu32)).to_string().parse::<u8>().unwrap_or(0);
                            mask_bytes.push(byte_val);
                            temp_mask >>= 8;
                        }
                        mask_bytes.reverse();
                        let mut mask = Self::from(0u32);
                        for byte in mask_bytes {
                            mask = (mask << 8) | Self::from(byte);
                        }
                        result = result | mask;
                    }
                }

                Ok(result)
            }
            fn tcn_is_zero(&self) -> bool { *self == Self::from(0u32) }
            fn tcn_shr(&self, _bits: usize) -> Self { *self >> _bits }
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
    fn test_toncellnum_write_num() -> anyhow::Result<()> {
        let num_val = -9i32;
        let bigint_val = BigInt::from(-9i32);
        let i512_val = I512::from(-9i32);

        let mut num_builder = TonCell::builder();
        let mut bigint_builder = TonCell::builder();
        let mut i512_builder = TonCell::builder();

        num_builder.write_num(&num_val, 10).unwrap();
        bigint_builder.write_num(&bigint_val, 10).unwrap();
        let num_cell = num_builder.build().unwrap();
        println!("{:?}", num_cell.data);
        i512_builder.write_num(&i512_val, 10).unwrap();

        assert_eq!(num_cell.data, vec![0b0000_0001, 0b0000_0010]);
        // assert_eq!(num_cell.data, bigint_builder.);
        assert_eq!(num_cell.data, i512_builder.build()?.data);
        Ok(())
    }

    #[test]
    fn test_toncellnum_write_i512() -> anyhow::Result<()> {
        use fastnum::I512;

        // Test writing and reading I512 values
        let test_cases = vec![
            (I512::from(0i32), 10),
            (I512::from(1i32), 10),
            (I512::from(123i32), 10),
            (-I512::from(1i32), 10), // -1
            (-I512::from(4i32), 10), // -4
        ];

        for (tv, bits) in test_cases {
            let mut builder = TonCell::builder();
            builder.write_num(&tv, bits)?;

            // Verify round-trip
            let cell = builder.build()?;
            let mut parser = cell.parser();
            let parsed = parser.read_num::<I512>(bits)?;
            assert_eq!(parsed, tv, "Failed for value {} with {} bits", tv, bits);
        }

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
