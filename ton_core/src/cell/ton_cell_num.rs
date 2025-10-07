use bitstream_io::Integer;
use num_bigint::{BigInt, BigUint};
use num_traits::{Signed, Zero};
use std::fmt::Display;
// fastnum support temporarily disabled due to API compatibility issues
use crate::bail_ton_core_data;
use crate::cell::{CellBuilder, CellParser};
use crate::errors::TonCoreError;
use fastnum::{I1024, I128, I256, I512};
use fastnum::{U1024, U128, U256, U512};
use num_traits::real::Real;

fn toncell_data_set_bit(data: &mut Vec<u8>, bit_id: usize, value: bool) -> Result<bool, TonCoreError> {
    // find the bit in data array in bigendian and set in value, Return true if value same and false otherwise

    todo!()
}
fn toncell_data_get_bit(data: &mut Vec<u8>, bit_id: u32) -> Result<bool, TonCoreError> {
    // find the bit in data array in bigendian and set in value, Return true if value same and false otherwise

    todo!()
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

    fn tcn_min_bits_len(&self) -> u32 {
        if let Some(mut value) = self.highest_bit_pos_ignore_sign() {
            value += 1u32; // bit pos to bit size
            value
        } else {
            0u32
        }
    }
}
macro_rules! abs_it {
    ($val:expr, $tp:ty) => {{
        if $val < <$tp>::from(0i8) {
            $val * <$tp>::from(-1i8)
        } else {
            $val
        }
    }};
}

pub trait TonFrom<T> {
    fn ton_from(val: T) -> Self;
}

macro_rules! ton_cell_num_primitive_signed_from_unsigned_impl {
    ($src:ty, $src_usinged:ty) => {
        impl TonCellNum for $src {
            fn tcn_from_bytes(mut data: Vec<u8>, bits_len: usize) -> Result<Self, TonCoreError> {
                let is_positive = toncell_data_set_bit(&mut data, bits_len - 1, false)?;
                let unsigned_val = <$src_usinged>::tcn_from_bytes(data, bits_len - 1)?;
                let mut result: Self = unsigned_val as $src;

                if !is_positive {
                    let zero: $src = unsafe { std::mem::zeroed() };
                    result = zero - result;
                }
                Ok(result)
            }
            fn highest_bit_pos_ignore_sign(&self) -> Option<u32> {
                let val = self.abs();
                val.highest_bit_pos_ignore_sign()
            }
            fn tcn_to_bytes(&self, bits_len: usize) -> Result<Vec<u8>, TonCoreError> {
                let zero: $src = unsafe { std::mem::zeroed() };
                let sign = *self < zero;
                let val = abs_it!(*self, $src);
                let mut bytes = self.tcn_to_bytes(bits_len)?;
                let ret_val = toncell_data_set_bit(&mut bytes, bits_len - 1, sign)?;
                if sign {
                    assert!(ret_val == false, "to_error");
                } else {
                    assert!(ret_val == true, "to_error");
                }
                Ok(bytes)
            }
            fn tcn_is_zero(&self) -> bool { *self == 0 }
            fn tcn_shr(&self, _bits: usize) -> Self { *self >> _bits }
        }
    };
}

// Implementation for primitive types
macro_rules! ton_cell_num_primitive_unsigned_impl {
    ($src:ty) => {
        impl TonCellNum for $src {
            fn highest_bit_pos_ignore_sign(&self) -> Option<u32> { todo!() }
            fn tcn_to_bytes(&self, bits_len: usize) -> Result<Vec<u8>, TonCoreError> { Ok(self.to_be_bytes().to_vec()) }
            fn tcn_from_bytes(data: Vec<u8>, bits_len: usize) -> Result<Self, TonCoreError> {
                if bits_len == 0 {
                    return Ok(Self::zero());
                }

                todo!()
            }

            // fn read_from(parser: &mut CellParser, bits_len: usize) -> Result<Self, TonCoreError> {
            //      parser.read_primitive(bits_len)
            // }
            // fn write_to(&self, builder: &mut CellBuilder, bits_len: usize) -> Result<(), TonCoreError> {
            //     // handling it like ton-core
            //     // https://github.com/ton-core/ton-core/blob/main/src/boc/BitBuilder.ts#L122
            //
            //     let min_bits_len = self.tcn_min_bits_len();
            //     if min_bits_len > (bits_len as u32) {
            //         let tmp_str = if Self::SIGNED { "signed" } else { "unsigned" };
            //         bail_ton_core_data!(
            //             "Can't write {} number {} ({} bits) in {} bits",
            //             tmp_str,
            //             self,
            //             min_bits_len,
            //             bits_len
            //         );
            //     }
            //
            //     builder.write_primitive(*self, bits_len)?;
            //     return Ok(());
            // }
            fn tcn_is_zero(&self) -> bool { *self == 0 }

            fn tcn_shr(&self, _bits: usize) -> Self { unreachable!() }
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
    fn tcn_min_bits_len(&self) -> u32 { todo!() }
    fn tcn_to_bytes(&self, bits_len: usize) -> Result<Vec<u8>, TonCoreError> { Ok(self.to_be_bytes().to_vec()) }
    fn tcn_from_bytes(data: Vec<u8>, bits_len: usize) -> Result<Self, TonCoreError> {
        if bits_len == 0 {
            Ok(Self::zero())
        } else {
            todo!()
        }
    }
    fn tcn_is_zero(&self) -> bool { *self == 0 }

    fn highest_bit_pos_ignore_sign(&self) -> Option<u32> {
        if self.tcn_is_zero() {
            return None;
        }
        let max_bit_id = (std::mem::size_of::<Self>() * 8 - 1) as u32;
        Some(max_bit_id - self.leading_zeros())
    }

    // fn read_from(parser: &mut CellParser, bits_len: usize) -> Result<Self, TonCoreError> {
    //     if bits_len == 0 {
    //         return Ok(0);
    //     }
    //     let value = parser.read_primitive::<u128>(bits_len)?;
    //     Ok(value as usize)
    // }

    // fn write_to(&self, builder: &mut CellBuilder, bits_len: usize) -> Result<(), TonCoreError> {
    //     // handling it like ton-core
    //     // https://github.com/ton-core/ton-core/blob/main/src/boc/BitBuilder.ts#L122
    //
    //     let min_bits_len = self.tcn_min_bits_len();
    //     if min_bits_len > (bits_len as u32) {
    //         bail_ton_core_data!("Can't write usize nm {} ({} bits) in {} bits", self, min_bits_len, bits_len);
    //     }
    //
    //     builder.write_primitive(*self as u128, bits_len)?;
    //     return Ok(());
    // }

    fn tcn_shr(&self, _bits: usize) -> Self { *self >> _bits }
}

// Implementation for BigInt
// impl TonCellNum for BigInt {
//
//     fn tcn_to_bytes(&self) -> Vec<u8> { BigInt::to_signed_bytes_be(self) }
//
//     fn tcn_is_zero(&self) -> bool { Zero::is_zero(self) }
//
//     fn highest_bit_pos_ignore_sign(&self) -> Option<u32> {
//         if self.tcn_is_zero() {
//             return None;
//         }
//         // For BigInt, use bits() which returns the number of bits needed
//         // The highest bit position is bits - 1
//         let bits = self.bits();
//         Some((bits - 1) as u32)
//     }
//
//     fn write_to(&self, builder: &mut CellBuilder, bits_len: usize) -> Result<(), TonCoreError> {
//         // handling it like ton-core
//         // https://github.com/ton-core/ton-core/blob/main/src/boc/BitBuilder.ts#L122
//
//         let min_bits_len = self.tcn_min_bits_len();
//         if min_bits_len > (bits_len as u32) {
//             bail_ton_core_data!("Can't write BigInt {} ({} bits) in {} bits", self, min_bits_len, bits_len);
//         }
//
//         let data_bytes = BigInt::to_signed_bytes_be(self);
//         let padding_val: u8 = match data_bytes.first() {
//             Some(&first_byte) if first_byte >> 7 != 0 => 255,
//             _ => 0,
//         };
//         let padding_bits_len = bits_len.saturating_sub(min_bits_len as usize);
//         let padding_to_write = vec![padding_val; padding_bits_len.div_ceil(8)];
//         builder.write_bits(padding_to_write, padding_bits_len)?;
//
//         let bits_offset = (data_bytes.len() * 8).saturating_sub(min_bits_len as usize);
//         builder.write_bits_with_offset(data_bytes, bits_len - padding_bits_len, bits_offset)
//     }
//
//     fn read_from(parser: &mut CellParser, bits_len: usize) -> Result<Self, TonCoreError> {
//         if bits_len == 0 {
//             return Ok(BigInt::from(0));
//         }
//         let bytes = parser.read_bits(bits_len)?;
//         let res = BigInt::from_signed_bytes_be(&bytes);
//         if bits_len % 8 != 0 {
//             return Ok(res.tcn_shr(8 - bits_len % 8));
//         }
//         Ok(res)
//     }
//
//     fn tcn_shr(&self, bits: usize) -> Self { self >> bits }
// }

// Implementation for BigUint
impl TonCellNum for BigUint {
    fn tcn_to_bytes(&self, bits_len: usize) -> Result<Vec<u8>, TonCoreError> { Ok(BigUint::to_bytes_be(self)) }

    fn tcn_from_bytes(data: Vec<u8>, bits_len: usize) -> Result<Self, TonCoreError> { todo!() }

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

    // fn write_to(&self, builder: &mut CellBuilder, bits_len: usize) -> Result<(), TonCoreError> {
    //     // handling it like ton-core
    //     // https://github.com/ton-core/ton-core/blob/main/src/boc/BitBuilder.ts#L122
    //
    //     let min_bits_len = self.tcn_min_bits_len();
    //     if min_bits_len > (bits_len as u32) {
    //         bail_ton_core_data!("Can't write BigUint {} ({} bits) in {} bits", self, min_bits_len, bits_len);
    //     }
    //
    //     let data_bytes = BigUint::to_bytes_be(self);
    //     // For unsigned, always pad with 0
    //     let padding_val: u8 = 0;
    //     let padding_bits_len = bits_len.saturating_sub(min_bits_len as usize);
    //     let padding_to_write = vec![padding_val; padding_bits_len.div_ceil(8)];
    //     builder.write_bits(padding_to_write, padding_bits_len)?;
    //
    //     let bits_offset = (data_bytes.len() * 8).saturating_sub(min_bits_len as usize);
    //     builder.write_bits_with_offset(data_bytes, bits_len - padding_bits_len, bits_offset)
    // }

    // fn read_from(parser: &mut CellParser, bits_len: usize) -> Result<Self, TonCoreError> {
    //     if bits_len == 0 {
    //         return Ok(BigUint::from(0u32));
    //     }
    //     let bytes = parser.read_bits(bits_len)?;
    //     let res = BigUint::from_bytes_be(&bytes);
    //     if bits_len % 8 != 0 {
    //         return Ok(res.tcn_shr(8 - bits_len % 8));
    //     }
    //     Ok(res)
    // }

    fn tcn_shr(&self, bits: usize) -> Self { self >> bits }
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

    fn tcn_to_bytes(&self, bits_len: usize) -> Result<Vec<u8>, TonCoreError> {
        use num_traits::Signed;
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
                todo!();
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
        }
    };
}

macro_rules! ton_cell_num_fastnum_signed_impl {
    ($src:ty) => {
        impl TonCellNum for $src {
            fn tcn_from_bytes(mut data: Vec<u8>, bits_len: usize) -> Result<Self, TonCoreError> {
                todo!();
            }
            fn highest_bit_pos_ignore_sign(&self) -> Option<u32> { todo!() }
            fn tcn_to_bytes(&self, bits_len: usize) -> Result<Vec<u8>, TonCoreError> {
                todo!();
            }
            fn tcn_is_zero(&self) -> bool { *self == Self::from(0u32) }
            fn tcn_shr(&self, _bits: usize) -> Self { *self >> _bits }
        }
    };
}

// ton_cell_num_fastnum_impl!(I128, true, U128);
//
// ton_cell_num_fastnum_impl!(U256, false, U256);
// ton_cell_num_fastnum_impl!(I256, true, U256);
// //
ton_cell_num_fastnum_unsigned_impl!(U128);
ton_cell_num_fastnum_unsigned_impl!(U256);
ton_cell_num_fastnum_unsigned_impl!(U512);
ton_cell_num_fastnum_unsigned_impl!(U1024);

ton_cell_num_fastnum_signed_impl!(I128);
ton_cell_num_fastnum_signed_impl!(I256);
ton_cell_num_fastnum_signed_impl!(I512);
ton_cell_num_fastnum_signed_impl!(I1024);

// ton_cell_num_fastnum_impl!(I512, true, U512);
// //
// ton_cell_num_fastnum_impl!(U1024, false, U1024);
// ton_cell_num_fastnum_impl!(I1024, true, U1024);

#[cfg(test)]
mod tests {
    use crate::cell::{CellParser, TonCell};
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
    fn test_toncellnum_store_and_parse_i512() -> anyhow::Result<()> {
        // Create a builder and store a usize value
        let mut builder = TonCell::builder();
        let test_value: I512 = (-1234i64).into();

        let test_bit = 30;
        builder.write_num(&test_value, test_bit)?;

        // Build the cell
        let cell = builder.build()?;

        // Create a parser and read back the usize value
        let mut parser = CellParser::new(&cell);
        let parsed_value = parser.read_num::<I512>(test_bit)?;

        // Verify the value matches
        assert_eq!(parsed_value, test_value);

        Ok(())
    }
}
