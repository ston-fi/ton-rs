use ton_lib_core::cell::{CellBorders, CellBuilder, CellParser, TonCell};
use ton_lib_core::errors::TonCoreError;
use ton_lib_core::traits::tlb::{TLBPrefix, TLB};

// https://github.com/ton-blockchain/ton/blob/ed4682066978f69ffa38dd98912ca77d4f660f66/crypto/block/block.tlb#L873
// really tricky to implement with current design,
#[derive(Clone, PartialEq, Debug)]
pub struct TVMCellSlice {
    pub value: TonCell,
    pub borders: CellBorders, // relative to value.borders
}

impl TVMCellSlice {
    pub fn from_cell(cell: TonCell) -> Self {
        let borders = CellBorders {
            start_bit: 0,
            end_bit: cell.data_len_bits(),
            start_ref: 0,
            end_ref: cell.refs().len() as u8,
        };
        Self {
            value: cell.clone(),
            borders,
        }
    }

    pub fn to_cell(&self) -> Result<TonCell, TonCoreError> { self.value.slice(self.borders) }
}

impl TLB for TVMCellSlice {
    const PREFIX: TLBPrefix = TLBPrefix::new(0x04, 8);

    fn read_definition(parser: &mut CellParser) -> Result<Self, TonCoreError> {
        let cell = parser.read_next_ref()?.clone();

        let borders = CellBorders {
            start_bit: parser.read_num(10)?,
            end_bit: parser.read_num(10)?,
            start_ref: parser.read_num(3)?,
            end_ref: parser.read_num(3)?,
        };
        Ok(Self { value: cell, borders })
    }

    fn write_definition(&self, builder: &mut CellBuilder) -> Result<(), TonCoreError> {
        builder.write_ref(self.value.clone())?;
        builder.write_num(&self.borders.start_bit, 10)?;
        builder.write_num(&self.borders.end_bit, 10)?;
        builder.write_num(&self.borders.start_ref, 3)?;
        builder.write_num(&self.borders.end_ref, 3)?;
        Ok(())
    }
}
