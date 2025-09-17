use std::marker::PhantomData;
use ton_lib_core::cell::CellBuilder;
use ton_lib_core::cell::CellParser;
use ton_lib_core::errors::TonCoreError;
use ton_lib_core::traits::tlb::TLB;

/// TLBOptRef - allows to save optional object ( Maybe(^X) ) in a reference cell.
/// use `#[tlb_derive(adapter="TLBRefOpt")]` to apply it using TLBDerive macro
#[derive(Debug, Clone, PartialEq)]
pub struct TLBRefOpt<T: TLB>(PhantomData<T>);

impl<T: TLB> TLBRefOpt<Option<T>> {
    pub fn new() -> Self { TLBRefOpt(PhantomData) }

    pub fn read(&self, parser: &mut CellParser) -> Result<Option<T>, TonCoreError> {
        match parser.read_bit()? {
            true => Ok(Some(T::from_cell(parser.read_next_ref()?)?)),
            false => Ok(None),
        }
    }

    pub fn write(&self, builder: &mut CellBuilder, val: &Option<T>) -> Result<(), TonCoreError> {
        builder.write_bit(val.is_some())?;
        if let Some(val) = val {
            builder.write_ref(val.to_cell_ref()?)?;
        }
        Ok(())
    }
}

impl<T: TLB> Default for TLBRefOpt<Option<T>> {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tlb_adapters::TLBRef;
    use ton_lib_core::cell::TonCell;
    use ton_lib_core::TLB;

    #[test]
    fn test_tlb_ref() -> anyhow::Result<()> {
        let mut builder = TonCell::builder();
        TLBRef::<bool>::new().write(&mut builder, &true)?;
        let cell = builder.build()?;
        assert_eq!(cell.refs.len(), 1);
        assert_eq!(cell.refs[0].data, vec![0b10000000]);

        let parsed = TLBRef::<bool>::new().read(&mut cell.parser())?;
        assert!(parsed);
        Ok(())
    }

    #[derive(PartialEq, Debug, TLB)]
    struct TestStruct {
        #[tlb(adapter = "TLBRef::<u8>::new()")]
        pub a: u8,
        #[tlb(adapter = "TLBRef")]
        pub b: u8,
    }

    #[test]
    fn test_tlb_ref_derive() -> anyhow::Result<()> {
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
