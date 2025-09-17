use crate::cell::CellBuilder;
use crate::cell::CellParser;
use crate::errors::TonCoreError;
use crate::traits::tlb::TLB;

// Maybe X
impl<T: TLB> TLB for Option<T> {
    fn read_definition(parser: &mut CellParser) -> Result<Self, TonCoreError> {
        match parser.read_bit()? {
            false => Ok(None),
            true => Ok(Some(T::read(parser)?)),
        }
    }

    fn write_definition(&self, dst: &mut CellBuilder) -> Result<(), TonCoreError> {
        match self {
            None => dst.write_bit(false)?,
            Some(value) => {
                dst.write_bit(true)?;
                value.write(dst)?;
            }
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::TonCell;
    use ton_lib_macros::TLB;

    #[derive(Debug, PartialEq, TLB)]
    struct TestType1(i32);

    #[test]
    fn test_option() -> anyhow::Result<()> {
        let obj1 = Some(TestType1(1));
        let obj2: Option<TestType1> = None;
        let mut builder = TonCell::builder();
        obj1.write(&mut builder)?;
        obj2.write(&mut builder)?;

        let cell = builder.build()?;
        let mut parser = cell.parser();
        let parsed_obj1: Option<TestType1> = TLB::read(&mut parser)?;
        let parsed_obj2: Option<TestType1> = TLB::read(&mut parser)?;
        assert_eq!(obj1, parsed_obj1);
        assert_eq!(None, parsed_obj2);

        // check layout
        let mut parser = cell.parser();
        assert!(parser.read_bit()?); // Some
        parser.seek_bits(32)?; // skipping
        assert!(!parser.read_bit()?); // None
        Ok(())
    }
}
