#[cfg(test)]
#[path = "../tests/cell/boc/test_boc_parsing.rs"]
mod _test_boc_parsing;
mod boc_reader;
pub(crate) mod raw_boc;
pub(crate) mod raw_cell;

use crate::bail_ton_core_data;
use crate::cell::TonCell;
use crate::cell::boc::raw_boc::RawBoC;
use crate::errors::TonCoreError;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use std::sync::Arc;

pub struct BoC {
    roots: Vec<TonCell>,
}

impl BoC {
    pub fn new(root: TonCell) -> Self {
        Self { roots: vec![root] }
    }
    pub fn from_roots<I>(roots: I) -> Self
    where
        I: IntoIterator<Item = TonCell>,
    {
        Self {
            roots: roots.into_iter().collect(),
        }
    }

    /// Parses a serialized bag of cells. The input may be untrusted.
    ///
    /// # Errors
    ///
    /// Returns an error unless the input is a single BoC, with no trailing bytes, that the TON node
    /// would accept: header sizes must match the input, the index and CRC32C must match when
    /// present, exotic cells must have valid layouts and Merkle hashes, level masks must match
    /// the cells, and no cell may be deeper than 1024.
    pub fn from_bytes<T: Into<Arc<Vec<u8>>>>(bytes: T) -> Result<Self, TonCoreError> {
        let bytes_ptr = bytes.into();
        if bytes_ptr.is_empty() {
            bail_ton_core_data!("Can't read BOC from empty slice");
        }
        Ok(Self {
            roots: RawBoC::from_bytes(bytes_ptr)?.into_verified_ton_cells()?,
        })
    }

    pub fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, TonCoreError> {
        Self::from_bytes(hex::decode(hex.as_ref())?)
    }
    pub fn from_base64<T: AsRef<[u8]>>(hex: T) -> Result<Self, TonCoreError> {
        Self::from_bytes(BASE64_STANDARD.decode(hex.as_ref())?)
    }

    pub fn to_bytes(&self, add_crc32: bool) -> Result<Vec<u8>, TonCoreError> {
        RawBoC::from_ton_cells(self.roots.as_slice(), false)?.to_bytes(add_crc32)
    }
    pub fn to_hex(&self, add_crc32: bool) -> Result<String, TonCoreError> {
        Ok(hex::encode(self.to_bytes(add_crc32)?))
    }
    pub fn to_base64(&self, add_crc32: bool) -> Result<String, TonCoreError> {
        Ok(BASE64_STANDARD.encode(self.to_bytes(add_crc32)?))
    }

    // zero-based index
    pub fn get_root(&self, index: usize) -> Option<&TonCell> {
        self.roots.get(index)
    }

    pub fn single_root(mut self) -> Result<TonCell, TonCoreError> {
        if self.roots.len() != 1 {
            bail_ton_core_data!("Expected 1 root cell, got {}", self.roots.len());
        }
        Ok(self.roots.pop().unwrap())
    }
}

#[cfg(test)]
#[path = "../tests/cell/test_boc.rs"]
mod tests;
