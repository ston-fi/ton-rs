use crate::cell::{CellParser, TonCell, TonCellNum};
use anyhow::bail;
use fastnum::*;
use num_bigint::{BigInt, BigUint};
use num_traits::FromPrimitive;
use std::any::type_name;
use std::fmt::Debug;
use tokio_test::assert_err;
use tokio_test::assert_ok;

// Test reading and writing zero bits for all supported numeric types
#[test]
fn test_ton_cell_num_zero_bits_len() -> anyhow::Result<()> {
    // Test that all types return 0 when reading/writing 0 bits
    let builder = TonCell::builder();
    let cell = builder.build()?;
    let mut parser = CellParser::new(&cell);

    // unsigned
    assert_eq!(parser.read_num::<u8>(0)?, 0u8);
    assert_eq!(parser.read_num::<u16>(0)?, 0u16);
    assert_eq!(parser.read_num::<u32>(0)?, 0u32);
    assert_eq!(parser.read_num::<u64>(0)?, 0u64);
    assert_eq!(parser.read_num::<u128>(0)?, 0u128);
    assert_eq!(parser.read_num::<usize>(0)?, 0usize);
    assert_eq!(parser.read_num::<U128>(0)?, U128::from(0u32));
    assert_eq!(parser.read_num::<U256>(0)?, U256::from(0u32));
    assert_eq!(parser.read_num::<U512>(0)?, U512::from(0u32));
    assert_eq!(parser.read_num::<U1024>(0)?, U1024::from(0u32));
    assert_eq!(parser.read_num::<BigUint>(0)?, BigUint::from(0u32));

    // signed
    assert_eq!(parser.read_num::<i8>(0)?, 0i8);
    assert_eq!(parser.read_num::<i16>(0)?, 0i16);
    assert_eq!(parser.read_num::<i32>(0)?, 0i32);
    assert_eq!(parser.read_num::<i64>(0)?, 0i64);
    assert_eq!(parser.read_num::<i128>(0)?, 0i128);
    // assert_eq!(parser.read_num::<isize>(0)?, 0i128); // operation == can't be applied to isize ¯\_(ツ)_/¯
    assert_eq!(parser.read_num::<I128>(0)?, I128::from(0u32));
    assert_eq!(parser.read_num::<I256>(0)?, I256::from(0u32));
    assert_eq!(parser.read_num::<I512>(0)?, I512::from(0u32));
    assert_eq!(parser.read_num::<I1024>(0)?, I1024::from(0u32));
    assert_eq!(parser.read_num::<BigInt>(0)?, BigInt::from(0i32));
    Ok(())
}

#[test]
#[rustfmt::skip]
fn test_ton_cell_num_primitives_corner_cases() -> anyhow::Result<()> {
    // unsigned
    assert_ton_cell_num_read_write(vec![(0u8, 8), (u8::MAX, 8)])?;
    assert_ton_cell_num_read_write(vec![(0u16, 16), (u16::MAX, 16)])?;
    assert_ton_cell_num_read_write(vec![(0u32, 32), (u32::MAX, 32)])?;
    assert_ton_cell_num_read_write(vec![(0u64, 64), (u64::MAX, 64)])?;
    assert_ton_cell_num_read_write(vec![(0u128, 128), (u128::MAX, 128)])?;
    let size_usize = usize::BITS as usize;
    assert_ton_cell_num_read_write(vec![(0usize, size_usize), (usize::MAX, size_usize)])?;

    // signed
    assert_ton_cell_num_read_write(vec![(0i8, 8), (i8::MAX, 8), (i8::MIN, 8), (i8::MIN / 2, 8)])?;
    assert_ton_cell_num_read_write(vec![(0i16, 16), (i16::MAX, 16), (i16::MIN, 16), (i16::MIN / 2, 16)])?;
    assert_ton_cell_num_read_write(vec![(0i32, 32), (i32::MAX, 32), (i32::MIN, 32), (i32::MIN / 2, 32)])?;
    assert_ton_cell_num_read_write(vec![(0i64, 64), (i64::MAX, 64), (i64::MIN, 64), (i64::MIN / 2, 64)])?;
    assert_ton_cell_num_read_write(vec![(0i128, 128), (i128::MAX, 128), (i128::MIN, 128), (i128::MIN / 2, 128)])?;

    let size_usize = isize::BITS as usize;
    assert_ton_cell_num_read_write(vec![(0isize, size_usize), (isize::MAX, size_usize), (isize::MIN, size_usize + 1), (isize::MIN / 2, size_usize)])?;
    Ok(())
}

#[test]
#[rustfmt::skip]
fn test_ton_cell_num_fastnum_corner_cases() -> anyhow::Result<()> {
    // fastnum unsigned
    assert_ton_cell_num_read_write(vec![(U128::from(0u8), 128), (U128::MAX, 128)])?;
    assert_ton_cell_num_read_write(vec![(U256::from(0u8), 256), (U256::MAX, 256)])?;
    assert_ton_cell_num_read_write(vec![(U512::from(0u8), 512), (U512::MAX, 512)])?;
    let max_1023b = U1024::MAX >> 1;
    assert_ton_cell_num_read_write(vec![(U1024::from(0u8), 1023), (max_1023b, 1023)])?;

    assert_ton_cell_num_read_write(vec![(i128!(0), 128), (I128::MAX, 128), (I128::MIN, 128), (I128::MIN / I128::from(2), 128)])?;
    assert_ton_cell_num_read_write(vec![(i256!(0), 256), (I256::MAX, 256), (I256::MIN, 256), (I256::MIN / I256::from(2), 256)])?;
    assert_ton_cell_num_read_write(vec![(i512!(0), 512), (I512::MAX, 512), (I512::MIN, 512), (I512::MIN / i512!(2), 512), (I512::MIN / i512!(2), 511)])?;
    let max_1023b: I1024 = I1024::MAX >> 1;
    let min_1023b = -max_1023b - I1024::from(1u8);
    assert_ton_cell_num_read_write(vec![(i1024!(0), 1023), (max_1023b, 1023), (min_1023b, 1023)])?;
    Ok(())
}

#[test]
fn test_ton_cell_read_write_bits_repr() -> anyhow::Result<()> {
    // unsigned
    assert_ton_cell_num_5::<u8, false>()?;
    assert_ton_cell_num_5::<u16, false>()?;
    assert_ton_cell_num_5::<u32, false>()?;
    assert_ton_cell_num_5::<u64, false>()?;
    assert_ton_cell_num_5::<u128, false>()?;
    assert_ton_cell_num_5::<usize, false>()?;
    assert_ton_cell_num_5::<U256, false>()?;
    assert_ton_cell_num_5::<U512, false>()?;
    assert_ton_cell_num_5::<U1024, false>()?;
    assert_ton_cell_num_5::<BigUint, false>()?;

    // signed positive
    assert_ton_cell_num_5::<i8, true>()?;
    assert_ton_cell_num_5::<i16, true>()?;
    assert_ton_cell_num_5::<i32, true>()?;
    assert_ton_cell_num_5::<i64, true>()?;
    assert_ton_cell_num_5::<i128, true>()?;
    assert_ton_cell_num_5::<isize, true>()?;
    assert_ton_cell_num_5::<I128, true>()?;
    assert_ton_cell_num_5::<I256, true>()?;
    assert_ton_cell_num_5::<I512, true>()?;
    assert_ton_cell_num_5::<I1024, true>()?;
    assert_ton_cell_num_5::<BigInt, true>()?;

    // signed negative
    assert_ton_cell_num_signed_minus_17::<i8>()?;
    assert_ton_cell_num_signed_minus_17::<i16>()?;
    assert_ton_cell_num_signed_minus_17::<i32>()?;
    assert_ton_cell_num_signed_minus_17::<i64>()?;
    assert_ton_cell_num_signed_minus_17::<i128>()?;
    assert_ton_cell_num_signed_minus_17::<isize>()?;
    assert_ton_cell_num_signed_minus_17::<I128>()?;
    assert_ton_cell_num_signed_minus_17::<I256>()?;
    assert_ton_cell_num_signed_minus_17::<I512>()?;
    assert_ton_cell_num_signed_minus_17::<I1024>()?;
    assert_ton_cell_num_signed_minus_17::<BigInt>()?;

    assert_err!(assert_ton_cell_num_signed_minus_500::<i8>()); // value doesn't fit in i8
    assert_ton_cell_num_signed_minus_500::<i16>()?;
    assert_ton_cell_num_signed_minus_500::<i32>()?;
    assert_ton_cell_num_signed_minus_500::<i64>()?;
    assert_ton_cell_num_signed_minus_500::<i128>()?;
    assert_ton_cell_num_signed_minus_500::<isize>()?;
    assert_ton_cell_num_signed_minus_500::<I128>()?;
    assert_ton_cell_num_signed_minus_500::<I256>()?;
    assert_ton_cell_num_signed_minus_500::<I512>()?;
    assert_ton_cell_num_signed_minus_500::<I1024>()?;
    assert_ton_cell_num_signed_minus_500::<BigInt>()?;

    Ok(())
}

#[rustfmt::skip]
fn assert_ton_cell_num_5<T: TonCellNum + Debug + Clone + FromPrimitive, const SIGNED: bool>() -> anyhow::Result<()> {
    let value_parsed = match T::from_u64(5) {
        Some(v) => v,
        None => bail!("value 5 doesn't fit in type {}", type_name::<T>()),
    };
    let value = &value_parsed;

    assert_err!(assert_ton_cell_num_read_write_bits_repr(value, 1, 0, &[0]));

    if SIGNED {
        assert_eq!(value.tcn_min_bits_len(), 4);
        assert_err!(assert_ton_cell_num_read_write_bits_repr(value, 3, 0, &[0]));
        assert_ton_cell_num_read_write_bits_repr(value, 4, 0, &[0b0101_0000])?;
    } else {
        assert_eq!(value.tcn_min_bits_len(), 3);
        assert_err!(assert_ton_cell_num_read_write_bits_repr(value, 2, 0, &[0]));
        assert_ton_cell_num_read_write_bits_repr(value, 3, 0,&[0b1010_0000])?;
    }

    assert_ton_cell_num_read_write_bits_repr(value, 5, 0, &[0b0010_1000])?;
    assert_ton_cell_num_read_write_bits_repr(value, 8, 0, &[0b00000101])?;
    assert_ton_cell_num_read_write_bits_repr(value, 16, 0, &[0b00000000, 5])?;
    assert_ton_cell_num_read_write_bits_repr(value, 19, 3, &[0b11100000, 0b00000000, 0b00010100])?;

    assert_ton_cell_num_read_write_bits_repr(value, 64, 0, &[0, 0, 0, 0, 0, 0, 0, 5])?;
    assert_ton_cell_num_read_write_bits_repr(value, 63, 1, &[0b1000_0000, 0, 0, 0, 0, 0, 0, 5])?;

    assert_ton_cell_num_read_write_bits_repr(value, 128, 0, &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5])?;
    assert_ton_cell_num_read_write_bits_repr(value, 256, 0, &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5])?;
    assert_ton_cell_num_read_write_bits_repr(value, 512, 0, &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5])?;
    assert_ton_cell_num_read_write_bits_repr(value, 1023, 0, &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0b0000_1010])?;
    assert_ton_cell_num_read_write_bits_repr(value, 1022, 1, &[0b1000_0000, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0b0000_1010])?;
    Ok(())
}

#[rustfmt::skip]
fn assert_ton_cell_num_signed_minus_17<T: TonCellNum + Debug + FromPrimitive>() -> anyhow::Result<()> {
    let value_parsed = match T::from_i64(-17) {
        Some(v) => v,
        None => bail!("value -17 doesn't fit in type {}", type_name::<T>()),
    };
    let value = &value_parsed;

    assert_eq!(value.tcn_min_bits_len(), 6);
    assert_err!(assert_ton_cell_num_read_write_bits_repr(value, 5, 0,&[0]));

    assert_ton_cell_num_read_write_bits_repr(value, 6, 0, &[0b1011_1100])?;
    assert_ton_cell_num_read_write_bits_repr(value, 8, 0, &[239])?;
    assert_ton_cell_num_read_write_bits_repr(value, 16, 0, &[255, 239])?;
    assert_ton_cell_num_read_write_bits_repr(value, 19, 3, &[255, 255, 0b1011_1100])?;

    assert_ton_cell_num_read_write_bits_repr(value, 64, 0, &[255, 255, 255, 255, 255, 255, 255, 239])?;
    assert_ton_cell_num_read_write_bits_repr(value, 63, 1, &[255, 255, 255, 255, 255, 255, 255, 239])?;

    assert_ton_cell_num_read_write_bits_repr(value, 128, 0, &[255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 239])?;
    assert_ton_cell_num_read_write_bits_repr(value, 256, 0, &[255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 239])?;
    assert_ton_cell_num_read_write_bits_repr(value, 512, 0, &[255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 239])?;
    for extra_bits in 0..1 {
        assert_ton_cell_num_read_write_bits_repr(value, 1023 - extra_bits, extra_bits, &[255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0b1101_1110])?;
    }
    Ok(())
}

#[rustfmt::skip]
fn assert_ton_cell_num_signed_minus_500<T: TonCellNum + Debug + FromPrimitive>() -> anyhow::Result<()> {
    let value_parsed = match T::from_i64(-500) {
        Some(v) => v,
        None => bail!("value -500 doesn't fit in type {}", type_name::<T>()),
    };
    let value = &value_parsed;
    assert_eq!(value.tcn_min_bits_len(), 10);
    assert_err!(assert_ton_cell_num_read_write_bits_repr(value, 9, 0,&[0]));

    assert_ton_cell_num_read_write_bits_repr(value, 10, 0, &[0b10000011, 0])?;
    assert_ton_cell_num_read_write_bits_repr(value, 16, 0, &[0b11111110, 0b00001100])?;
    assert_ton_cell_num_read_write_bits_repr(value, 19, 3, &[0b11111111, 0b11111000, 0b00110000])?;

    assert_ton_cell_num_read_write_bits_repr(value, 64, 0, &[255, 255, 255, 255, 255, 255, 0b11111110, 0b00001100])?;
    assert_ton_cell_num_read_write_bits_repr(value, 63, 1, &[255, 255, 255, 255, 255, 255, 0b11111110, 0b00001100])?;

    assert_ton_cell_num_read_write_bits_repr(value, 128, 0, &[255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0b11111110, 0b00001100])?;
    assert_ton_cell_num_read_write_bits_repr(value, 256, 0, &[255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0b11111110, 0b00001100])?;
    assert_ton_cell_num_read_write_bits_repr(value, 512, 0, &[255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0b11111110, 0b00001100])?;
    for extra_bits in 0..1 {
        assert_ton_cell_num_read_write_bits_repr(value, 1023 - extra_bits, extra_bits, &[255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0b11111100, 0b00011000])?;
    }
    Ok(())
}

// checks reading/writing and bit representation in the cell
fn assert_ton_cell_num_read_write_bits_repr<T: TonCellNum + Debug + Clone>(
    value: &T,
    bits_len: usize,
    extra_bits: usize,       // write N extra bits before the value to test alignment
    expected_storage: &[u8], // won't be checked if not specified
) -> anyhow::Result<()> {
    let assert_info = format!("{:?} ({}), bits_len {}, extra_bits {}", value, type_name::<T>(), bits_len, extra_bits);

    let mut builder = TonCell::builder();
    for _bit in 0..extra_bits {
        builder.write_bit(true)?;
    }
    builder.write_num(value, bits_len)?;
    let cell = builder.build()?;
    assert_eq!(bits_len + extra_bits, cell.data_len_bits(), "bits_len mismatch for {assert_info}");

    let mut parser = cell.parser();
    let _ = parser.read_bits(extra_bits)?;
    let parsed_value = parser.read_num::<T>(bits_len)?;
    assert_eq!(value, &parsed_value, "value mismatch for {assert_info}");
    assert_ok!(parser.ensure_empty(), "ensure_empty failed for {assert_info}");

    assert_eq!(expected_storage, cell.underlying_storage(), "wrong storage for {assert_info}");

    Ok(())
}

fn assert_ton_cell_num_read_write<T: TonCellNum + Debug>(input: Vec<(T, usize)>) -> anyhow::Result<()> {
    for (value, bits_len) in input {
        let mut builder = TonCell::builder();
        builder.write_num(&value, bits_len)?;
        let cell = builder.build()?;
        let mut parser = CellParser::new(&cell);
        let parsed = parser.read_num::<T>(bits_len)?;
        assert_eq!(parsed, value, "Failed for {}: value {value:?}, bits_len {bits_len}", type_name::<T>());
    }
    Ok(())
}
