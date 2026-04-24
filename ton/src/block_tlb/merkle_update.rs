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

    fn ton_cell_type(&self) -> CellType { CellType::MerkleUpdate }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block_tlb::_test_block_data::TONVIEWER_BLOCK_0_8000000000000000_57314442_BOC_HEX;
    use crate::block_tlb::Block;
    use ton_core::cell::{TonCell, TonHash};
    use ton_core::traits::tlb::TLB;

    #[test]
    fn test_merkle_update_tlb_roundtrip() -> anyhow::Result<()> {
        let mut old_builder = TonCell::builder();
        old_builder.write_num(&0x1234u16, 16)?;
        let old = old_builder.build()?;

        let mut new_builder = TonCell::builder();
        new_builder.write_num(&0x5678u16, 16)?;
        let new = new_builder.build()?;

        let update = MerkleUpdate {
            old_hash: TonHash::from_slice_sized(old.hash()?.as_slice_sized()),
            new_hash: TonHash::from_slice_sized(new.hash()?.as_slice_sized()),
            old_depth: old.depth()?,
            new_depth: new.depth()?,
            old: old.clone(),
            new: new.clone(),
        };

        let cell = update.to_cell()?;
        assert_eq!(cell.cell_type(), CellType::MerkleUpdate);

        let parsed = MerkleUpdate::<TonCell>::from_cell(&cell)?;
        assert_eq!(parsed, update);
        assert_eq!(parsed.old, old);
        assert_eq!(parsed.new, new);
        Ok(())
    }

    #[test]
    fn test_merkle_update_from_recent_tonviewer_block() -> anyhow::Result<()> {
        let block = Block::from_boc_hex(TONVIEWER_BLOCK_0_8000000000000000_57314442_BOC_HEX)?;
        let state_update = block.state_update.into_inner();

        assert_ne!(state_update.old_hash, state_update.new_hash);
        assert_ne!(state_update.old_depth, 0);
        assert_ne!(state_update.new_depth, 0);

        let cell = state_update.to_cell()?;
        assert_eq!(cell.cell_type(), CellType::MerkleUpdate);

        let parsed_back = MerkleUpdate::<TonCell>::from_cell(&cell)?;
        assert_eq!(parsed_back, state_update);
        Ok(())
    }
}
