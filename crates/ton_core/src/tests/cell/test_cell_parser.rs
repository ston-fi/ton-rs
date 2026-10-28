use super::*;
use crate::cell::TonCell;
use num_bigint::{BigInt, BigUint};
use tokio_test::{assert_err, assert_ok};

fn make_test_cell(data: &[u8], bits_len: usize) -> anyhow::Result<TonCell> {
    let mut builder = TonCell::builder();
    builder.write_bits(data, bits_len)?;
    Ok(builder.build()?)
}

#[test]
fn test_parser_seek_bits() -> anyhow::Result<()> {
    let cell = make_test_cell(&[0b10101001, 0b01010100], 10)?;
    let mut parser = CellParser::new(&cell);
    assert_ok!(parser.seek_bits(3));
    assert_eq!(parser.data_reader.position_in_bits()? as usize, 3);
    assert_ok!(parser.seek_bits(-2));
    assert_eq!(parser.data_reader.position_in_bits()? as usize, 1);
    assert_ok!(parser.seek_bits(0));
    assert_eq!(parser.data_reader.position_in_bits()? as usize, 1);
    assert_ok!(parser.seek_bits(-1));
    assert_eq!(parser.data_reader.position_in_bits()? as usize, 0);
    assert_err!(parser.seek_bits(-1));
    assert_eq!(parser.data_reader.position_in_bits()? as usize, 0);
    assert_ok!(parser.seek_bits(cell.borders.end_bit as i32 - 1));
    assert_eq!(parser.data_reader.position_in_bits()? as usize, cell.borders.end_bit - 1);
    assert_ok!(parser.seek_bits(1));
    assert_eq!(parser.data_reader.position_in_bits()? as usize, cell.borders.end_bit);
    assert_err!(parser.seek_bits(1));
    assert_err!(parser.seek_bits(20));
    Ok(())
}

#[test]
fn test_parser_lookup_bits() -> anyhow::Result<()> {
    let cell = make_test_cell(&[0b10101010, 0b01010101], 16)?;
    let mut parser = CellParser::new(&cell);
    assert_eq!(parser.lookup_bits(3)?, 0b101);
    assert_eq!(parser.data_reader.position_in_bits()?, 0);
    assert!(assert_ok!(parser.read_bit()));
    assert_eq!(parser.data_reader.position_in_bits()?, 1);
    assert_eq!(parser.lookup_bits(3)?, 0b010);
    assert_eq!(parser.data_reader.position_in_bits()?, 1);
    Ok(())
}

#[test]
fn test_parser_read_bit() -> anyhow::Result<()> {
    let cell = make_test_cell(&[0b10101010, 0b01010101], 16)?;
    let mut parser = CellParser::new(&cell);
    for i in 0..8 {
        assert_eq!(assert_ok!(parser.read_bit()), i % 2 == 0);
    }
    for i in 0..8 {
        assert_eq!(assert_ok!(parser.read_bit()), i % 2 != 0);
    }
    Ok(())
}

#[test]
fn test_parser_ensure_enough_bits() -> anyhow::Result<()> {
    let cell = make_test_cell(&[0b10101010, 0b01010101], 10)?;
    let mut parser = CellParser::new(&cell);
    assert_eq!(parser.data_reader.position_in_bits()?, 0);
    assert_ok!(parser.ensure_enough_bits(0));
    assert_ok!(parser.ensure_enough_bits(1));
    assert_ok!(parser.ensure_enough_bits(6));
    assert_ok!(parser.ensure_enough_bits(10));
    assert_err!(parser.ensure_enough_bits(11));
    Ok(())
}

#[test]
fn test_parser_read_ref() -> anyhow::Result<()> {
    let mut ref_builder = TonCell::builder();
    ref_builder.write_num(&0b11110000u8, 8)?;
    let cell1 = ref_builder.build()?;

    let mut cell_builder = TonCell::builder();
    cell_builder.write_ref(cell1.clone())?;
    cell_builder.write_ref(cell1.clone())?;
    let cell_2 = cell_builder.build()?;

    let mut parser = CellParser::new(&cell_2);
    assert_eq!(parser.read_next_ref()?, &cell1);
    assert_eq!(parser.read_next_ref()?, &cell1);
    assert!(parser.read_next_ref().is_err());
    Ok(())
}

#[test]
fn test_parser_read_bits() -> anyhow::Result<()> {
    let cell = make_test_cell(&[0b10101010u8, 0b01010101u8], 16)?;
    let mut parser = CellParser::new(&cell);
    let dst = parser.read_bits(3)?;
    assert_eq!(dst, [0b10100000]);
    let dst = parser.read_bits(6)?;
    assert_eq!(dst, [0b01010000]);
    Ok(())
}

#[test]
fn test_parser_read_num() -> anyhow::Result<()> {
    let cell = make_test_cell(&[0b10101010, 0b01010101], 16)?;
    let mut parser = CellParser::new(&cell);
    assert_eq!(parser.read_num::<u8>(3)?, 0b101);
    assert_eq!(parser.data_reader.position_in_bits()?, 3);
    assert_eq!(parser.read_num::<u32>(3)?, 0b010);
    assert_eq!(parser.data_reader.position_in_bits()?, 6);
    assert_eq!(parser.read_num::<u64>(3)?, 0b100);
    assert_eq!(parser.data_reader.position_in_bits()?, 9);
    Ok(())
}

#[test]
fn test_parser_read_num_unaligned() -> anyhow::Result<()> {
    let cell = make_test_cell(&[0b0001_0001, 0b0000_0000, 0b1010_0000], 19)?;
    let mut parser = CellParser::new(&cell);
    assert_eq!(parser.read_num::<u8>(4)?, 1);
    assert_eq!(parser.data_reader.position_in_bits()?, 4);
    assert_eq!(parser.read_num::<u16>(5)?, 2);
    assert_eq!(parser.data_reader.position_in_bits()?, 9);
    assert_eq!(parser.read_num::<u32>(10)?, 5);
    assert_eq!(parser.data_reader.position_in_bits()?, 19);
    Ok(())
}

#[test]
fn test_parser_read_cell() -> anyhow::Result<()> {
    let mut builder = TonCell::builder();
    builder.write_bits([255, 0, 255, 0], 24)?;

    for i in 0..3 {
        let mut ref_builder = TonCell::builder();
        ref_builder.write_bits([i], 8)?;
        builder.write_ref(ref_builder.build()?)?;
    }

    let orig_cell = builder.build()?;
    let mut parser = CellParser::new(&orig_cell);
    parser.read_bits(4)?; // skip 4 bits
    parser.read_next_ref()?; // skip first ref

    let cell = parser.read_remaining()?;
    let expected_borders = CellBorders {
        start_bit: 4,
        end_bit: orig_cell.borders.end_bit,
        start_ref: 1,
        end_ref: orig_cell.borders.end_ref,
    };
    assert_eq!(cell.borders, expected_borders);
    Ok(())
}

#[test]
fn test_parser_read_slice() -> anyhow::Result<()> {
    let mut builder = TonCell::builder();
    builder.write_bits([255, 0, 255, 0], 24)?;

    for i in 0..3 {
        let mut ref_builder = TonCell::builder();
        ref_builder.write_bits([i], 8)?;
        builder.write_ref(ref_builder.build()?)?;
    }

    let orig_cell = builder.build()?;
    let mut parser = CellParser::new(&orig_cell);
    parser.read_bits(4)?; // skip 4 bits
    parser.read_next_ref()?; // skip first ref

    let cell = parser.read_cell(2, 0)?;
    let expected_borders = CellBorders {
        start_bit: 4,
        end_bit: 6,
        start_ref: 1,
        end_ref: 1,
    };
    assert_eq!(cell.borders, expected_borders);
    Ok(())
}

#[test]
fn test_parser_read_bigint() -> anyhow::Result<()> {
    let cell = make_test_cell(&[0b111_01010, 0b01101011, 0b10000000, 0b00000001], 32)?;
    let mut parser = CellParser::new(&cell);
    assert_eq!(parser.read_num::<BigInt>(3)?, (-1).into());
    assert_eq!(parser.data_reader.position_in_bits()?, 3);
    assert_eq!(parser.read_num::<BigInt>(5)?, 10.into()); // finish with first byte
    assert_eq!(parser.data_reader.position_in_bits()?, 8);
    parser.read_bit()?; // skip 1 bit
    assert_eq!(parser.read_num::<BigInt>(7)?, (-21).into()); // finish with second byte
    assert_eq!(parser.data_reader.position_in_bits()?, 16);
    assert_eq!(parser.read_num::<BigInt>(16)?, (-32767).into());
    Ok(())
}

#[test]
fn test_parser_read_bigint_unaligned() -> anyhow::Result<()> {
    let cell = make_test_cell(&[0b00011110, 0b11111111], 16)?;
    let mut parser = CellParser::new(&cell);
    parser.seek_bits(3)?;
    assert_eq!(parser.read_num::<BigInt>(9)?, (-17).into());
    Ok(())
}

#[test]
fn test_parser_read_biguint() -> anyhow::Result<()> {
    let cell_slice = make_test_cell(&[0b10101010, 0b01010101, 0b11111111, 0b11111111], 32)?;
    let mut parser = CellParser::new(&cell_slice);
    assert_eq!(parser.read_num::<BigUint>(3)?, 5u32.into());
    assert_eq!(parser.data_reader.position_in_bits()?, 3);
    assert_eq!(parser.read_num::<BigUint>(5)?, 10u32.into()); // finish with first byte
    assert_eq!(parser.data_reader.position_in_bits()?, 8);
    parser.read_bit()?; // skip 1 bit
    assert_eq!(parser.read_num::<BigUint>(7)?, 85u32.into()); // finish with second byte
    assert_eq!(parser.data_reader.position_in_bits()?, 16);
    assert_eq!(parser.read_num::<BigUint>(16)?, 65535u32.into());
    Ok(())
}

#[test]
fn test_parser_ensure_empty() -> anyhow::Result<()> {
    let cell_ref = make_test_cell(&[0b10101010, 0b01010101], 16)?;
    let mut builder = TonCell::builder();
    builder.write_ref(cell_ref)?;
    builder.write_num(&3, 3)?;
    let cell = builder.build()?;

    let mut parser = CellParser::new(&cell);
    assert_err!(parser.ensure_empty());
    parser.read_bits(3)?;
    assert_err!(parser.ensure_empty());
    parser.read_next_ref()?;
    assert_ok!(parser.ensure_empty());
    Ok(())
}
