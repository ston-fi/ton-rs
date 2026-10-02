use crate::tlb_adapters::{
    DictKeyAdapter, DictKeyAdapterInt, DictKeyAdapterUint, DictValAdapterTLB, TLBHashMap, TLBHashMapE,
};
use num_bigint::BigUint;
use std::collections::HashMap;
use ton_core::cell::{CellBuilder, TonCell};
use ton_core::traits::tlb::TLB;
use ton_core::types::tlb_core::adapters::UnaryLen;

#[test]
fn test_signed_dictionary_key_boundaries() -> anyhow::Result<()> {
    fn check<const N: usize>() -> anyhow::Result<()> {
        let limit = 1i64 << (N - 1);
        let adapter = TLBHashMapE::<DictKeyAdapterInt<N, i64>, DictValAdapterTLB<u32>>::new(N as u32);
        let data = HashMap::from([(-limit, 7u32), (-1, 8), (0, 9), (limit - 1, 10)]);
        let mut builder = TonCell::builder();
        adapter.write(&mut builder, &data)?;
        assert_eq!(adapter.read(&mut builder.build()?.parser())?, data);
        assert_eq!(DictKeyAdapterInt::<N, i64>::make_key(&-limit)?, BigUint::from(limit as u64));
        assert_eq!(DictKeyAdapterInt::<N, i64>::make_key(&(limit - 1))?, BigUint::from((limit - 1) as u64));
        Ok(())
    }
    check::<1>()?;
    check::<8>()?;
    check::<24>()?;
    check::<32>()?;
    Ok(())
}

#[test]
fn test_signed_dictionary_rejects_out_of_range_int24_keys() {
    let adapter = TLBHashMapE::<DictKeyAdapterInt<24, i32>, DictValAdapterTLB<u32>>::new(24);
    for key in [-0x800001, 0x800000, i32::MIN, i32::MAX] {
        assert!(DictKeyAdapterInt::<24, i32>::make_key(&key).is_err(), "key {key}");
        assert!(adapter.write(&mut TonCell::builder(), &HashMap::from([(key, 7u32)])).is_err(), "key {key}");
    }
}

#[test]
fn test_signed_dictionary_rejects_invalid_raw_keys_and_zero_width() {
    for key in [BigUint::from(1u32) << 24, (BigUint::from(1u32) << 24) + 1u32] {
        assert!(DictKeyAdapterInt::<24, i32>::extract_key(&key).is_err());
    }
    assert!(DictKeyAdapterInt::<0, i32>::make_key(&0).is_err());
    assert!(DictKeyAdapterInt::<0, i32>::extract_key(&BigUint::from(0u8)).is_err());
}

#[derive(Clone, Copy)]
enum Label {
    Short,
    Long,
    Same(bool),
}
const LABELS: [Label; 4] = [Label::Short, Label::Long, Label::Same(false), Label::Same(true)];

fn write_label(builder: &mut CellBuilder, label: Label, len: usize, remaining: usize) -> anyhow::Result<()> {
    let len_bits = (usize::BITS - remaining.leading_zeros()) as usize;
    match label {
        Label::Short => {
            builder.write_bit(false)?;
            UnaryLen(len).write(builder)?;
            builder.write_num(&0u16, len)?;
        },
        Label::Long => {
            builder.write_num(&0b10u8, 2)?;
            builder.write_num(&len, len_bits)?;
            builder.write_num(&0u16, len)?;
        },
        Label::Same(bit) => {
            builder.write_num(&0b11u8, 2)?;
            builder.write_bit(bit)?;
            builder.write_num(&len, len_bits)?;
        },
    }
    Ok(())
}

fn overlong_node(label: Label, len: usize, remaining: usize) -> anyhow::Result<TonCell> {
    let mut builder = TonCell::builder();
    write_label(&mut builder, label, len, remaining)?;
    // A child with a same label exposes the old parser's key-length underflow.
    let mut child = TonCell::builder();
    child.write_num(&0b110u8, 3)?;
    builder.write_ref(child.build()?)?;
    builder.write_ref(TonCell::empty().clone())?;
    Ok(builder.build()?)
}

#[test]
fn test_dictionary_rejects_overlong_root_labels() -> anyhow::Result<()> {
    let adapter = TLBHashMap::<DictKeyAdapterUint<u8>, DictValAdapterTLB<u32>>::new(8);
    for label in LABELS {
        for len in [9, 15] {
            assert!(adapter.read(&mut overlong_node(label, len, 8)?.parser()).is_err());
        }
    }
    Ok(())
}

#[test]
fn test_dictionary_rejects_overlong_nested_labels() -> anyhow::Result<()> {
    let adapter = TLBHashMap::<DictKeyAdapterUint<u8>, DictValAdapterTLB<u32>>::new(8);
    for label in LABELS {
        for len in [6, 7] {
            let mut root = TonCell::builder();
            write_label(&mut root, Label::Short, 2, 8)?;
            // The prefix and fork bit leave five key bits for each child.
            root.write_ref(overlong_node(label, len, 5)?)?;
            root.write_ref(TonCell::empty().clone())?;
            assert!(adapter.read(&mut root.build()?.parser()).is_err());
        }
    }
    Ok(())
}

#[test]
fn test_dictionary_accepts_alternative_and_zero_length_labels() -> anyhow::Result<()> {
    for label in LABELS {
        let mut leaf = TonCell::builder();
        write_label(&mut leaf, label, 8, 8)?;
        leaf.write_num(&7u32, 32)?;
        let key = if matches!(label, Label::Same(true)) { 255u8 } else { 0 };
        let adapter = TLBHashMap::<DictKeyAdapterUint<u8>, DictValAdapterTLB<u32>>::new(8);
        assert_eq!(adapter.read(&mut leaf.build()?.parser())?, HashMap::from([(key, 7u32)]));

        let mut leaf = TonCell::builder();
        write_label(&mut leaf, label, 0, 0)?;
        leaf.write_num(&7u32, 32)?;
        let adapter = TLBHashMap::<DictKeyAdapterUint<u8>, DictValAdapterTLB<u32>>::new(0);
        assert_eq!(adapter.read(&mut leaf.build()?.parser())?, HashMap::from([(0u8, 7u32)]));

        let mut root = TonCell::builder();
        write_label(&mut root, Label::Short, 0, 1)?;
        for value in [7u32, 8] {
            let mut child = TonCell::builder();
            write_label(&mut child, label, 0, 0)?;
            child.write_num(&value, 32)?;
            root.write_ref(child.build()?)?;
        }
        let adapter = TLBHashMap::<DictKeyAdapterUint<u8>, DictValAdapterTLB<u32>>::new(1);
        assert_eq!(adapter.read(&mut root.build()?.parser())?, HashMap::from([(0u8, 7u32), (1, 8)]));
    }
    Ok(())
}
