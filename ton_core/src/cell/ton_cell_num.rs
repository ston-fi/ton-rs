use bitstream_io::Integer;
use num_bigint::{BigInt, BigUint};
use num_traits::Zero;
use std::fmt::Display;

use crate::bail_ton_core_data;
use crate::cell::CellBuilder;
use crate::errors::TonCoreError;

/// Allows generic read/write operation for any numeric type
pub trait TonCellNum: Display + Sized + Clone {
    const SIGNED: bool;
    const IS_PRIMITIVE: bool;
    type Primitive: Zero + Integer;
    type UnsignedPrimitive: Integer;

    fn tcn_from_bytes(bytes: &[u8]) -> Self;
    fn tcn_to_bytes(&self) -> Vec<u8>;

    fn tcn_from_primitive(value: Self::Primitive) -> Self;
    fn tcn_to_unsigned_primitive(&self) -> Option<Self::UnsignedPrimitive>;

    fn tcn_is_zero(&self) -> bool;
    fn tcn_min_bits_len(&self) -> usize; // must includes sign bit if SIGNED=true
    fn tcn_shr(&self, bits: usize) -> Self;

    fn write_to(&self, builder: &mut CellBuilder, bits_len: usize) -> Result<(), TonCoreError> {
        // handling it like ton-core
        // https://github.com/ton-core/ton-core/blob/main/src/boc/BitBuilder.ts#L122

        if let Some(unsigned) = self.tcn_to_unsigned_primitive() {
            builder.write_unsigned_number(unsigned, bits_len)?;
            return Ok(());
        }

        let min_bits_len = self.tcn_min_bits_len();
        if min_bits_len > bits_len {
            bail_ton_core_data!("Can't write number {} ({} bits) in {} bits", self, min_bits_len, bits_len);
        }

        let data_bytes = self.tcn_to_bytes();
        let padding_val: u8 = match (Self::SIGNED, data_bytes[0] >> 7 != 0) {
            (true, true) => 255,
            _ => 0,
        };
        let padding_bits_len = bits_len.saturating_sub(min_bits_len);
        let padding_to_write = vec![padding_val; padding_bits_len.div_ceil(8)];
        builder.write_bits(padding_to_write, padding_bits_len)?;

        let bits_offset = (data_bytes.len() * 8).saturating_sub(min_bits_len);
        builder.write_bits_with_offset(data_bytes, bits_len - padding_bits_len, bits_offset)
    }
}

// Implementation for primitive types
macro_rules! ton_cell_num_primitive_impl {
    ($src:ty, $sign:tt, $unsign:ty) => {
        impl TonCellNum for $src {
            const SIGNED: bool = $sign;
            const IS_PRIMITIVE: bool = true;
            type Primitive = $src;
            type UnsignedPrimitive = $unsign;
            fn tcn_from_bytes(_bytes: &[u8]) -> Self {
                unreachable!()
            }
            fn tcn_to_bytes(&self) -> Vec<u8> {
                unreachable!()
            }

            fn tcn_from_primitive(value: Self::Primitive) -> Self {
                value
            }
            fn tcn_to_unsigned_primitive(&self) -> Option<Self::UnsignedPrimitive> {
                Some(*self as $unsign)
            }

            fn tcn_is_zero(&self) -> bool {
                *self == 0
            }
            fn tcn_min_bits_len(&self) -> usize {
                unreachable!()
            }
            fn tcn_shr(&self, _bits: usize) -> Self {
                unreachable!()
            }
        }
    };
}

ton_cell_num_primitive_impl!(i8, true, u8);
ton_cell_num_primitive_impl!(u8, false, u8);
ton_cell_num_primitive_impl!(i16, true, u16);
ton_cell_num_primitive_impl!(u16, false, u16);
ton_cell_num_primitive_impl!(i32, true, u32);
ton_cell_num_primitive_impl!(u32, false, u32);
ton_cell_num_primitive_impl!(i64, true, u64);
ton_cell_num_primitive_impl!(u64, false, u64);
ton_cell_num_primitive_impl!(i128, true, u128);
ton_cell_num_primitive_impl!(u128, false, u128);

// Implementation for usize
impl TonCellNum for usize {
    const SIGNED: bool = false;
    const IS_PRIMITIVE: bool = true;
    type Primitive = u128;
    type UnsignedPrimitive = u128;
    fn tcn_from_bytes(_bytes: &[u8]) -> Self {
        unreachable!()
    }
    fn tcn_to_bytes(&self) -> Vec<u8> {
        unreachable!()
    }

    fn tcn_from_primitive(value: Self::Primitive) -> Self {
        value as Self
    }
    fn tcn_to_unsigned_primitive(&self) -> Option<Self::UnsignedPrimitive> {
        Some(*self as u128)
    }

    fn tcn_is_zero(&self) -> bool {
        *self == 0
    }
    fn tcn_min_bits_len(&self) -> usize {
        unreachable!()
    } // extra bit for sign
    fn tcn_shr(&self, _bits: usize) -> Self {
        unreachable!()
    }
}


// Implementation for BigInt and BigUint
impl TonCellNum for BigInt {
    const SIGNED: bool = true;
    const IS_PRIMITIVE: bool = false;
    type Primitive = i128;
    type UnsignedPrimitive = u128;
    fn tcn_from_bytes(bytes: &[u8]) -> Self {
        BigInt::from_signed_bytes_be(bytes)
    }
    fn tcn_to_bytes(&self) -> Vec<u8> {
        BigInt::to_signed_bytes_be(self)
    }

    fn tcn_from_primitive(value: Self::Primitive) -> Self {
        value.into()
    }
    fn tcn_to_unsigned_primitive(&self) -> Option<Self::UnsignedPrimitive> {
        None
    }

    fn tcn_is_zero(&self) -> bool {
        Zero::is_zero(self)
    }
    fn tcn_min_bits_len(&self) -> usize {
        self.bits() as usize + 1
    } // extra bit for sign
    fn tcn_shr(&self, bits: usize) -> Self {
        self >> bits
    }
}


impl TonCellNum for BigUint {
    const SIGNED: bool = false;
    const IS_PRIMITIVE: bool = false;
    type Primitive = u128;
    type UnsignedPrimitive = u128;
    fn tcn_from_bytes(bytes: &[u8]) -> Self {
        BigUint::from_bytes_be(bytes)
    }
    fn tcn_to_bytes(&self) -> Vec<u8> {
        BigUint::to_bytes_be(self)
    }

    fn tcn_from_primitive(value: Self::Primitive) -> Self {
        value.into()
    }
    fn tcn_to_unsigned_primitive(&self) -> Option<Self::UnsignedPrimitive> {
        None
    }

    fn tcn_is_zero(&self) -> bool {
        Zero::is_zero(self)
    }
    fn tcn_min_bits_len(&self) -> usize {
        self.bits() as usize
    }
    fn tcn_shr(&self, bits: usize) -> Self {
        self >> bits
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::TonCell;
    use num_bigint::{BigInt, BigUint};
    use std::str::FromStr;

    // Helper function to test basic TonCellNum trait constants
    fn test_ton_cell_num_constants<T: TonCellNum>(
        expected_signed: bool,
        expected_is_primitive: bool,
    ) {
        assert_eq!(T::SIGNED, expected_signed);
        assert_eq!(T::IS_PRIMITIVE, expected_is_primitive);
    }

    // Test struct containing all numeric types for comprehensive testing
    #[derive(Debug, Clone)]
    struct NumericTestValues {
        // Signed types
        i8_val: i8,
        i16_val: i16,
        i32_val: i32,
        i64_val: i64,
        i128_val: i128,
        bigint_val: BigInt,
        
        // Unsigned types
        u8_val: u8,
        u16_val: u16,
        u32_val: u32,
        u64_val: u64,
        u128_val: u128,
        usize_val: usize,
        biguint_val: BigUint,
    }

    impl NumericTestValues {
        fn new() -> Self {
            Self {
                // Signed values (using negative values for better testing)
                i8_val: -42i8,
                i16_val: -42i16,
                i32_val: -42i32,
                i64_val: -42i64,
                i128_val: -42i128,
                bigint_val: BigInt::from(-42),
                
                // Unsigned values
                u8_val: 42u8,
                u16_val: 42u16,
                u32_val: 42u32,
                u64_val: 42u64,
                u128_val: 42u128,
                usize_val: 42usize,
                biguint_val: BigUint::from(42u32),
            }
        }

        fn test_all_signed_traits(&self) {
            // Test i8
            test_ton_cell_num_constants::<i8>(true, true);
            assert!(!self.i8_val.tcn_is_zero());
            assert!(0i8.tcn_is_zero());
            if let Some(unsigned) = self.i8_val.tcn_to_unsigned_primitive() {
                assert_eq!(unsigned, 214u8); // -42 as u8 wraps to 214
            }

            // Test i16
            test_ton_cell_num_constants::<i16>(true, true);
            assert!(!self.i16_val.tcn_is_zero());
            assert!(0i16.tcn_is_zero());
            if let Some(unsigned) = self.i16_val.tcn_to_unsigned_primitive() {
                assert_eq!(unsigned, 65494u16); // -42 as u16 wraps to 65494
            }

            // Test i32
            test_ton_cell_num_constants::<i32>(true, true);
            assert!(!self.i32_val.tcn_is_zero());
            assert!(0i32.tcn_is_zero());
            if let Some(unsigned) = self.i32_val.tcn_to_unsigned_primitive() {
                assert_eq!(unsigned, 4294967254u32); // -42 as u32 wraps to 4294967254
            }

            // Test i64
            test_ton_cell_num_constants::<i64>(true, true);
            assert!(!self.i64_val.tcn_is_zero());
            assert!(0i64.tcn_is_zero());
            if let Some(unsigned) = self.i64_val.tcn_to_unsigned_primitive() {
                assert_eq!(unsigned, 18446744073709551574u64); // -42 as u64 wraps to 18446744073709551574
            }

            // Test i128
            test_ton_cell_num_constants::<i128>(true, true);
            assert!(!self.i128_val.tcn_is_zero());
            assert!(0i128.tcn_is_zero());
            if let Some(unsigned) = self.i128_val.tcn_to_unsigned_primitive() {
                assert_eq!(unsigned, 340282366920938463463374607431768211414u128); // -42 as u128 wraps
            }

            // Test BigInt
            test_ton_cell_num_constants::<BigInt>(true, false);
            assert_eq!(self.bigint_val.tcn_is_zero(), false);
            assert_eq!(BigInt::from(0).tcn_is_zero(), true);
            let bytes = self.bigint_val.tcn_to_bytes();
            let reconstructed = BigInt::tcn_from_bytes(&bytes);
            assert_eq!(self.bigint_val, reconstructed);
            let min_bits = self.bigint_val.tcn_min_bits_len();
            assert!(min_bits > 0);
            assert!(min_bits <= 8); // -42 should fit in 8 bits (including sign)
            let shifted = self.bigint_val.tcn_shr(1);
            assert_eq!(shifted, BigInt::from(-21));
            assert!(self.bigint_val.tcn_to_unsigned_primitive().is_none());
        }

        fn test_all_unsigned_traits(&self) {
            // Test u8
            test_ton_cell_num_constants::<u8>(false, true);
            assert!(!self.u8_val.tcn_is_zero());
            assert!(0u8.tcn_is_zero());
            if let Some(unsigned) = self.u8_val.tcn_to_unsigned_primitive() {
                assert_eq!(unsigned, 42u8);
            }

            // Test u16
            test_ton_cell_num_constants::<u16>(false, true);
            assert!(!self.u16_val.tcn_is_zero());
            assert!(0u16.tcn_is_zero());
            if let Some(unsigned) = self.u16_val.tcn_to_unsigned_primitive() {
                assert_eq!(unsigned, 42u16);
            }

            // Test u32
            test_ton_cell_num_constants::<u32>(false, true);
            assert!(!self.u32_val.tcn_is_zero());
            assert!(0u32.tcn_is_zero());
            if let Some(unsigned) = self.u32_val.tcn_to_unsigned_primitive() {
                assert_eq!(unsigned, 42u32);
            }

            // Test u64
            test_ton_cell_num_constants::<u64>(false, true);
            assert!(!self.u64_val.tcn_is_zero());
            assert!(0u64.tcn_is_zero());
            if let Some(unsigned) = self.u64_val.tcn_to_unsigned_primitive() {
                assert_eq!(unsigned, 42u64);
            }

            // Test u128
            test_ton_cell_num_constants::<u128>(false, true);
            assert!(!self.u128_val.tcn_is_zero());
            assert!(0u128.tcn_is_zero());
            if let Some(unsigned) = self.u128_val.tcn_to_unsigned_primitive() {
                assert_eq!(unsigned, 42u128);
            }

            // Test usize
            test_ton_cell_num_constants::<usize>(false, true);
            assert!(!self.usize_val.tcn_is_zero());
            assert!(0usize.tcn_is_zero());
            if let Some(unsigned) = self.usize_val.tcn_to_unsigned_primitive() {
                assert_eq!(unsigned, 42u128);
            }

            // Test BigUint
            test_ton_cell_num_constants::<BigUint>(false, false);
            assert_eq!(self.biguint_val.tcn_is_zero(), false);
            assert_eq!(BigUint::from(0u32).tcn_is_zero(), true);
            let bytes = self.biguint_val.tcn_to_bytes();
            let reconstructed = BigUint::tcn_from_bytes(&bytes);
            assert_eq!(self.biguint_val, reconstructed);
            let min_bits = self.biguint_val.tcn_min_bits_len();
            assert!(min_bits > 0);
            assert!(min_bits <= 8); // 42 should fit in 8 bits
            let shifted = self.biguint_val.tcn_shr(1);
            assert_eq!(shifted, BigUint::from(21u32));
            assert!(self.biguint_val.tcn_to_unsigned_primitive().is_none());
        }
    }

    // Grouped tests for signed types
    #[test]
    fn test_all_signed_types() {
        let test_values = NumericTestValues::new();
        test_values.test_all_signed_traits();
    }

    // Grouped tests for unsigned types
    #[test]
    fn test_all_unsigned_types() {
        let test_values = NumericTestValues::new();
        test_values.test_all_unsigned_traits();
    }

    // Test write_to functionality with CellBuilder using the test struct
    #[test]
    fn test_write_to_all_types() -> anyhow::Result<()> {
        let test_values = NumericTestValues::new();
        
        // Test signed types write_to
        let mut builder = TonCell::builder();
        test_values.i8_val.write_to(&mut builder, 8)?;
        let cell = builder.build()?;
        assert_eq!(cell.data, vec![214]); // -42 as u8

        let mut builder = TonCell::builder();
        test_values.i32_val.write_to(&mut builder, 32)?;
        let cell = builder.build()?;
        assert_eq!(cell.data, vec![0xFF, 0xFF, 0xFF, 0xD6]); // -42 as i32

        // Test unsigned types write_to
        let mut builder = TonCell::builder();
        test_values.u8_val.write_to(&mut builder, 8)?;
        let cell = builder.build()?;
        assert_eq!(cell.data, vec![42]);

        let mut builder = TonCell::builder();
        test_values.u16_val.write_to(&mut builder, 16)?;
        let cell = builder.build()?;
        assert_eq!(cell.data, vec![0, 42]);

        // Test BigInt write_to
        let mut builder = TonCell::builder();
        test_values.bigint_val.write_to(&mut builder, 8)?;
        let cell = builder.build()?;
        assert_eq!(cell.data, vec![214]); // -42 as u8

        // Test BigUint write_to
        let mut builder = TonCell::builder();
        test_values.biguint_val.write_to(&mut builder, 8)?;
        let cell = builder.build()?;
        assert_eq!(cell.data, vec![42]);

        Ok(())
    }

    #[test]
    fn test_write_to_large_numbers() -> anyhow::Result<()> {
        // Test large positive BigInt
        let mut builder = TonCell::builder();
        let value = BigInt::from_str("123456789012345678901234567890")?;
        value.write_to(&mut builder, 128)?;
        let cell = builder.build()?;
        assert_eq!(cell.data.len(), 16); // 128 bits = 16 bytes

        // Test large negative BigInt
        let mut builder = TonCell::builder();
        let value = BigInt::from_str("-123456789012345678901234567890")?;
        value.write_to(&mut builder, 128)?;
        let cell = builder.build()?;
        assert_eq!(cell.data.len(), 16); // 128 bits = 16 bytes

        // Test large BigUint
        let mut builder = TonCell::builder();
        let value = BigUint::from_str("123456789012345678901234567890")?;
        value.write_to(&mut builder, 128)?;
        let cell = builder.build()?;
        assert_eq!(cell.data.len(), 16); // 128 bits = 16 bytes

        Ok(())
    }

    #[test]
    fn test_write_to_edge_cases() -> anyhow::Result<()> {
        // Test zero values
        let mut builder = TonCell::builder();
        let value = 0i32;
        value.write_to(&mut builder, 32)?;
        let cell = builder.build()?;
        assert_eq!(cell.data, vec![0, 0, 0, 0]);

        // Test maximum values
        let mut builder = TonCell::builder();
        let value = i8::MAX;
        value.write_to(&mut builder, 8)?;
        let cell = builder.build()?;
        assert_eq!(cell.data, vec![127]);

        // Test minimum values
        let mut builder = TonCell::builder();
        let value = i8::MIN;
        value.write_to(&mut builder, 8)?;
        let cell = builder.build()?;
        assert_eq!(cell.data, vec![128]);

        // Test padding for positive numbers
        let mut builder = TonCell::builder();
        let value = 1i32;
        value.write_to(&mut builder, 32)?;
        let cell = builder.build()?;
        assert_eq!(cell.data, vec![0, 0, 0, 1]);

        // Test padding for negative numbers
        let mut builder = TonCell::builder();
        let value = -1i32;
        value.write_to(&mut builder, 32)?;
        let cell = builder.build()?;
        assert_eq!(cell.data, vec![0xFF, 0xFF, 0xFF, 0xFF]);

        Ok(())
    }

    #[test]
    fn test_write_to_insufficient_bits() {
        // Test that writing a number that requires more bits than provided fails
        let mut builder = TonCell::builder();
        let value = 0xFFu8; // Requires 8 bits
        assert!(value.write_to(&mut builder, 7).is_err()); // Only 7 bits provided

        let mut builder = TonCell::builder();
        let value = -1i8; // Requires 8 bits (including sign)
        assert!(value.write_to(&mut builder, 7).is_err()); // Only 7 bits provided
    }

    #[test]
    fn test_write_to_unaligned_bits() -> anyhow::Result<()> {
        // Test writing numbers with non-byte-aligned bit lengths
        let mut builder = TonCell::builder();
        let value = 0b1010u8;
        value.write_to(&mut builder, 4)?;
        let cell = builder.build()?;
        assert_eq!(cell.data, vec![0b1010_0000]);
        assert_eq!(cell.data_bits_len, 4);

        let mut builder = TonCell::builder();
        let value = 0b1010_1010u8;
        value.write_to(&mut builder, 8)?;
        let cell = builder.build()?;
        assert_eq!(cell.data, vec![0b1010_1010]);
        assert_eq!(cell.data_bits_len, 8);

        Ok(())
    }

    #[test]
    fn test_write_to_combined_operations() -> anyhow::Result<()> {
        let test_values = NumericTestValues::new();
        
        // Test writing multiple numbers of different types
        let mut builder = TonCell::builder();
        
        // Write a bit first
        builder.write_bit(true)?;
        
        // Write various number types from the test struct
        test_values.i8_val.write_to(&mut builder, 8)?;
        test_values.u16_val.write_to(&mut builder, 16)?;
        test_values.i32_val.write_to(&mut builder, 32)?;
        
        let cell = builder.build()?;
        
        // Verify the combined result
        // The actual result depends on how the bits are packed
        // Let's just verify the data length and some key bytes
        assert_eq!(cell.data.len(), 8);
        assert_eq!(cell.data_bits_len, 57); // 1 + 8 + 16 + 32 = 57 bits
        
        Ok(())
    }
}
