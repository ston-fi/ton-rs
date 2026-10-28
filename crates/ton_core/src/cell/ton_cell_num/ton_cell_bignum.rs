use crate::cell::TonCellNum;
use crate::cell::{CellBuilder, CellParser};
use fastnum::{I1024, U1024};
use num_bigint::{BigInt, BigUint, Sign};
use num_traits::Zero;

use crate::errors::{TonCoreError, TonCoreResult};

use crate::bail_ton_core_data;

/// Slow implementation uses U/I1024 to support bignum
/// If you use bignum, you likely don't care about performance that much
impl TonCellNum for BigUint {
    fn tcn_write_bits(&self, writer: &mut CellBuilder, bits_len: usize) -> Result<(), TonCoreError> {
        biguint_to_u1024(self)?.tcn_write_bits(writer, bits_len)
    }

    fn tcn_read_bits(reader: &mut CellParser, bits_len: usize) -> Result<Self, TonCoreError> {
        biguint_from_u1024(U1024::tcn_read_bits(reader, bits_len)?)
    }

    fn tcn_min_bits_len(&self) -> usize {
        self.bits() as usize
    }
}

impl TonCellNum for BigInt {
    fn tcn_write_bits(&self, writer: &mut CellBuilder, bits_len: usize) -> Result<(), TonCoreError> {
        bigint_to_i1024(self)?.tcn_write_bits(writer, bits_len)
    }

    fn tcn_read_bits(reader: &mut CellParser, bits_len: usize) -> Result<Self, TonCoreError> {
        bigint_from_i1024(I1024::tcn_read_bits(reader, bits_len)?)
    }

    fn tcn_min_bits_len(&self) -> usize {
        self.bits() as usize + 1 // sign bit
    }
}

fn biguint_from_u1024(val: U1024) -> TonCoreResult<BigUint> {
    if val.is_zero() {
        return Ok(BigUint::ZERO);
    }
    let bytes_le = val.to_radix_le(256);
    Ok(BigUint::from_bytes_le(&bytes_le))
}

fn biguint_to_u1024(value: &BigUint) -> TonCoreResult<U1024> {
    if value.is_zero() {
        return Ok(U1024::ZERO);
    }
    let bytes_le = value.to_bytes_le();
    let Some(res) = U1024::from_le_slice(&bytes_le) else {
        bail_ton_core_data!("Can't convert {value} to U1024");
    };
    Ok(res)
}

fn bigint_from_i1024(val: I1024) -> TonCoreResult<BigInt> {
    if val.is_zero() {
        return Ok(BigInt::ZERO);
    }
    let sign = if val < I1024::ZERO { Sign::Minus } else { Sign::Plus };
    let bytes_le = val.unsigned_abs().to_radix_le(256);
    Ok(BigInt::from_biguint(sign, BigUint::from_bytes_le(&bytes_le)))
}

fn bigint_to_i1024(value: &BigInt) -> TonCoreResult<I1024> {
    if value.is_zero() {
        return Ok(I1024::ZERO);
    }
    let (sign, bytes_le) = value.to_bytes_le();
    let Some(res) = U1024::from_le_slice(&bytes_le).map(|x| x.cast_signed()) else {
        bail_ton_core_data!("Can't convert {value} (unsigned part) to U1024");
    };
    if sign == Sign::Minus && res != I1024::MIN { Ok(res.neg()) } else { Ok(res) }
}

// We don't test cell read/write here, because it's already tested for U1024/I1024
#[cfg(test)]
#[path = "../../tests/cell/ton_cell_num/test_ton_cell_bignum.rs"]
mod tests;
