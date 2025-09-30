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
        // unreachable!()

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
