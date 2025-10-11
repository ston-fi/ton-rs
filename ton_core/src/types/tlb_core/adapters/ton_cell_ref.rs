use crate::cell::{BoC, CellBuilder, CellParser, CellType, TonCell, TonHash};
use crate::errors::TonCoreError;
use crate::traits::tlb::TLB;
use std::ops::Deref;
use std::sync::Arc;

// It's commonly-used adapter with overloaded TLB implementation
// Which allows you to write TonCell as a reference instead of using TLBRef adapter
#[derive(Clone, PartialEq, Eq)]
pub struct TonCellRef(TonCell);

impl TLB for TonCellRef {
    fn read_definition(parser: &mut CellParser) -> Result<Self, TonCoreError> { parser.read_next_ref().map(Into::into) }
    fn write_definition(&self, builder: &mut CellBuilder) -> Result<(), TonCoreError> { builder.write_ref(self) }
    fn cell_hash(&self) -> Result<TonHash, TonCoreError> { Ok(self.hash()?.clone()) }
    fn from_boc<T: Into<Arc<Vec<u8>>>>(boc: T) -> Result<Self, TonCoreError> {
        BoC::from_bytes(boc)?.single_root().map(Into::into)
    }
    fn to_cell(&self) -> Result<TonCell, TonCoreError> { Ok(self.deref().clone()) }
    fn to_boc_extra(&self, add_crc32: bool) -> Result<Vec<u8>, TonCoreError> {
        BoC::new(self.deref().clone()).to_bytes(add_crc32)
    }
    fn ton_cell_type(&self) -> CellType { self.cell_type() }
}

mod traits_impl {
    use crate::cell::TonCell;
    use crate::types::tlb_core::adapters::ton_cell_ref::TonCellRef;
    use std::fmt::{Debug, Display, Formatter};
    use std::ops::{Deref, DerefMut};

    // TonCellRef
    impl Display for TonCellRef {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { write!(f, "{}", self.0) }
    }
    // expensive
    impl Debug for TonCellRef {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { write!(f, "{self}") }
    }
    impl Deref for TonCellRef {
        type Target = TonCell;
        fn deref(&self) -> &Self::Target { &self.0 }
    }
    impl DerefMut for TonCellRef {
        fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
    }
    impl From<TonCell> for TonCellRef {
        fn from(cell: TonCell) -> Self { Self(cell) }
    }
    impl From<&TonCell> for TonCellRef {
        fn from(cell: &TonCell) -> Self { Self(cell.clone()) }
    }
    impl From<TonCellRef> for TonCell {
        fn from(cell_ref: TonCellRef) -> Self { cell_ref.0 }
    }
    impl From<&TonCellRef> for TonCell {
        fn from(cell_ref: &TonCellRef) -> Self { cell_ref.0.clone() }
    }
}

#[cfg(test)]
mod tests {
    use crate::cell::{CellType, TonCell};
    use crate::traits::tlb::TLB;
    use crate::types::tlb_core::adapters::ton_cell_ref::TonCellRef;
    use ton_lib_macros::TLB;

    #[test]
    fn test_tlb_ton_cell_ref() -> anyhow::Result<()> {
        let lib_cell = TonCell::from_boc_hex(
            "b5ee9c720101010100230008420257de63d28e4d3608e0c02d437a7b50ef5f28f36a4821a047fd663ce63f4597ec",
        )?;
        #[derive(Debug, PartialEq, TLB)]
        struct TestStruct {
            cell: TonCellRef,
        }
        let test_struct = TestStruct { cell: lib_cell.into() };
        let struct_hex = test_struct.to_boc_hex()?;
        let parsed_struct = TestStruct::from_boc_hex(&struct_hex)?;
        assert_eq!(test_struct, parsed_struct);
        let parsed_cell = TonCell::from_boc_hex(&struct_hex)?;
        assert_eq!(parsed_cell.cell_type(), CellType::Ordinary);
        assert_eq!(parsed_cell.refs()[0].cell_type(), CellType::LibraryRef);
        Ok(())
    }
}
