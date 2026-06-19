use crate::cell::{CellBuilder, CellParser};

use num_traits::Zero;
use std::fmt::Display;

#[cfg(test)]
mod _ton_cell_num_tests;
mod ton_cell_bignum;
mod ton_cell_fastnum;
mod ton_cell_primitives;

use crate::errors::TonCoreResult;

/// Allows generic read/write operation for any numeric type
/// Implemented for:
/// All primitives types (i8, u8, i16, u16, i32, u32, i64, u64, i128, u128, isize, usize)
/// Fastnum types (I/U128, I/U256, I/U512, I/U1024)
/// BigNum types: BigInt/BigUint
///
/// Caller (CellBuilder) provides the following guarantees:
/// * bits_len != 0 (CellBuilder & CellParser handle zero bits itself)
/// * bits_len >= tcn_min_bits_len()
///
/// Caller (CellParser) provides the following guarantees:
/// * bits_len != 0 (CellBuilder & CellParser handle zero bits_len itself)
pub trait TonCellNum: Display + Sized + Clone + Zero + PartialOrd {
    fn tcn_write_bits(&self, writer: &mut CellBuilder, bits_len: usize) -> TonCoreResult<()>;
    fn tcn_read_bits(reader: &mut CellParser, bits_len: usize) -> TonCoreResult<Self>;
    fn tcn_min_bits_len(&self) -> usize;
}
