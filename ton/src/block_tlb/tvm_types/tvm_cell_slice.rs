use ton_lib_core::cell::{CellBuilder, CellParser, TonCellRef};
use ton_lib_core::errors::TonCoreError;
use ton_lib_core::traits::tlb::{TLBPrefix, TLB};

// https://github.com/ton-blockchain/ton/blob/ed4682066978f69ffa38dd98912ca77d4f660f66/crypto/block/block.tlb#L873
// really tricky to implement with current design,
#[derive(Clone, PartialEq, Debug)]
pub struct TVMCellSlice {
    pub value: TonCellRef, // is not part of TLB
    pub cell_original: TonCellRef,
    pub start_bit: usize,
    pub end_bit: usize,
    pub start_ref: usize,
    pub end_ref: usize,
}

impl TVMCellSlice {
    pub fn from_cell(cell: TonCellRef) -> Self {
        let end_bit = cell.data_bits_len;
        let end_ref = cell.refs.len();
        Self {
            value: cell.clone(),
            cell_original: cell,
            start_bit: 0,
            end_bit,
            start_ref: 0,
            end_ref,
        }
    }
}

impl TLB for TVMCellSlice {
    const PREFIX: TLBPrefix = TLBPrefix::new(0x04, 8);

    fn read_definition(parser: &mut CellParser) -> Result<Self, TonCoreError> {
        let cell_original = parser.read_next_ref()?.clone();
        let start_bit = parser.read_num(10)?;
        let end_bit = parser.read_num(10)?;
        let start_ref = parser.read_num(3)?;
        let end_ref = parser.read_num(3)?;

        let value = cell_original.parser().read_cell_slice(start_bit, end_bit, start_ref, end_ref)?.into_ref();
        Ok(Self {
            value,
            cell_original,
            start_bit,
            end_bit,
            start_ref,
            end_ref,
        })
    }

    fn write_definition(&self, builder: &mut CellBuilder) -> Result<(), TonCoreError> {
        builder.write_ref(self.cell_original.clone())?;
        builder.write_num(&self.start_bit, 10)?;
        builder.write_num(&self.end_bit, 10)?;
        builder.write_num(&self.start_ref, 3)?;
        builder.write_num(&self.end_ref, 3)?;
        Ok(())
    }
}
