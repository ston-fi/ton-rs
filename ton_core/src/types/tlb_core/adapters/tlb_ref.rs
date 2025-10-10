use crate::cell::CellBuilder;
use crate::cell::CellParser;
use crate::errors::TonCoreError;
use crate::traits::tlb::TLB;
use std::marker::PhantomData;

/// TLBRef - allows to save object in a reference cell ( ^X).
/// use `#[tlb(adapter="TLBRef")]` to apply it using TLB macro
#[derive(Debug, Clone, PartialEq)]
pub struct TLBRef<T: TLB>(PhantomData<T>);

impl<T: TLB> TLBRef<T> {
    pub fn new() -> Self { TLBRef(PhantomData) }
    pub fn read(&self, parser: &mut CellParser) -> Result<T, TonCoreError> { T::from_cell(parser.read_next_ref()?) }
    pub fn write(&self, builder: &mut CellBuilder, val: &T) -> Result<(), TonCoreError> {
        builder.write_ref(val.to_cell()?)
    }
}

impl<T: TLB> Default for TLBRef<T> {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ton_lib_macros::TLB;

    #[derive(TLB, PartialEq, Debug)]
    struct TestStruct {
        #[tlb(adapter = "TLBRef::<u8>::new()")]
        pub a: u8,
        #[tlb(adapter = "TLBRef")]
        pub b: u8,
    }

    #[test]
    fn test_tlb_ref_opt_derive() -> anyhow::Result<()> {
        let expected = TestStruct { a: 255, b: 255 };
        let cell = expected.to_cell()?;
        assert_eq!(cell.refs().len(), 2);
        assert_eq!(cell.refs()[0].underlying_storage(), vec![255]);
        assert_eq!(cell.refs()[1].underlying_storage(), vec![255]);

        let parsed = TestStruct::from_cell(&cell)?;
        assert_eq!(parsed, expected);
        Ok(())
    }
}
