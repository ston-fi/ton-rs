use crate::bail_ton_core_data;
use crate::errors::TonCoreError;

/// Bounds-checked cursor over serialized BoC bytes.
///
/// Positions are absolute offsets into the whole input, so parsed cells can
/// point into the shared BoC storage.
pub(super) struct BocReader<'a> {
    data: &'a [u8],
    pos: usize,
    end: usize,
}

impl<'a> BocReader<'a> {
    pub(super) fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            pos: 0,
            end: data.len(),
        }
    }

    pub(super) fn position(&self) -> usize {
        self.pos
    }

    pub(super) fn remaining(&self) -> usize {
        self.end - self.pos
    }

    pub(super) fn take(&mut self, len: usize) -> Result<&'a [u8], TonCoreError> {
        let Some(bytes) = self.data.get(self.pos..self.end).and_then(|rest| rest.get(..len)) else {
            bail_ton_core_data!(
                "Unexpected end of BoC: need {len} bytes at offset {}, {} left",
                self.pos,
                self.remaining()
            );
        };
        self.pos += len;
        Ok(bytes)
    }

    pub(super) fn read_u8(&mut self) -> Result<u8, TonCoreError> {
        Ok(be_uint(self.take(1)?) as u8)
    }

    /// Reads a big-endian unsigned integer of `len <= 8` bytes.
    pub(super) fn read_uint(&mut self, len: usize) -> Result<u64, TonCoreError> {
        Ok(be_uint(self.take(len)?))
    }

    /// Moves the next `len` bytes into a separate reader.
    pub(super) fn split(&mut self, len: usize) -> Result<BocReader<'a>, TonCoreError> {
        let start = self.pos;
        self.take(len)?;
        Ok(BocReader {
            data: self.data,
            pos: start,
            end: self.pos,
        })
    }
}

/// Decodes a big-endian unsigned integer of at most 8 bytes.
pub(super) fn be_uint(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0, |acc, &byte| (acc << 8) | u64::from(byte))
}
