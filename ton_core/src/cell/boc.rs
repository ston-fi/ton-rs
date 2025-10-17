mod raw_boc;
mod raw_cell;
mod read_var_size;

use crate::bail_ton_core_data;
use crate::cell::boc::raw_boc::RawBoC;
use crate::cell::TonCell;
use crate::errors::TonCoreError;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use std::sync::Arc;

pub struct BoC {
    roots: Vec<TonCell>,
}

impl BoC {
    pub fn new(root: TonCell) -> Self { Self { roots: vec![root] } }
    pub fn from_roots<I>(roots: I) -> Self
    where
        I: IntoIterator<Item = TonCell>,
    {
        Self {
            roots: roots.into_iter().collect(),
        }
    }

    pub fn from_bytes<T: Into<Arc<Vec<u8>>>>(bytes: T) -> Result<Self, TonCoreError> {
        let bytes_ptr = bytes.into();
        if bytes_ptr.is_empty() {
            bail_ton_core_data!("Can't read BOC from empty slice");
        }
        Ok(Self {
            roots: RawBoC::from_bytes(bytes_ptr)?.into_ton_cells()?,
        })
    }

    pub fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, TonCoreError> {
        Self::from_bytes(hex::decode(hex.as_ref())?)
    }
    pub fn from_base64<T: AsRef<[u8]>>(hex: T) -> Result<Self, TonCoreError> {
        Self::from_bytes(BASE64_STANDARD.decode(hex.as_ref())?)
    }

    pub fn to_bytes(&self, add_crc32: bool) -> Result<Vec<u8>, TonCoreError> {
        RawBoC::from_ton_cells(self.roots.as_slice())?.to_bytes(add_crc32)
    }
    pub fn to_hex(&self, add_crc32: bool) -> Result<String, TonCoreError> { Ok(hex::encode(self.to_bytes(add_crc32)?)) }
    pub fn to_base64(&self, add_crc32: bool) -> Result<String, TonCoreError> {
        Ok(BASE64_STANDARD.encode(self.to_bytes(add_crc32)?))
    }

    // zero-based index
    pub fn get_root(&self, index: usize) -> Option<&TonCell> { self.roots.get(index) }

    pub fn single_root(mut self) -> Result<TonCell, TonCoreError> {
        if self.roots.len() != 1 {
            bail_ton_core_data!("Expected 1 root cell, got {}", self.roots.len());
        }
        Ok(self.roots.pop().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::TonHash;
    use std::str::FromStr;

    #[test]
    fn test_boc_create() {
        let boc = BoC::new(TonCell::empty().to_owned());
        assert_eq!(boc.roots.len(), 1);
    }

    #[test]
    fn test_boc_from_to() -> anyhow::Result<()> {
        let boc_hex = "b5ee9c720101040100b900010d000000000000050102c9801459f7c0a12bb4ac4b78a788c425ee4d52f8b6041dda17b77b09fc5a03e894d6900287cd9fbe2ea663415da0aa6bbdf0cb136abe9c4f45214dd259354b80da8c265a006aebb27f5d0f1daf43e200f52408f3eb9ff5610f5b43284224644e7c6a590d14400203084202c00836440d084e44fb94316132ac5a21417ef4f429ee09b5560b5678b334c3e8084202c95a2ed22ab516f77f9d4898dc4578e72f18a2448e8f6832334b0b4bf501bc79";
        let boc = BoC::from_hex(boc_hex)?;
        let boc_hex_back = boc.to_hex(false)?;
        assert_eq!(boc_hex, boc_hex_back);
        Ok(())
    }

    #[test]
    fn test_boc_from_to_cell_data() -> anyhow::Result<()> {
        let boc_hex = "b5ee9c720101040100b900010d000000000000050102c9801459f7c0a12bb4ac4b78a788c425ee4d52f8b6041dda17b77b09fc5a03e894d6900287cd9fbe2ea663415da0aa6bbdf0cb136abe9c4f45214dd259354b80da8c265a006aebb27f5d0f1daf43e200f52408f3eb9ff5610f5b43284224644e7c6a590d14400203084202c00836440d084e44fb94316132ac5a21417ef4f429ee09b5560b5678b334c3e8084202c95a2ed22ab516f77f9d4898dc4578e72f18a2448e8f6832334b0b4bf501bc79";
        let cell = BoC::from_hex(boc_hex)?.single_root()?;
        let hash = cell.hash()?;
        assert_eq!(hash, &TonHash::from_str("46284eb2ecbae743ce06dc39ba379f854f135dc8029c488abf8e84e90106302a")?);
        let serial_hex = BoC::new(cell).to_hex(false)?;
        assert_eq!(boc_hex, serial_hex);
        Ok(())
    }

    #[test]
    fn test_boc_from_to_cell_lib() -> anyhow::Result<()> {
        let boc_hex = "b5ee9c7201010101002300084202a9338ecd624ca15d37e4a8d9bf677ddc9b84f0e98f05f2fb84c7afe332a281b4";
        let cell = BoC::from_hex(boc_hex)?.single_root()?;
        let hash = cell.hash()?;
        assert_eq!(hash, &TonHash::from_str("ec614ea4aaea3f7768606f1c1632b3374d3de096a1e7c4ba43c8009c487fee9d")?);
        let serial_hex = BoC::new(cell).to_hex(false)?;
        assert_eq!(boc_hex, serial_hex);
        Ok(())
    }
}
