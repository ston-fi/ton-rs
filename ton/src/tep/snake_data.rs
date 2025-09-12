use std::borrow::Cow;
use std::cmp::min;
use std::str::FromStr;
use ton_lib_core::bail_ton_core_data;
use ton_lib_core::cell::{CellBuilder, CellParser, TonCell};
use ton_lib_core::errors::TonCoreError;
use ton_lib_core::traits::tlb::TLB;

// https://docs.ton.org/v3/guidelines/dapps/asset-processing/nft-processing/metadata-parsing#snake-data-encoding

/// support only bytes-aligned data
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnakeData {
    pub data: Vec<u8>,
    pub chunks_len: Vec<usize>,
}

#[rustfmt::skip]
impl SnakeData {
    pub fn new(data: Vec<u8>) -> Self { Self { data, chunks_len: vec![] } }
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
    fn from_str(s: &str) -> Result<Self, Self::Err> { Ok(SnakeData::new(s.as_bytes().to_vec())) }
}

impl TLB for SnakeData {
    fn read_definition(parser: &mut CellParser) -> Result<Self, TonCoreError> {
        let mut result = SnakeData {
            data: vec![],
            chunks_len: vec![],
        };
        Self::read_chunk(parser, &mut result)?;

        let mut maybe_next_ref = parser.read_next_ref().cloned();
        while let Ok(next_ref) = maybe_next_ref {
            let mut cur_parser = next_ref.parser();
            Self::read_chunk(&mut cur_parser, &mut result)?;
            maybe_next_ref = cur_parser.read_next_ref().cloned();
        }
        Ok(result)
    }

    fn write_definition(&self, builder: &mut CellBuilder) -> Result<(), TonCoreError> {
        Self::write_chunk(builder, &self.data, &self.chunks_len)
    }
}

impl SnakeData {
    fn read_chunk(parser: &mut CellParser, result: &mut Self) -> Result<(), TonCoreError> {
        let chunk_bits_len = parser.data_bits_remaining()?;
        if chunk_bits_len % 8 != 0 {
            bail_ton_core_data!("Expecting data_chunk_len % 8 == 0, got {chunk_bits_len}");
        }
        result.data.extend(parser.read_bits(chunk_bits_len)?);
        result.chunks_len.push(chunk_bits_len / 8);
        Ok(())
    }

    fn write_chunk(builder: &mut CellBuilder, data: &[u8], chunks_len: &[usize]) -> Result<(), TonCoreError> {
        let bytes_to_write = if chunks_len.is_empty() {
            min(data.len(), builder.data_bits_left() / 8)
        } else {
            chunks_len[0]
        };

        let chunk = &data[0..bytes_to_write];
        builder.write_bits(chunk, chunk.len() * 8)?;
        if bytes_to_write == data.len() {
            return Ok(());
        }

        let chunks_len_rest = if chunks_len.len() > 1 { &chunks_len[1..] } else { &[] };
        let mut child_builder = TonCell::builder();
        Self::write_chunk(&mut child_builder, &data[bytes_to_write..], chunks_len_rest)?;
        builder.write_ref(child_builder.build_ref()?)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use ton_lib_core::cell::TonCell;
    use ton_lib_core::traits::tlb::TLB;

    use crate::tep::snake_data::SnakeData;

    #[test]
    fn test_snake_data() -> anyhow::Result<()> {
        let mut builder2 = TonCell::builder();
        builder2.write_bits([0b10101010; 64], 512)?;
        let child2 = builder2.build_ref()?;

        let mut builder1 = TonCell::builder();
        builder1.write_bits([0b01010101; 64], 512)?;
        builder1.write_ref(child2)?;
        let child1 = builder1.build_ref()?;

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
        assert_eq!(parser.data_bits_remaining()?, 63 * 8);
        assert_eq!(parser.read_bits(63 * 8)?, vec![0b11111111; 63]);

        // just in case - write to empty cell
        let cell = snake_data.to_cell()?;
        assert_eq!(cell.data.len(), 127);
        assert_eq!(cell.data_bits_len, 1016);
        assert_eq!(cell.refs[0].data.len(), 1);
        assert_eq!(cell.refs[0].data_bits_len, 8);

        // from_str

        assert_eq!(SnakeData::from_str("my awesome snakedata")?.as_str(), "my awesome snakedata");

        Ok(())
    }
}
