use crate::cell::{CellBuilder, CellParser, CellType};
use crate::errors::TonCoreError;
use crate::traits::tlb::TLB;

#[derive(Debug, Clone, PartialEq)]
pub struct TLBRef<T>(T);

impl<T: TLB> TLBRef<T> {
    pub fn new<K: Into<T>>(val: K) -> Self { Self(val.into()) }
    pub fn into_inner(self) -> T { self.0 }
}

impl<T: TLB> TLB for TLBRef<T> {
    fn read_definition(parser: &mut CellParser) -> Result<Self, TonCoreError> {
        Ok(Self(T::from_cell(parser.read_next_ref()?)?))
    }
    fn write_definition(&self, builder: &mut CellBuilder) -> Result<(), TonCoreError> {
        builder.write_ref(self.0.to_cell()?)
    }
    fn ton_cell_type(&self) -> CellType { self.0.ton_cell_type() }
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
mod tests {
    use super::*;
    use ton_lib_macros::TLB;

    #[derive(TLB, PartialEq, Debug)]
    struct TestStruct {
        pub a: TLBRef<u8>,
        pub b: TLBRef<u8>,
    }

    #[test]
    fn test_tlb_ref_opt_derive() -> anyhow::Result<()> {
        let expected = TestStruct {
            a: 255.into(),
            b: 255.into(),
        };
        let cell = expected.to_cell()?;
        assert_eq!(cell.refs().len(), 2);
        assert_eq!(cell.refs()[0].underlying_storage(), vec![255]);
        assert_eq!(cell.refs()[1].underlying_storage(), vec![255]);
        let parsed = TestStruct::from_cell(&cell)?;
        assert_eq!(parsed, expected);
        Ok(())
    }
}
