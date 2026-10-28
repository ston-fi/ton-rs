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
            },
        };
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../tests/traits/tlb/test_tlb_opt.rs"]
mod tests;
