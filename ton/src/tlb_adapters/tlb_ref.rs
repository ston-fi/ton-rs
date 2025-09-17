use std::marker::PhantomData;
use ton_lib_core::cell::CellBuilder;
use ton_lib_core::cell::CellParser;
use ton_lib_core::errors::TonCoreError;
use ton_lib_core::traits::tlb::TLB;

/// TLBRef - allows to save object in a reference cell ( ^X).
/// use `#[tlb_derive(adapter="TLBRef")]` to apply it using TLBDerive macro
#[derive(Debug, Clone, PartialEq)]
pub struct TLBRef<T: TLB>(PhantomData<T>);

impl<T: TLB> TLBRef<T> {
    pub fn new() -> Self { TLBRef(PhantomData) }
    pub fn read(&self, parser: &mut CellParser) -> Result<T, TonCoreError> { T::from_cell(parser.read_next_ref()?) }
    pub fn write(&self, builder: &mut CellBuilder, val: &T) -> Result<(), TonCoreError> {
        builder.write_ref(val.to_cell_ref()?)
    }
}

impl<T: TLB> Default for TLBRef<T> {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ton_lib_core::TLB;

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
        assert_eq!(cell.refs.len(), 2);
        assert_eq!(cell.refs[0].data, vec![255]);
        assert_eq!(cell.refs[1].data, vec![255]);

        let parsed = TestStruct::from_cell(&cell)?;
        assert_eq!(parsed, expected);
        Ok(())
    }
}
