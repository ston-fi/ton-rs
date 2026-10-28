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
