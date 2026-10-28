//! Snake-format byte strings used by TEP metadata.

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
    /// Creates byte-aligned snake data with chunk boundaries chosen during serialization.
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
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from(s))
    }
}

impl From<&str> for SnakeData {
    fn from(s: &str) -> Self {
        SnakeData::new(s.as_bytes().to_vec())
    }
}

impl TLB for SnakeData {
    fn read_definition(parser: &mut CellParser) -> Result<Self, TonCoreError> {
        let mut result = SnakeData {
            data: vec![],
            chunks_bits_len: vec![],
        };
        let mut bits_read = 0;
        result.read_chunk(parser, &mut bits_read)?;

        let mut maybe_next_ref = parser.read_next_ref().cloned();
        while let Ok(next_ref) = maybe_next_ref {
            let mut cur_parser = next_ref.parser();
            result.read_chunk(&mut cur_parser, &mut bits_read)?;
            maybe_next_ref = cur_parser.read_next_ref().cloned();
        }
        Ok(result)
    }

    fn write_definition(&self, builder: &mut CellBuilder) -> Result<(), TonCoreError> {
        let data_bits_len =
            if self.chunks_bits_len.is_empty() { self.data.len() * 8 } else { self.chunks_bits_len.iter().sum() };
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
    fn read_chunk(&mut self, parser: &mut CellParser, bits_read: &mut usize) -> Result<(), TonCoreError> {
        let cur_cell_bits_len = parser.data_bits_left()?;
        if cur_cell_bits_len == 0 {
            return Ok(());
        }
        if self.chunks_bits_len.is_empty() {
            self.data.extend(parser.read_bits(cur_cell_bits_len)?);
            self.chunks_bits_len.push(cur_cell_bits_len);
            *bits_read += cur_cell_bits_len;
            return Ok(());
        }

        let last_byte_filled_bits = *bits_read % 8;
        let last_byte_free_bits = (8 - last_byte_filled_bits) % 8;
        if last_byte_free_bits >= cur_cell_bits_len {
            let bits = parser.read_bits(cur_cell_bits_len)?[0];
            *self.data.last_mut().unwrap() |= bits >> (last_byte_filled_bits); // definetely have something in data as chunks_bits_len.last() != None
            self.chunks_bits_len.push(cur_cell_bits_len);
            *bits_read += cur_cell_bits_len;

            return Ok(());
        }

        if last_byte_free_bits > 0 {
            let bits = parser.read_bits(last_byte_free_bits)?[0];
            *self.data.last_mut().unwrap() |= bits >> (last_byte_filled_bits); // filling last unfilled byte
        }

        self.data.extend(parser.read_bits(cur_cell_bits_len - last_byte_free_bits)?);
        self.chunks_bits_len.push(cur_cell_bits_len);
        *bits_read += cur_cell_bits_len;

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

        let chunks_bits_len_rest = if chunks_bits_len.len() > 1 { &chunks_bits_len[1..] } else { &[] };
        let mut child_builder = TonCell::builder();
        self.write_chunk(&mut child_builder, bits_offset + bits_to_write, data_bits_len, chunks_bits_len_rest)?;
        builder.write_ref(child_builder.build()?)
    }
}

#[cfg(test)]
#[path = "../../tests/contracts/tep/test_snake_data.rs"]
mod tests;
