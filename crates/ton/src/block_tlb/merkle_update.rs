use ton_core::cell::{CellBuilder, CellParser, CellType, TonHash};
use ton_core::errors::TonCoreError;
use ton_core::traits::tlb::{TLB, TLBPrefix};

// https://github.com/ton-blockchain/ton/blob/6f745c04daf8861bb1791cffce6edb1beec62204/crypto/block/block.tlb#L290-L291
#[derive(Debug, Clone, PartialEq)]
pub struct MerkleUpdate<T: TLB> {
    pub old_hash: TonHash,
    pub new_hash: TonHash,
    pub old_depth: u16,
    pub new_depth: u16,
    pub old: T,
    pub new: T,
}

impl<T: TLB> TLB for MerkleUpdate<T> {
    const PREFIX: TLBPrefix = TLBPrefix::new(0x04, 8);

    fn read_definition(parser: &mut CellParser) -> Result<Self, TonCoreError> {
        Ok(Self {
            old_hash: TLB::read(parser)?,
            new_hash: TLB::read(parser)?,
            old_depth: TLB::read(parser)?,
            new_depth: TLB::read(parser)?,
            old: T::from_cell(parser.read_next_ref()?)?,
            new: T::from_cell(parser.read_next_ref()?)?,
        })
    }

    fn write_definition(&self, builder: &mut CellBuilder) -> Result<(), TonCoreError> {
        self.old_hash.write(builder)?;
        self.new_hash.write(builder)?;
        self.old_depth.write(builder)?;
        self.new_depth.write(builder)?;
        builder.write_ref(self.old.to_cell()?)?;
        builder.write_ref(self.new.to_cell()?)?;
        Ok(())
    }

    fn ton_cell_type(&self) -> CellType {
        CellType::MerkleUpdate
    }
}

#[cfg(test)]
#[path = "../tests/block_tlb/test_merkle_update.rs"]
mod tests;
