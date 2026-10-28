use crate::cell::BoC;
use crate::cell::CellBuilder;
use crate::cell::CellParser;
use crate::cell::CellType;
use crate::cell::{TonCell, TonHash};
use crate::errors::TonCoreError;
use crate::traits::tlb::TLB;
use std::sync::Arc;

impl TLB for TonCell {
    fn read_definition(parser: &mut CellParser) -> Result<Self, TonCoreError> {
        parser.read_remaining()
    }

    fn write_definition(&self, builder: &mut CellBuilder) -> Result<(), TonCoreError> {
        builder.write_cell(self)
    }

    fn cell_hash(&self) -> Result<TonHash, TonCoreError> {
        Ok(*self.hash()?)
    }

    fn from_boc<T: Into<Arc<Vec<u8>>>>(boc: T) -> Result<Self, TonCoreError> {
        BoC::from_bytes(boc)?.single_root()
    }

    fn to_cell(&self) -> Result<TonCell, TonCoreError> {
        Ok(self.clone())
    }

    fn to_boc_extra(&self, add_crc32: bool) -> Result<Vec<u8>, TonCoreError> {
        BoC::new(self.clone()).to_bytes(add_crc32)
    }
    fn ton_cell_type(&self) -> CellType {
        self.cell_type()
    }
}

impl TLB for TonHash {
    fn read_definition(parser: &mut CellParser) -> Result<Self, TonCoreError> {
        TonHash::from_vec(parser.read_bits(TonHash::BITS_LEN)?)
    }

    fn write_definition(&self, builder: &mut CellBuilder) -> Result<(), TonCoreError> {
        builder.write_bits(self.as_slice(), TonHash::BITS_LEN)
    }
}

#[cfg(test)]
#[path = "../../tests/traits/tlb/test_tlb_cell.rs"]
mod tests;
