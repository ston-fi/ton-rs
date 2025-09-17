use crate::cell::CellBuilder;
use crate::cell::CellParser;
use crate::errors::TonCoreError;
use crate::traits::tlb::TLB;

impl TLB for bool {
    fn read_definition(parser: &mut CellParser) -> Result<Self, TonCoreError> { parser.read_bit() }
    fn write_definition(&self, builder: &mut CellBuilder) -> Result<(), TonCoreError> { builder.write_bit(*self) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::TonCell;

    #[test]
    fn test_bool() -> anyhow::Result<()> {
        let mut builder = TonCell::builder();
        true.write(&mut builder)?;
        false.write(&mut builder)?;
        let cell = builder.build()?;
        assert_eq!(cell.data_bits_len, 2);
        assert_eq!(cell.data, vec![0b10000000]);
        Ok(())
    }
}
