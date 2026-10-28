use super::*;
use crate::cell::ton_cell::{CellBorders, CellData, RefStorage};
use std::sync::Arc;

fn empty_cell_ref() -> TonCell {
    TonCell::empty().to_owned()
}

#[test]
fn test_refs_descriptor_d1() {
    let meta_builder = CellMetaBuilder::new(TonCell::empty());
    assert_eq!(meta_builder.get_refs_descriptor(0), 0);
    assert_eq!(meta_builder.get_refs_descriptor(3), 96);

    let cell_2 = TonCell {
        cell_type: CellType::Ordinary,
        cell_data: Arc::new(CellData {
            data_storage: Arc::new(vec![]),
            refs: RefStorage::from_iter([empty_cell_ref()]),
        }),
        meta: Arc::new(CellMeta::default()),
        borders: CellBorders {
            start_bit: 0,
            end_bit: 0,
            start_ref: 0,
            end_ref: 1,
        },
    };
    let meta_builder = CellMetaBuilder::new(&cell_2);
    assert_eq!(meta_builder.get_refs_descriptor(3), 97);

    let cell_3 = TonCell {
        cell_type: CellType::Ordinary,
        cell_data: Arc::new(CellData {
            data_storage: Arc::new(vec![]),
            refs: RefStorage::from_iter([empty_cell_ref(), empty_cell_ref()]),
        }),
        meta: Arc::new(CellMeta::default()),
        borders: CellBorders {
            start_bit: 0,
            end_bit: 0,
            start_ref: 0,
            end_ref: 2,
        },
    };
    let meta_builder = CellMetaBuilder::new(&cell_3);
    assert_eq!(meta_builder.get_refs_descriptor(3), 98);
}

#[test]
fn test_bits_descriptor_d2() {
    assert_eq!(get_bits_descriptor(0), 0);
    assert_eq!(get_bits_descriptor(1023), 255);
}

#[test]
fn test_hashes_and_depths() -> anyhow::Result<()> {
    let meta_builder = CellMetaBuilder::new(TonCell::empty());
    let level_mask = LevelMask::new(0);
    let (hashes, depths) = meta_builder.calc_hashes_and_depths(level_mask)?;

    for i in 0..4 {
        assert_eq!(hashes[i], TonCell::EMPTY_CELL_HASH);
        assert_eq!(depths[i], 0);
    }
    Ok(())
}
