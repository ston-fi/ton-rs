use bitstream_io::Integer;
use num_bigint::{BigInt, BigUint};
use num_traits::Zero;
use std::fmt::Display;
// fastnum support temporarily disabled due to API compatibility issues
use crate::bail_ton_core_data;
use crate::cell::{CellBuilder, CellParser};
use crate::errors::TonCoreError;
use fastnum::{I1024, I128, I256, I512};
use fastnum::{U1024, U128, U256, U512};
use num_traits::real::Real;
/// Allows generic read/write operation for any numeric type
///
/// Questions
/// Split on Primitive and not Primitive?
pub trait TonCellNum: Display + Sized + Clone {
    const SIGNED: bool;
    const IS_PRIMITIVE: bool;
    type Primitive: Zero + Integer;
    type UnsignedPrimitive: Integer;

    fn tcn_to_bytes(&self) -> Vec<u8> { unreachable!() }

    fn highest_bit_pos_ignore_sign(&self) -> Option<u32>;
    fn tcn_is_zero(&self) -> bool;

    fn tcn_shr(&self, bits: usize) -> Self;

    fn write_to(&self, builder: &mut CellBuilder, bits_len: usize) -> Result<(), TonCoreError>;
    // {
    //     // handling it like ton-core
    //     // https://github.com/ton-core/ton-core/blob/main/src/boc/BitBuilder.ts#L122
    //
    //     let min_bits_len = self.tcn_min_bits_len();
    //     if min_bits_len > bits_len {
    //         bail_ton_core_data!("Can't write number {} ({} bits) in {} bits", self, min_bits_len, bits_len);
    //     }
    //
    //     if let Some(unsigned) = self.tcn_to_unsigned_primitive() {
    //         builder.write_unsigned_number(unsigned, bits_len)?;
    //         return Ok(());
    //     }
    //
    //     let data_bytes = self.tcn_to_bytes();
    //     let padding_val: u8 = match (Self::SIGNED, data_bytes[0] >> 7 != 0) {
    //         (true, true) => 255,
    //         _ => 0,
    //     };
    //     let padding_bits_len = bits_len.saturating_sub(min_bits_len);
    //     let padding_to_write = vec![padding_val; padding_bits_len.div_ceil(8)];
    //     builder.write_bits(padding_to_write, padding_bits_len)?;
    //
    //     let bits_offset = (data_bytes.len() * 8).saturating_sub(min_bits_len);
    //     builder.write_bits_with_offset(data_bytes, bits_len - padding_bits_len, bits_offset)
    // }
    fn read_from(parser: &mut CellParser, bits_len: usize) -> Result<Self, TonCoreError>;
    // {
    //     if Self::IS_PRIMITIVE {
    //         // read_primitive
    //         let primitive = parser.read_primitive::<Self::Primitive>(bits_len )?;
    //         return Ok(Self::tcn_from_primitive(primitive));
    //     }
    //     let bytes = parser.read_bits(bits_len)?;
    //     let res = Self::tcn_from_bytes(&bytes);
    //     if bits_len % 8 != 0 {
    //         return Ok(res.tcn_shr(8 - bits_len % 8));
    //     }
    //     Ok(res)
    // }
    fn tcn_min_bits_len(&self) -> u32 {
        if let Some(mut value) = self.highest_bit_pos_ignore_sign() {
            value += 1u32; // bit pos to bit size
            if Self::SIGNED {
                value += 1u32; // 1 for sign
            }
            value
        } else {
            0u32
        }
    }
}

macro_rules! ton_highest_bit_pos_ignore_sign_impl {
    (true, $unsign:ty) => {
        fn highest_bit_pos_ignore_sign(&self) -> Option<u32> {
            if self.tcn_is_zero() {
                return None;
            }
            let max_bit_id = (std::mem::size_of::<Self>() * 8 - 1) as u32;
            let uval = self.abs() as $unsign;
            return Some(max_bit_id - (uval.leading_zeros()))
        }
    };
    (false, $unsign:ty) => {
        fn highest_bit_pos_ignore_sign(&self) -> Option<u32> {
            if self.tcn_is_zero() {
                return None;
            }
            let max_bit_id = (std::mem::size_of::<Self>() * 8 - 1) as u32;
            return Some(max_bit_id - (*self as $unsign).leading_zeros())
        }
    };
}

// Implementation for primitive types
macro_rules! ton_cell_num_primitive_impl {
    ($src:ty, $sign:tt, $unsign:ty) => {
        impl TonCellNum for $src {
            const SIGNED: bool = $sign;
            const IS_PRIMITIVE: bool = true;
            type Primitive = $src;
            type UnsignedPrimitive = $unsign;

            ton_highest_bit_pos_ignore_sign_impl!($sign, $unsign);

            fn read_from(parser: &mut CellParser, bits_len: usize) -> Result<Self, TonCoreError> {
                if bits_len == 0 {
                    return Ok(Self::Primitive::zero());
                }
                parser.read_primitive(bits_len)
            }
            fn write_to(&self, builder: &mut CellBuilder, bits_len: usize) -> Result<(), TonCoreError> {
                // handling it like ton-core
                // https://github.com/ton-core/ton-core/blob/main/src/boc/BitBuilder.ts#L122

                let min_bits_len = self.tcn_min_bits_len();
                if min_bits_len > (bits_len as u32) {
                    let tmp_str = if Self::SIGNED { "signed" } else { "unsigned" };
                    bail_ton_core_data!(
                        "Can't write {} number {} ({} bits) in {} bits",
                        tmp_str,
                        self,
                        min_bits_len,
                        bits_len
                    );
                }

                builder.write_primitive(*self, bits_len)?;
                return Ok(());
            }
            fn tcn_is_zero(&self) -> bool { *self == 0 }

            fn tcn_shr(&self, _bits: usize) -> Self { unreachable!() }
        }
    };
}

ton_cell_num_primitive_impl!(i8, true, u8);
ton_cell_num_primitive_impl!(i16, true, u16);
ton_cell_num_primitive_impl!(i32, true, u32);
ton_cell_num_primitive_impl!(i64, true, u64);

ton_cell_num_primitive_impl!(u8, false, u8);
ton_cell_num_primitive_impl!(u16, false, u16);
ton_cell_num_primitive_impl!(u32, false, u32);
ton_cell_num_primitive_impl!(u64, false, u64);

ton_cell_num_primitive_impl!(i128, true, u128);
ton_cell_num_primitive_impl!(u128, false, u128);

// Implementation for usize
impl TonCellNum for usize {
    const SIGNED: bool = false;
    const IS_PRIMITIVE: bool = true;
    type Primitive = u128;
    type UnsignedPrimitive = u128;

    fn tcn_is_zero(&self) -> bool { *self == 0 }

    fn highest_bit_pos_ignore_sign(&self) -> Option<u32> {
        if self.tcn_is_zero() {
            return None;
        }
        let max_bit_id = (std::mem::size_of::<Self>() * 8 - 1) as u32;
        return Some(max_bit_id - self.leading_zeros());
    }

    fn read_from(parser: &mut CellParser, bits_len: usize) -> Result<Self, TonCoreError> {
        if bits_len == 0 {
            return Ok(0);
        }
        let value = parser.read_primitive::<u128>(bits_len)?;
        Ok(value as usize)
    }

    fn write_to(&self, builder: &mut CellBuilder, bits_len: usize) -> Result<(), TonCoreError> {
        // handling it like ton-core
        // https://github.com/ton-core/ton-core/blob/main/src/boc/BitBuilder.ts#L122

        let min_bits_len = self.tcn_min_bits_len();
        if min_bits_len > (bits_len as u32) {
            bail_ton_core_data!("Can't write usize nm {} ({} bits) in {} bits", self, min_bits_len, bits_len);
        }

        builder.write_primitive(*self as u128, bits_len)?;
        return Ok(());
    }

    fn tcn_shr(&self, _bits: usize) -> Self { *self >> _bits }
}

// Implementation for BigInt
impl TonCellNum for BigInt {
    const SIGNED: bool = true;
    const IS_PRIMITIVE: bool = false;
    type Primitive = i128;
    type UnsignedPrimitive = u128;

    fn tcn_is_zero(&self) -> bool { Zero::is_zero(self) }

    fn highest_bit_pos_ignore_sign(&self) -> Option<u32> {
        if self.tcn_is_zero() {
            return None;
        }
        // For BigInt, use bits() which returns the number of bits needed
        // The highest bit position is bits - 1
        let bits = self.bits();
        Some((bits - 1) as u32)
    }

    fn write_to(&self, builder: &mut CellBuilder, bits_len: usize) -> Result<(), TonCoreError> {
        // handling it like ton-core
        // https://github.com/ton-core/ton-core/blob/main/src/boc/BitBuilder.ts#L122

        let min_bits_len = self.tcn_min_bits_len();
        if min_bits_len > (bits_len as u32) {
            bail_ton_core_data!("Can't write BigInt {} ({} bits) in {} bits", self, min_bits_len, bits_len);
        }

        let data_bytes = BigInt::to_signed_bytes_be(self);
        let padding_val: u8 = match data_bytes.first() {
            Some(&first_byte) if first_byte >> 7 != 0 => 255,
            _ => 0,
        };
        let padding_bits_len = bits_len.saturating_sub(min_bits_len as usize);
        let padding_to_write = vec![padding_val; padding_bits_len.div_ceil(8)];
        builder.write_bits(padding_to_write, padding_bits_len)?;

        let bits_offset = (data_bytes.len() * 8).saturating_sub(min_bits_len as usize);
        builder.write_bits_with_offset(data_bytes, bits_len - padding_bits_len, bits_offset)
    }

    fn read_from(parser: &mut CellParser, bits_len: usize) -> Result<Self, TonCoreError> {
        if bits_len == 0 {
            return Ok(BigInt::from(0));
        }
        let bytes = parser.read_bits(bits_len)?;
        let res = BigInt::from_signed_bytes_be(&bytes);
        if bits_len % 8 != 0 {
            return Ok(res.tcn_shr(8 - bits_len % 8));
        }
        Ok(res)
    }

    fn tcn_shr(&self, bits: usize) -> Self { self >> bits }
}

// Implementation for BigUint
impl TonCellNum for BigUint {
    const SIGNED: bool = false;
    const IS_PRIMITIVE: bool = false;
    type Primitive = u128;
    type UnsignedPrimitive = u128;

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

    fn write_to(&self, builder: &mut CellBuilder, bits_len: usize) -> Result<(), TonCoreError> {
        // handling it like ton-core
        // https://github.com/ton-core/ton-core/blob/main/src/boc/BitBuilder.ts#L122

        let min_bits_len = self.tcn_min_bits_len();
        if min_bits_len > (bits_len as u32) {
            bail_ton_core_data!("Can't write BigUint {} ({} bits) in {} bits", self, min_bits_len, bits_len);
        }

        let data_bytes = BigUint::to_bytes_be(self);
        // For unsigned, always pad with 0
        let padding_val: u8 = 0;
        let padding_bits_len = bits_len.saturating_sub(min_bits_len as usize);
        let padding_to_write = vec![padding_val; padding_bits_len.div_ceil(8)];
        builder.write_bits(padding_to_write, padding_bits_len)?;

        let bits_offset = (data_bytes.len() * 8).saturating_sub(min_bits_len as usize);
        builder.write_bits_with_offset(data_bytes, bits_len - padding_bits_len, bits_offset)
    }

    fn read_from(parser: &mut CellParser, bits_len: usize) -> Result<Self, TonCoreError> {
        if bits_len == 0 {
            return Ok(BigUint::from(0u32));
        }
        let bytes = parser.read_bits(bits_len)?;
        let res = BigUint::from_bytes_be(&bytes);
        if bits_len % 8 != 0 {
            return Ok(res.tcn_shr(8 - bits_len % 8));
        }
        Ok(res)
    }

    fn tcn_shr(&self, bits: usize) -> Self { self >> bits }
}

//
// // Implementation for BigInt and BigUint
// impl TonCellNum for BigInt {
//     const SIGNED: bool = true;
//     const IS_PRIMITIVE: bool = false;
//     type Primitive = i128;
//     type UnsignedPrimitive = u128;
//     fn tcn_from_bytes(bytes: &[u8]) -> Self { BigInt::from_signed_bytes_be(bytes) }
//     fn tcn_to_bytes(&self) -> Vec<u8> { BigInt::to_signed_bytes_be(self) }
//
//     fn tcn_from_primitive(value: Self::Primitive) -> Self { value.into() }
//     fn tcn_to_unsigned_primitive(&self) -> Option<Self::UnsignedPrimitive> { None }
//
//     fn tcn_is_zero(&self) -> bool { Zero::is_zero(self) }
//     fn tcn_min_bits_len(&self) -> usize { self.bits() as usize + 1 } // extra bit for sign
//     fn tcn_shr(&self, bits: usize) -> Self { self >> bits }
// }
//
// impl TonCellNum for BigUint {
//     const SIGNED: bool = false;
//     const IS_PRIMITIVE: bool = false;
//     type Primitive = u128;
//     type UnsignedPrimitive = u128;
//     fn tcn_from_bytes(bytes: &[u8]) -> Self { BigUint::from_bytes_be(bytes) }
//     fn tcn_to_bytes(&self) -> Vec<u8> { BigUint::to_bytes_be(self) }
//
//     fn tcn_from_primitive(value: Self::Primitive) -> Self { value.into() }
//     fn tcn_to_unsigned_primitive(&self) -> Option<Self::UnsignedPrimitive> { None }
//
//     fn tcn_is_zero(&self) -> bool { Zero::is_zero(self) }
//     fn tcn_min_bits_len(&self) -> usize { self.bits() as usize }
//     fn tcn_shr(&self, bits: usize) -> Self { self >> bits }
// }
//
// macro_rules! ton_cell_num_fastnum_impl {
//     ($src:ty, $sign:tt, $prim:ty) => {
//         impl TonCellNum for $src {
//             const SIGNED: bool = $sign;
//             const IS_PRIMITIVE: bool = false;
//             type Primitive = $prim;
//             type UnsignedPrimitive = u64;
//
//             fn tcn_from_bytes(bytes: &[u8]) -> Self { Self::from_be_slice(bytes).expect("Could not convert bytes ") }
//             fn tcn_to_bytes(&self) -> Vec<u8> {
//                 // Convert Tyoe to big-endian bytes
//                 let mut bytes = vec![0u8; std::mem::size_of::<Self>()];
//
//                 // Try to access the internal representation
//                 // U256 is likely represented as 4 u64 words
//                 // We need to convert to big-endian byte representation
//                 let mut temp = *self;
//                 for i in (0..bytes.len()).rev() {
//                     bytes[i] = (temp & Self::from(0xFFu8)).to_u64().unwrap_or(0) as u8;
//                     temp = temp >> 8;
//                 }
//                 bytes
//             }
//
//             fn tcn_from_primitive(value: Self::Primitive) -> Self {
//                 // Since U256 doesn't have from_words, we'll convert via bytes
//                 let bytes = value.to_be_bytes();
//                 Self::from_be_slice(&bytes).expect("Could not convert u128 to ")
//             }
//             fn tcn_to_unsigned_primitive(&self) -> Option<Self::UnsignedPrimitive> { None }
//
//             fn tcn_is_zero(&self) -> bool { *self == Self::from(0u32) }
//             fn tcn_min_bits_len(&self) -> usize {
//                 // Calculate the minimum number of bits needed to represent this number
//                 if self.tcn_is_zero() {
//                     return if Self::SIGNED { 1 } else { 0 };
//                 }
//
//                 // For fastnum types, we can use the bits() method if available
//                 // Otherwise, find the position of the highest set bit
//                 let mut temp = *self;
//                 let mut bits = 0;
//
//                 // Find the position of the highest set bit
//                 while temp > Self::from(0u32) {
//                     temp = temp >> 1;
//                     bits += 1;
//                 }
//
//                 // Add sign bit for signed numbers
//                 if Self::SIGNED {
//                     bits += 1;
//                 }
//
//                 bits
//             }
//             fn tcn_shr(&self, bits: usize) -> Self { *self >> bits }
//         }
//     };
// }
//
// ton_cell_num_fastnum_impl!(U128, false, u64);
// ton_cell_num_fastnum_impl!(I128, true, i64);
//
// ton_cell_num_fastnum_impl!(U256, false, u64);
// ton_cell_num_fastnum_impl!(I256, true, i64);
//
// ton_cell_num_fastnum_impl!(U512, false, u64);
// ton_cell_num_fastnum_impl!(I512, true, i64);
//
// ton_cell_num_fastnum_impl!(U1024, false, u64);
// ton_cell_num_fastnum_impl!(I1024, true, i64);

#[cfg(test)]
mod tests {
    use crate::cell::{CellParser, TonCell};
    use num_bigint::BigInt;
    use num_bigint::BigUint;
    use num_traits::Signed;

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
        let test_value = BigInt::from(-12);

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
        let test_value: BigUint = BigUint::from(12u64);

        let test_bit = 14;
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
}
