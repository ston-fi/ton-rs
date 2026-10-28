use crate::cell::{CellBuilder, CellParser, CellType, TonCell, TonHash};
use crate::errors::TonCoreError;
use crate::traits::tlb::TLB;
use std::sync::Arc;

/// Wrapper type for TLB types stored by reference in cells.
/// Behaviour of wrapper itself is identical to the inner type
/// Only `read_definition` and `write_definition` methods are different
/// Type itself will be serialized the same way as the inner type (not to the ref cell!)
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct TLBRef<T>(T);

impl<T: TLB> TLBRef<T> {
    /// Wraps a value so enclosing TL-B fields store it in a referenced cell.
    pub const fn new(val: T) -> Self {
        Self(val)
    }
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T: TLB> TLB for TLBRef<T> {
    fn read_definition(parser: &mut CellParser) -> Result<Self, TonCoreError> {
        Ok(Self(T::from_cell(parser.read_next_ref()?)?))
    }
    fn write_definition(&self, builder: &mut CellBuilder) -> Result<(), TonCoreError> {
        builder.write_ref(self.0.to_cell()?)
    }
    fn cell_hash(&self) -> Result<TonHash, TonCoreError> {
        self.0.cell_hash()
    }
    fn from_boc<B: Into<Arc<Vec<u8>>>>(boc: B) -> Result<Self, TonCoreError> {
        Ok(Self::new(T::from_boc(boc)?))
    }
    fn to_cell(&self) -> Result<TonCell, TonCoreError> {
        self.0.to_cell()
    }
    fn to_boc_extra(&self, add_crc32: bool) -> Result<Vec<u8>, TonCoreError> {
        self.0.to_boc_extra(add_crc32)
    }
    fn ton_cell_type(&self) -> CellType {
        self.0.ton_cell_type()
    }
}

#[rustfmt::skip]
mod traits_impl {
    use std::ops::{Deref, DerefMut};
    use crate::traits::tlb::TLB;
    use crate::types::tlb_core::tlb_ref::TLBRef;
    
    impl<T> Deref for TLBRef<T> { type Target = T; fn deref(&self) -> &Self::Target { &self.0 } }
    impl<T> DerefMut for TLBRef<T> { fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 } }
    impl<T: TLB> From<T> for TLBRef<T> { fn from(value: T) -> Self { Self::new(value) } }
}

#[cfg(test)]
#[path = "../../tests/types/tlb_core/test_tlb_ref.rs"]
mod tests;
