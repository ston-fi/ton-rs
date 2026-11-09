use std::collections::HashMap;

use ton_core::cell::TonCell;
use ton_core::types::tlb_core::TLBRef;

use crate::tlb_adapters::{DictKeyAdapterUint, DictValAdapterNum, DictValAdapterTLB, TLBHashMap, TLBHashMapE};

type Dictionary = TLBHashMap<DictKeyAdapterUint<u8>, DictValAdapterNum<u8, 8>>;
type OptionalDictionary = TLBHashMapE<DictKeyAdapterUint<u8>, DictValAdapterNum<u8, 8>>;

fn append_at(cell: &TonCell, path: &[usize], reference: bool) -> anyhow::Result<TonCell> {
    let mut builder = TonCell::builder();
    builder.write_bits(&cell.parser().read_bits(cell.data_len_bits())?, cell.data_len_bits())?;
    for (index, child) in cell.refs().iter().enumerate() {
        if path.first() == Some(&index) {
            builder.write_ref(append_at(child, &path[1..], reference)?)?;
        } else {
            builder.write_ref(child.clone())?;
        }
    }
    if path.is_empty() {
        if reference {
            builder.write_ref(TonCell::empty().clone())?;
        } else {
            builder.write_bit(true)?;
        }
    }
    Ok(builder.build()?)
}

#[test]
fn rejects_remainders_in_leaf_root_fork_and_nested_fork() -> anyhow::Result<()> {
    let entries = HashMap::from([(0u8, 4u8), (1, 5), (128, 6)]);
    let mut builder = TonCell::builder();
    Dictionary::new(8).write(&mut builder, &entries)?;
    let cell = builder.build()?;
    assert_eq!(Dictionary::new(8).with_strict_nodes().read(&mut cell.parser())?, entries);
    for path in [&[][..], &[0][..], &[0, 0][..]] {
        for reference in [false, true] {
            let malformed = append_at(&cell, path, reference)?;
            assert_eq!(Dictionary::new(8).read(&mut malformed.parser())?, entries);
            assert!(
                Dictionary::new(8).with_strict_nodes().read(&mut malformed.parser()).is_err(),
                "path={path:?}, reference={reference}"
            );
            let mut wrapper = TonCell::builder();
            wrapper.write_bit(true)?;
            wrapper.write_ref(malformed)?;
            assert!(OptionalDictionary::new(8).with_strict_nodes().read(&mut wrapper.build()?.parser()).is_err());
        }
    }
    Ok(())
}

#[test]
fn accepts_all_label_encodings_including_zero_length() -> anyhow::Result<()> {
    for width in [0usize, 8] {
        for kind in [0, 1, 2] {
            let mut builder = TonCell::builder();
            let len_bits = (usize::BITS - width.leading_zeros()) as usize;
            match kind {
                0 => {
                    builder.write_bit(false)?;
                    for _ in 0..width {
                        builder.write_bit(true)?;
                    }
                    builder.write_bit(false)?;
                    builder.write_num(&0u8, width)?;
                },
                1 => {
                    builder.write_num(&2u8, 2)?;
                    builder.write_num(&width, len_bits)?;
                    builder.write_num(&0u8, width)?;
                },
                _ => {
                    builder.write_num(&3u8, 2)?;
                    builder.write_bit(false)?;
                    builder.write_num(&width, len_bits)?;
                },
            }
            builder.write_num(&42u8, 8)?;
            let cell = builder.build()?;
            assert_eq!(
                Dictionary::new(width as u32).with_strict_nodes().read(&mut cell.parser())?,
                HashMap::from([(0, 42)])
            );
        }
    }
    Ok(())
}

#[test]
fn optional_dictionary_leaves_containing_cell_fields_available() -> anyhow::Result<()> {
    for entries in [HashMap::new(), HashMap::from([(0u8, 42u8)])] {
        let mut builder = TonCell::builder();
        OptionalDictionary::new(8).with_strict_nodes().write(&mut builder, &entries)?;
        builder.write_num(&7u8, 8)?;
        builder.write_ref(TonCell::empty().clone())?;
        let cell = builder.build()?;
        let mut parser = cell.parser();
        assert_eq!(OptionalDictionary::new(8).with_strict_nodes().read(&mut parser)?, entries);
        assert_eq!(parser.read_num::<u8>(8)?, 7);
        assert_eq!(parser.read_next_ref()?, TonCell::empty());
        parser.ensure_empty()?;
    }
    Ok(())
}

#[test]
fn value_adapter_can_consume_a_leaf_reference() -> anyhow::Result<()> {
    let entries = HashMap::from([(0u8, TLBRef::new(TonCell::empty().clone()))]);
    let adapter = TLBHashMap::<DictKeyAdapterUint<u8>, DictValAdapterTLB<TLBRef<TonCell>>>::new(8).with_strict_nodes();
    let mut builder = TonCell::builder();
    adapter.write(&mut builder, &entries)?;
    assert_eq!(adapter.read(&mut builder.build()?.parser())?, entries);
    Ok(())
}

#[test]
fn accepts_zero_length_labels_at_forks_and_leaves() -> anyhow::Result<()> {
    for root_kind in [0u8, 1, 2] {
        for leaf_kind in [0u8, 1, 2] {
            let mut leaf = TonCell::builder();
            match leaf_kind {
                0 => {
                    leaf.write_num(&0u8, 2)?;
                },
                1 => {
                    leaf.write_num(&2u8, 2)?;
                },
                _ => {
                    leaf.write_num(&3u8, 2)?;
                    leaf.write_bit(false)?;
                },
            }
            leaf.write_num(&42u8, 8)?;
            let leaf = leaf.build()?;
            let mut root = TonCell::builder();
            match root_kind {
                0 => {
                    root.write_num(&0u8, 2)?;
                },
                1 => {
                    root.write_num(&2u8, 2)?;
                    root.write_bit(false)?;
                },
                _ => {
                    root.write_num(&3u8, 2)?;
                    root.write_bit(false)?;
                    root.write_bit(false)?;
                },
            }
            root.write_ref(leaf.clone())?;
            root.write_ref(leaf)?;
            assert_eq!(
                Dictionary::new(1).with_strict_nodes().read(&mut root.build()?.parser())?,
                HashMap::from([(0, 42), (1, 42)])
            );
        }
    }
    Ok(())
}
