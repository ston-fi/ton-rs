use std::str::FromStr;

use ton_core::cell::TonCell;
use ton_core::traits::tlb::TLB;

use crate::contracts::tep::snake_data::SnakeData;

#[test]
fn test_snake_data() -> anyhow::Result<()> {
    let mut builder2 = TonCell::builder();
    builder2.write_bits([0b10101010; 64], 512)?;
    let child2 = builder2.build()?;

    let mut builder1 = TonCell::builder();
    builder1.write_bits([0b01010101; 64], 512)?;
    builder1.write_ref(child2)?;
    let child1 = builder1.build()?;

    let mut builder = TonCell::builder();
    builder.write_bits([0b00000000; 64], 512)?;
    builder.write_ref(child1)?;
    let cell = builder.build()?;

    let mut expected = vec![0b00000000; 64];
    expected.extend(vec![0b01010101; 64]);
    expected.extend(vec![0b10101010; 64]);

    let parsed_no_prefix = SnakeData::from_cell(&cell)?;
    assert_eq!(parsed_no_prefix.data, expected);
    let serialized = parsed_no_prefix.to_cell()?;
    assert_eq!(serialized, cell);

    // test serialization fill all available bits in cell by default
    let snake_data = SnakeData::new(vec![0b11111111; 128]); // 1024 bits
    let mut builder = TonCell::builder();
    builder.write_bits([0b00000000; 64], 512)?;
    snake_data.write(&mut builder)?;

    let cell = builder.build()?;
    let mut parser = cell.parser();
    let _ = parser.read_bits(512); // skip
    assert_eq!(parser.data_bits_left()?, 511);
    let mut expected_bits = vec![0b11111111; 63];
    expected_bits.push(0b11111110);
    assert_eq!(parser.read_bits(511)?, expected_bits);

    // just in case - write to empty cell
    let cell = snake_data.to_cell()?;
    assert_eq!(cell.data_len_bits(), 1023);
    assert_eq!(cell.refs()[0].data_len_bits(), 1);

    // from_str

    assert_eq!(SnakeData::from_str("my awesome snakedata")?.as_str(), "my awesome snakedata");

    Ok(())
}

#[test]
fn test_snake_data_read_definition_two_unaligned_cells() -> anyhow::Result<()> {
    let mut child_builder = TonCell::builder();
    child_builder.write_bits([0b0111_0010, 0b1100_0000], 11)?;
    let child = child_builder.build()?;

    let mut builder = TonCell::builder();
    builder.write_bits([0b1010_1100, 0b1110_1000], 13)?;
    builder.write_ref(child)?;
    let cell = builder.build()?;

    assert_eq!(cell.data_len_bits(), 13);
    assert_eq!(cell.refs().len(), 1);
    assert_eq!(cell.refs()[0].data_len_bits(), 11);

    let mut parser = cell.parser();
    let parsed = SnakeData::read_definition(&mut parser)?;
    assert_eq!(parsed.data, vec![0b1010_1100, 0b1110_1011, 0b1001_0110]);
    assert_eq!(parsed.chunks_bits_len, vec![13, 11]);

    Ok(())
}

#[test]
fn test_snake_data_read_definition_three_unaligned_cells() -> anyhow::Result<()> {
    let mut last_child_builder = TonCell::builder();
    last_child_builder.write_bits([0b1010_0000], 3)?;
    let last_child = last_child_builder.build()?;

    let mut child_builder = TonCell::builder();
    child_builder.write_bits([0b0011_1100], 8)?;
    child_builder.write_ref(last_child)?;
    let child = child_builder.build()?;

    let mut builder = TonCell::builder();
    builder.write_bits([0b1101_1000], 5)?;
    builder.write_ref(child)?;
    let cell = builder.build()?;

    let mut parser = cell.parser();
    let parsed = SnakeData::read_definition(&mut parser)?;

    assert_eq!(parsed.data, vec![0b1101_1001, 0b1110_0101]);
    assert_eq!(parsed.chunks_bits_len, vec![5, 8, 3]);

    Ok(())
}

#[test]
fn test_snake_data_write_definition_two_unaligned_cells() -> anyhow::Result<()> {
    let snake_data = SnakeData {
        data: vec![0b1010_1100, 0b1110_1011, 0b1001_0110],
        chunks_bits_len: vec![13, 11],
    };

    let cell = snake_data.to_cell()?;

    assert_eq!(cell.data_len_bits(), 13);
    assert_eq!(cell.refs().len(), 1);
    assert_eq!(cell.refs()[0].data_len_bits(), 11);
    assert!(cell.refs()[0].refs().is_empty());

    let mut parser = cell.parser();
    assert_eq!(parser.read_bits(13)?, vec![0b1010_1100, 0b1110_1000]);

    let mut child_parser = cell.refs()[0].parser();
    assert_eq!(child_parser.read_bits(11)?, vec![0b0111_0010, 0b1100_0000]);

    Ok(())
}

#[test]
fn test_snake_data_string_roundtrip_across_cells() -> anyhow::Result<()> {
    let s = "snake-data-string-roundtrip-".repeat(40);
    let snake_data = SnakeData::from_str(&s)?;

    let cell = snake_data.to_cell()?;
    let parsed = SnakeData::from_cell(&cell)?;
    assert_eq!(parsed.as_str(), s);
    assert_eq!(parsed.data, s.as_bytes());

    Ok(())
}

#[test]
fn test_snake_data_from_str() -> anyhow::Result<()> {
    let s = "Hello, SnakeData!";
    let snake_data = SnakeData::from_str(s)?;
    assert_eq!(snake_data.as_str(), s);
    Ok(())
}
