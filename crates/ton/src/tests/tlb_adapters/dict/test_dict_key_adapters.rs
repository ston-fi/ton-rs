use super::*;
use crate::tlb_adapters::{DictValAdapterTLB, TLBHashMap};
use std::str::FromStr;

#[test]
fn test_dict_key_adapter_ton_hash() -> anyhow::Result<()> {
    let dict_key = DictKeyAdapterTonHash::make_key(&TonHash::ZERO)?;
    assert_eq!(dict_key, 0u32.into());
    assert_eq!(DictKeyAdapterTonHash::extract_key(&dict_key)?, TonHash::ZERO);

    let dict_key = DictKeyAdapterTonHash::make_key(&TonHash::from([0b1010_1010; 32]))?;
    assert_eq!(
        dict_key,
        BigUint::from_str("77194726158210796949047323339125271902179989777093709359638389338608753093290")?
    );
    assert_eq!(DictKeyAdapterTonHash::extract_key(&dict_key)?, TonHash::from([0b1010_1010; 32]));
    Ok(())
}

#[test]
fn test_dict_key_adapter_uint() -> anyhow::Result<()> {
    for val in [0u32, 1, 13, 190, 9999999] {
        let dict_key = DictKeyAdapterUint::make_key(&val)?;
        let extracted_val = DictKeyAdapterUint::<u32>::extract_key(&dict_key)?;
        assert_eq!(val, extracted_val);
    }
    Ok(())
}

#[test]
fn test_dict_key_adapter_int() -> anyhow::Result<()> {
    for val in [-9999999i32, -190, -13, -1, 0, 1, 13, 190, 9999999] {
        let dict_key = DictKeyAdapterInt::<30, _>::make_key(&val)?;
        let extracted_val = DictKeyAdapterInt::<30, i32>::extract_key(&dict_key)?;
        assert_eq!(val, extracted_val);
    }
    Ok(())
}

#[test]
fn test_dict_key_adapter_signed_key_parse_tlb() -> anyhow::Result<()> {
    let dict_hex = "b5ee9c72010207010001e600020120010200e7ae3626d0000000000000000000000000000000000000000000000000000046ec8cd22d8bffffffffffffffffffffb913732dd27800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002020277030400e7a69d930000000000000000000000000000000000000000000000000000046ec8cd22d880000000000000000000046ec8cd22d8800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002002038ddc050600e5b14500000000000000000000000000000000000000000000000000001c0c3e8aba24400000000000000000001c0c3e8aba24400000000000000000000000000000000002f23e52bc009da2a23b462da0fa694000000000000000000000000000000000000e3288b94abb3942fb8a96aeac2fe000e5b2b800000000000000000000000000000000000000000000000000001c0c3e8aba247fffffffffffffffffffe3f3c17545dbc000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000020";
    let dict_cell = TonCell::from_boc_hex(dict_hex)?;
    let dict =
        TLBHashMap::<DictKeyAdapterInt<24, i32>, DictValAdapterTLB<TonCell>>::new(24).read(&mut dict_cell.parser())?;
    assert_eq!(dict.len(), 4);
    for key in &[-34080, -39660, -887220, 887220] {
        assert!(dict.contains_key(key), "key {key} not found, available keys: {:?}", dict.keys());
    }

    let mut builder = TonCell::builder();
    TLBHashMap::<DictKeyAdapterInt<24, i32>, DictValAdapterTLB<TonCell>>::new(24).write(&mut builder, &dict)?;
    let serialized = builder.build()?.to_boc_hex()?;
    assert_eq!(dict_hex, serialized);
    Ok(())
}
