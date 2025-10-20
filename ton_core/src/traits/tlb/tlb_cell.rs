use crate::cell::BoC;
use crate::cell::CellBuilder;
use crate::cell::CellParser;
use crate::cell::CellType;
use crate::cell::{TonCell, TonHash};
use crate::errors::TonCoreError;
use crate::traits::tlb::TLB;
use std::sync::Arc;

impl TLB for TonCell {
    fn read_definition(parser: &mut CellParser) -> Result<Self, TonCoreError> { parser.read_remaining() }

    fn write_definition(&self, builder: &mut CellBuilder) -> Result<(), TonCoreError> { builder.write_cell(self) }

    fn cell_hash(&self) -> Result<TonHash, TonCoreError> { Ok(self.hash()?.clone()) }

    fn from_boc<T: Into<Arc<Vec<u8>>>>(boc: T) -> Result<Self, TonCoreError> { BoC::from_bytes(boc)?.single_root() }

    fn to_cell(&self) -> Result<TonCell, TonCoreError> { Ok(self.clone()) }

    fn to_boc_extra(&self, add_crc32: bool) -> Result<Vec<u8>, TonCoreError> {
        BoC::new(self.clone()).to_bytes(add_crc32)
    }
    fn ton_cell_type(&self) -> CellType { self.cell_type() }
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
mod tests {
    use super::*;
    use tokio_test::assert_err;
    use ton_macros::TLB;

    #[test]
    fn test_tlb_cell() -> anyhow::Result<()> {
        let mut builder = TonCell::builder();
        builder.write_num(&3u32, 32)?;
        let cell = builder.build()?;
        let parsed = TonCell::from_cell(&cell)?;
        assert_eq!(cell, parsed);
        Ok(())
    }

    #[test]
    fn test_tlb_cell_boc() -> anyhow::Result<()> {
        let mut cell = TonCell::builder();
        cell.write_num(&3u32, 32)?;
        let cell_ref = cell.build()?;
        let boc = cell_ref.to_boc()?;
        let parsed_ref = TonCell::from_boc(boc.clone())?;
        assert_eq!(cell_ref, parsed_ref);

        let parsed_cell = TonCell::from_boc(boc)?;
        assert_eq!(parsed_cell, cell_ref);
        Ok(())
    }

    #[test]
    fn test_tlb_cell_boc_library() -> anyhow::Result<()> {
        let lib_hex = "b5ee9c720101010100230008420257de63d28e4d3608e0c02d437a7b50ef5f28f36a4821a047fd663ce63f4597ec";
        let lib_cell = TonCell::from_boc_hex(lib_hex)?;
        assert_eq!(lib_cell.cell_type(), CellType::LibraryRef);
        assert_eq!(lib_cell.to_boc_hex()?, lib_hex);

        let lib_cell_ref = TonCell::from_boc_hex(lib_hex)?;
        assert_eq!(lib_cell.cell_type(), CellType::LibraryRef);
        assert_eq!(lib_cell.to_boc_hex()?, lib_hex);

        // now library is a second cell
        let mut builder = TonCell::builder();
        builder.write_ref(lib_cell_ref.clone())?;
        let cell_with_lib_child_hex = builder.build()?.to_boc_hex()?;

        let cell_with_lib_child = TonCell::from_boc_hex(&cell_with_lib_child_hex)?;
        assert_eq!(cell_with_lib_child.cell_type(), CellType::Ordinary);
        assert_eq!(cell_with_lib_child.refs()[0].cell_type(), CellType::LibraryRef);
        assert_eq!(cell_with_lib_child.to_boc_hex()?, cell_with_lib_child_hex);

        let lib_child_cell_ref = TonCell::from_boc_hex(&cell_with_lib_child_hex)?;
        assert_eq!(lib_child_cell_ref.cell_type(), CellType::Ordinary);
        assert_eq!(lib_child_cell_ref.refs()[0].cell_type(), CellType::LibraryRef);
        assert_eq!(lib_child_cell_ref.to_boc_hex()?, cell_with_lib_child_hex);

        // using extra tlb-object
        #[derive(Debug, PartialEq, TLB)]
        struct TestStruct {
            cell: TonCell,
        }
        let test_struct = TestStruct {
            cell: cell_with_lib_child.clone(),
        };
        let struct_hex = test_struct.to_boc_hex()?;
        let parsed_struct = TestStruct::from_boc_hex(&struct_hex)?;
        assert_eq!(test_struct, parsed_struct);
        let parsed_cell = TonCell::from_boc_hex(&struct_hex)?;
        assert_eq!(parsed_cell.cell_type(), CellType::Ordinary);
        assert_eq!(parsed_cell.refs()[0].cell_type(), CellType::LibraryRef);
        Ok(())
    }

    #[test]
    fn test_tlb_from_boc_nice_error() -> anyhow::Result<()> {
        // using extra tlb-object
        #[derive(Debug, PartialEq, TLB)]
        struct TestStruct;
        let err = assert_err!(TestStruct::from_boc(vec![0x00, 0x01, 0x02]));
        assert!(err.to_string().contains("Fail to read"), "Actual error: {err}");
        assert!(err.to_string().contains("TestStruct"), "Actual error: {err}");
        Ok(())
    }
}
