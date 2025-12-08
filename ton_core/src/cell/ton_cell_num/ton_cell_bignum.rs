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

    fn tcn_min_bits_len(&self) -> usize { self.bits() as usize }
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
    if sign == Sign::Minus && res != I1024::MIN {
        Ok(res.neg())
    } else {
        Ok(res)
    }
}

// We don't test cell read/write here, because it's already tested for U1024/I1024
#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_biguint_to_from_u1024() -> anyhow::Result<()> {
        for value in [0, 1, 123, 1024, u128::MAX] {
            let fastnum_expected = U1024::from_u128(value)?;
            let bignum_converted = biguint_from_u1024(fastnum_expected)?;
            let bignum_expected = BigUint::from(value);
            let fastnum_converted = biguint_to_u1024(&bignum_converted)?;
            assert_eq!(bignum_converted, bignum_expected, "value={value}");
            assert_eq!(fastnum_converted, fastnum_expected, "value={value}");
        }
        let fastnum_max_expected = U1024::MAX;
        let bignum_max_converted = biguint_from_u1024(fastnum_max_expected)?;
        let bignum_max_expected = BigUint::from_str(
            "179769313486231590772930519078902473361797697894230657273430081157732675805500963132708477322407536021120113879871393357658789768814416622492847430639474124377767893424865485276302219601246094119453082952085005768838150682342462881473913110540827237163350510684586298239947245938479716304835356329624224137215",
        )?;
        let fastnum_max_converted = biguint_to_u1024(&bignum_max_converted)?;
        assert_eq!(bignum_max_converted, bignum_max_expected, "value=U1024::MAX");
        assert_eq!(fastnum_max_converted, fastnum_max_expected, "value=U1024::MAX");
        Ok(())
    }

    #[test]
    fn test_biguint_to_from_i1024() -> anyhow::Result<()> {
        for value in [i128::MIN, -1024, -123, -1, 0, 1, 123, 1024, i128::MAX] {
            let fastnum_expected = I1024::from_i128(value)?;
            let bignum_converted = bigint_from_i1024(fastnum_expected)?;
            let bignum_expected = BigInt::from(value);
            let fastnum_converted = bigint_to_i1024(&bignum_converted)?;
            assert_eq!(bignum_converted, bignum_expected, "value={value}");
            assert_eq!(fastnum_converted, fastnum_expected, "value={value}");
        }
        let fastnum_max_expected = I1024::MAX;
        let bignum_max_converted = bigint_from_i1024(fastnum_max_expected)?;
        let bignum_max_expected = BigInt::from_str(
            "89884656743115795386465259539451236680898848947115328636715040578866337902750481566354238661203768010560056939935696678829394884407208311246423715319737062188883946712432742638151109800623047059726541476042502884419075341171231440736956555270413618581675255342293149119973622969239858152417678164812112068607",
        )?;
        let fastnum_max_converted = bigint_to_i1024(&bignum_max_converted)?;
        assert_eq!(bignum_max_converted, bignum_max_expected, "value=I1024::MAX");
        assert_eq!(fastnum_max_converted, fastnum_max_expected, "value=I1024::MAX");

        let fastnum_min_expected = I1024::MIN;
        let bignum_min_converted = bigint_from_i1024(fastnum_min_expected)?;
        let bignum_min_expected = BigInt::from_str(
            "-89884656743115795386465259539451236680898848947115328636715040578866337902750481566354238661203768010560056939935696678829394884407208311246423715319737062188883946712432742638151109800623047059726541476042502884419075341171231440736956555270413618581675255342293149119973622969239858152417678164812112068608",
        )?;
        let fastnum_min_converted = bigint_to_i1024(&bignum_min_converted)?;
        assert_eq!(bignum_min_converted, bignum_min_expected, "value=I1024::MIN");
        assert_eq!(fastnum_min_converted, fastnum_min_expected, "value=I1024::MIN");
        Ok(())
    }
}
