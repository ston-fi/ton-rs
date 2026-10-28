use crate::cell::{BoC, CellBorders, TonCell};
use std::sync::Arc;

#[test]
fn test_ton_cell_slice() -> anyhow::Result<()> {
    let mut builder = TonCell::builder();
    builder.write_bits([1, 2, 3], 24)?;

    for i in 0..4 {
        let mut c_builder = TonCell::builder();
        c_builder.write_num(&i, 8)?;
        builder.write_ref(c_builder.build()?)?;
    }
    let cell = builder.build()?;
    assert_eq!(cell.underlying_storage(), &[1, 2, 3]);
    assert_eq!(cell.data_len_bits(), 24);
    assert_eq!(cell.refs().len(), 4);

    let slice = cell.slice(CellBorders {
        start_bit: 8,
        end_bit: 16,
        start_ref: 1,
        end_ref: 3,
    })?;
    assert_eq!(slice.underlying_storage(), &[1, 2, 3]);
    assert_eq!(slice.data_len_bits(), 8);
    assert_eq!(slice.refs().len(), 2);
    assert_eq!(slice.refs()[0].underlying_storage(), &[1]);
    assert_eq!(slice.refs()[1].underlying_storage(), &[2]);
    Ok(())
}

#[test]
fn test_deep_copy_subtree_from_right_child() -> anyhow::Result<()> {
    //      /  left_child \
    // root                 shared_leaf
    //      \ right_child /
    let mut leaf_builder = TonCell::builder();
    leaf_builder.write_bits([0b1110_0000], 7)?;
    let shared_leaf = leaf_builder.build()?;

    let mut left_builder = TonCell::builder();
    left_builder.write_bits([0b1011_0000], 6)?;
    left_builder.write_ref(shared_leaf.clone())?;
    let left = left_builder.build()?;

    let mut right_builder = TonCell::builder();
    right_builder.write_bits([0b0101_0000], 6)?;
    right_builder.write_ref(shared_leaf.clone())?;
    let right = right_builder.build()?;

    let mut root_builder = TonCell::builder();
    root_builder.write_bits([0xF0], 8)?;
    root_builder.write_ref(left.clone())?;
    root_builder.write_ref(right.clone())?;
    let _root = root_builder.build()?;

    // Deep copy only the right subtree
    let copy = TonCell::deep_copy(&right)?;
    let copy_bytes = BoC::new(copy.clone()).to_bytes(false)?;
    let parsed_copy = BoC::from_bytes(copy_bytes)?.single_root()?;
    assert_eq!(right, parsed_copy);

    Ok(())
}

#[test]
fn test_ton_cell_deep_copy_shard_block() -> anyhow::Result<()> {
    let boc_hex =
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/resources/tests/shard_block_6000000000000000_52111590.hex"));
    let cell = BoC::from_hex(boc_hex)?.single_root()?;
    let weak_data_ptr = Arc::downgrade(&cell.cell_data.data_storage);

    let copy = TonCell::deep_copy(&cell)?;
    let copy_bytes = BoC::new(copy.clone()).to_bytes(false)?;
    let parsed_copy = BoC::from_bytes(copy_bytes)?.single_root()?;
    assert_eq!(cell.hash()?, parsed_copy.hash()?);

    drop(cell);
    assert!(weak_data_ptr.upgrade().is_none());

    Ok(())
}

#[test]
fn test_ton_cell_view() -> anyhow::Result<()> {
    // https://ton.org/tvm.pdf
    // For example, 8A corresponds to binary string 10001010, while 8A_ and
    // 8A0_ both correspond to 100010
    let mut builder = TonCell::builder();
    builder.write_bit(true)?;
    builder.write_bit(false)?;
    builder.write_bit(false)?;
    builder.write_bit(false)?;
    builder.write_bit(true)?;
    builder.write_bit(false)?;

    let cell = builder.build()?;
    let repr = format!("{cell}");
    assert!(repr.contains("data: [8A_]"));
    Ok(())
}
