use crate::cell::CellBuilder;
use crate::cell::CellParser;
use crate::errors::TonCoreError;
use crate::traits::tlb::TLB;

impl TLB for bool {
    fn read_definition(parser: &mut CellParser) -> Result<Self, TonCoreError> {
        parser.read_bit()
    }
    fn write_definition(&self, builder: &mut CellBuilder) -> Result<(), TonCoreError> {
        builder.write_bit(*self)
    }
}

#[cfg(test)]
#[path = "../../tests/traits/tlb/test_tlb_bool.rs"]
mod tests;
