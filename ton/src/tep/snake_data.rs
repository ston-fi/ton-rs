use std::borrow::Cow;
use std::cmp::min;
use std::str::FromStr;
use ton_core::bail_ton_core_data;
use ton_core::cell::{CellBuilder, CellParser, TonCell};
use ton_core::errors::TonCoreError;
use ton_core::traits::tlb::TLB;

// https://docs.ton.org/v3/guidelines/dapps/asset-processing/nft-processing/metadata-parsing#snake-data-encoding

/// support only bytes-aligned data
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnakeData {
    pub data: Vec<u8>,
    pub chunks_bits_len: Vec<usize>,
}

#[rustfmt::skip]
impl SnakeData {
    pub fn new(data: Vec<u8>) -> Self { Self { data, chunks_bits_len: vec![] } }
    pub fn as_str(&self) -> Cow<'_, str> {
        if self.data.is_empty() {
            return Cow::Borrowed("");
        }
        if self.data[0] == 0 {
            return String::from_utf8_lossy(&self.data[1..]);
        }
        String::from_utf8_lossy(&self.data)
    }
    pub fn as_slice(&self) -> &[u8] { &self.data }
}

impl FromStr for SnakeData {
    type Err = TonCoreError;
    fn from_str(s: &str) -> Result<Self, Self::Err> { Ok(Self::from(s)) }
}

impl From<&str> for SnakeData {
    fn from(s: &str) -> Self { SnakeData::new(s.as_bytes().to_vec()) }
}

impl TLB for SnakeData {
    fn read_definition(parser: &mut CellParser) -> Result<Self, TonCoreError> {
        let mut result = SnakeData {
            data: vec![],
            chunks_bits_len: vec![],
        };
        result.read_chunk(parser)?;

        let mut maybe_next_ref = parser.read_next_ref().cloned();
        while let Ok(next_ref) = maybe_next_ref {
            let mut cur_parser = next_ref.parser();
            result.read_chunk(&mut cur_parser)?;
            maybe_next_ref = cur_parser.read_next_ref().cloned();
        }
        Ok(result)
    }

    fn write_definition(&self, builder: &mut CellBuilder) -> Result<(), TonCoreError> {
        let data_bits_len = if self.chunks_bits_len.is_empty() {
            self.data.len() * 8
        } else {
            self.chunks_bits_len.iter().sum()
        };
        if data_bits_len > self.data.len() * 8 {
            bail_ton_core_data!(
                "SnakeData chunks contain {data_bits_len} bits, but data contains only {} bits",
                self.data.len() * 8
            );
        }
        self.write_chunk(builder, 0, data_bits_len, &self.chunks_bits_len)
    }
}

impl SnakeData {
    fn read_chunk(&mut self, parser: &mut CellParser) -> Result<(), TonCoreError> {
        let cur_cell_bits_len = parser.data_bits_left()?;
        if cur_cell_bits_len == 0 {
            return Ok(());
        }
        let Some(prev_bits_len) = self.chunks_bits_len.last() else {
            self.data.extend(parser.read_bits(cur_cell_bits_len)?);
            self.chunks_bits_len.push(cur_cell_bits_len);
            return Ok(());
        };

        let last_byte_filled_bits = prev_bits_len % 8;
        let last_byte_free_bits = (8 - prev_bits_len % 8) % 8;
        if last_byte_free_bits >= cur_cell_bits_len {
            let bits = parser.read_bits(cur_cell_bits_len)?[0];
            *self.data.last_mut().unwrap() |= bits >> (last_byte_filled_bits); // definetely have something in data as chunks_bits_len.last() != None
            self.chunks_bits_len.push(cur_cell_bits_len);

            return Ok(());
        }

        if last_byte_free_bits > 0 {
            let bits = parser.read_bits(last_byte_free_bits)?[0];
            *self.data.last_mut().unwrap() |= bits >> (last_byte_filled_bits); // filling last unfilled byte
        }

        self.data.extend(parser.read_bits(cur_cell_bits_len - last_byte_free_bits)?);
        self.chunks_bits_len.push(cur_cell_bits_len);

        dbg!(&self.data);
        dbg!(&self.chunks_bits_len);
        Ok(())
    }

    fn write_chunk(
        &self,
        builder: &mut CellBuilder,
        bits_offset: usize,
        data_bits_len: usize,
        chunks_bits_len: &[usize],
    ) -> Result<(), TonCoreError> {
        if bits_offset == data_bits_len {
            return Ok(());
        }

        let bits_to_write = if let Some(chunk_bits_len) = chunks_bits_len.first() {
            *chunk_bits_len
        } else {
            min(data_bits_len - bits_offset, builder.data_bits_left())
        };
        if bits_to_write > data_bits_len - bits_offset {
            bail_ton_core_data!(
                "SnakeData chunk contains {bits_to_write} bits, but only {} data bits left",
                data_bits_len - bits_offset
            );
        }

        builder.write_bits_with_offset(&self.data, bits_offset, bits_to_write)?;
        if bits_offset + bits_to_write == data_bits_len {
            return Ok(());
        }

        let chunks_bits_len_rest = if chunks_bits_len.len() > 1 {
            &chunks_bits_len[1..]
        } else {
            &[]
        };
        let mut child_builder = TonCell::builder();
        self.write_chunk(&mut child_builder, bits_offset + bits_to_write, data_bits_len, chunks_bits_len_rest)?;
        builder.write_ref(child_builder.build()?)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use ton_core::cell::TonCell;
    use ton_core::traits::tlb::TLB;

    use crate::tep::snake_data::SnakeData;

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
        let s = "snake-data-string-roundtrip-".repeat(8);
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
}
