use crate::cell::TonCellNum;
use crate::cell::{CellBuilder, CellParser};
use crate::toncellnum_use_type_as;
use fastnum::U1024;
use fastnum::{TryCast, I1024};
use num_bigint::{BigInt, BigUint, Sign};
use num_traits::Zero;

use crate::errors::TonCoreError;

use crate::bail_ton_core_data;
fn u1024_to_biguint(val: U1024) -> Result<BigUint, TonCoreError> {
    if val.is_zero() {
        return Ok(BigUint::zero());
    }

    let mut tmp = val;
    let mut bytes = Vec::with_capacity(128);

    // Extract bytes from least significant to most significant
    for _ in 0..128 {
        let byte_val = (tmp & 0xFFu8.into())
            .to_u8()
            .map_err(|_| TonCoreError::data("u1024_to_biguint", "Failed to extract byte from U1024"))?;
        bytes.push(byte_val);
        tmp >>= 8;
        if tmp.is_zero() {
            break; // Stop early if remaining value is zero
        }
    }

    bytes.reverse();
    Ok(BigUint::from_bytes_be(&bytes))
}

fn biguint_to_u1024(value: &BigUint) -> Result<U1024, TonCoreError> {
    if value.is_zero() {
        return Ok(U1024::ZERO);
    }

    let bytes = value.to_bytes_be();

    // U1024 can hold at most 128 bytes (1024 bits)
    if bytes.len() > 128 {
        bail_ton_core_data!("BigUint value exceeds U1024 capacity: {} bytes > 128 bytes", bytes.len());
    }

    let mut uval = U1024::ZERO;
    for &b in &bytes {
        uval = (uval << 8) | U1024::from(b);
    }

    Ok(uval)
}

fn i1024_to_bigint(val: I1024) -> Result<BigInt, TonCoreError> {
    if val.is_zero() {
        return Ok(BigInt::zero());
    }
    let is_negative = val < I1024::ZERO;
    let abs_val = if is_negative { -val } else { val };

    let mut tmp: U1024 = TryCast::<U1024>::try_cast(abs_val)
        .map_err(|_| TonCoreError::data("i1024_to_bigint", "Failed to cast to BigInt"))?;
    let mut bytes = Vec::with_capacity(128);

    // Extract bytes from least significant to most significant
    for _ in 0..128 {
        let byte_val = (tmp & 0xFFu8.into())
            .to_u8()
            .map_err(|_| TonCoreError::data("i1024_to_bigint", "Failed to extract byte from U1024"))?;
        bytes.push(byte_val);
        tmp >>= 8;
        if tmp.is_zero() {
            break; // Stop early if remaining value is zero
        }
    }
    bytes.reverse();
    Ok(BigInt::from_bytes_be(if is_negative { Sign::Minus } else { Sign::Plus }, &bytes))
}
toncellnum_use_type_as!(BigUint, U1024, biguint_to_u1024, u1024_to_biguint);
toncellnum_use_type_as!(BigInt, I1024, bigint_to_i1024, i1024_to_bigint);

fn bigint_to_i1024(value: &BigInt) -> Result<I1024, TonCoreError> {
    if value.is_zero() {
        return Ok(I1024::ZERO);
    }

    let (sign, bytes) = value.to_bytes_be();

    // I1024 can hold at most 128 bytes (1024 bits)
    if bytes.len() > 128 {
        bail_ton_core_data!("BigInt value exceeds I1024 capacity: {} bytes > 128 bytes", bytes.len());
    }

    let mut uval = U1024::ZERO;
    for &b in &bytes {
        uval = (uval << 8) | U1024::from(b);
    }

    let result = match sign {
        Sign::Plus => TryCast::<I1024>::try_cast(uval)
            .map_err(|_| TonCoreError::data("bigint_to_i1024", "Failed to cast to I1024"))?,
        Sign::NoSign => I1024::ZERO,
        Sign::Minus => {
            let abs_val = TryCast::<I1024>::try_cast(uval)
                .map_err(|_| TonCoreError::data("bigint_to_i1024", "Failed to cast to I1024"))?;
            -abs_val
        }
    };
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::{bigint_to_i1024, biguint_to_u1024, i1024_to_bigint, u1024_to_biguint};
    use crate::cell::ton_cell_num::tests::test_num_read_write;
    use crate::cell::{CellParser, TonCell};
    use num_bigint::{BigInt, BigUint};

    #[test]
    fn test_toncellnum_bigint_toi1024_conv() {
        let test_big_int = -1 * BigInt::from(1234i64);
        let test_fastnum = bigint_to_i1024(&test_big_int).unwrap();
        let result_big_int = i1024_to_bigint(test_fastnum).unwrap();

        assert_eq!(test_big_int, result_big_int);
    }

    #[test]
    fn test_toncellnum_biguint_tou1024_conv() {
        // Since u1024_to_biguint and biguint_to_u1024 are private functions
        // in the same module, we can call them directly without importing

        // Test with a simple value
        let test_big_uint = BigUint::from(1234u64);
        let test_fastnum = biguint_to_u1024(&test_big_uint).unwrap();
        let result_big_uint = u1024_to_biguint(test_fastnum).unwrap();
        assert_eq!(test_big_uint, result_big_uint);

        // Test with zero
        let test_big_uint = BigUint::from(0u32);
        let test_fastnum = biguint_to_u1024(&test_big_uint).unwrap();
        let result_big_uint = u1024_to_biguint(test_fastnum).unwrap();
        assert_eq!(test_big_uint, result_big_uint);

        // Test with a large value
        let test_big_uint = BigUint::from(u128::MAX);
        let test_fastnum = biguint_to_u1024(&test_big_uint).unwrap();
        let result_big_uint = u1024_to_biguint(test_fastnum).unwrap();
        assert_eq!(test_big_uint, result_big_uint);

        // Test with a very large value (256 bits)
        let test_big_uint = (BigUint::from(1u32) << 255) + BigUint::from(12345u64);
        let test_fastnum = biguint_to_u1024(&test_big_uint).unwrap();
        let result_big_uint = u1024_to_biguint(test_fastnum).unwrap();
        assert_eq!(test_big_uint, result_big_uint);

        let test_big_uint = (BigUint::from(1u32) << 255) - BigUint::from(1u32);
        let test_fastnum = biguint_to_u1024(&test_big_uint).unwrap();
        let result_big_uint = u1024_to_biguint(test_fastnum).unwrap();
        assert_eq!(test_big_uint, result_big_uint, "Failed for 2^255 - 1");
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
    fn test_toncellnum_bignum_corner_cases() -> anyhow::Result<()> {
        let bigval: BigUint = (BigUint::from(1u32) << 255) - BigUint::from(1u32);

        test_num_read_write(
            vec![
                (BigUint::from(0u32), 256),
                (BigUint::from(u128::MAX), 128),
                (bigval, 256),
            ],
            "Big Uint",
        )
        .unwrap();

        test_num_read_write(
            vec![
                (BigInt::from(0i32), 257),
                (BigInt::from(i128::MAX), 128),
                (BigInt::from(i128::MIN), 128),
            ],
            "Big Int ",
        )
        .unwrap();
        Ok(())
    }
}
