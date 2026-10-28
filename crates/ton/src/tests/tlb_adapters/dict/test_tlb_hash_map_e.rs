use super::*;
use crate::tlb_adapters::{DictKeyAdapterUint, DictValAdapterNum};
use num_bigint::BigUint;
use ton_core::traits::tlb::TLB;

#[test]
fn test_dict_blockchain_data() -> anyhow::Result<()> {
    let expected_data = HashMap::from([
        (0u8, BigUint::from(25965603044000000000u128)),
        (1, BigUint::from(5173255344000000000u64)),
        (2, BigUint::from(344883687000000000u64)),
    ]);
    let boc_hex = "b5ee9c7241010601005a000119c70d3ca5000d99b931ea4e8cc0010201cd020302012004050027400000000000000000000001325178b51d9180200026000000000000000000000168585a65986be8000026000000000000000000000047cb18538782e000353c80b9";
    let dict_cell = TonCell::from_boc_hex(boc_hex)?;
    let mut parser = dict_cell.parser();
    let some_data = parser.read_bits(96)?;

    let parsed_data = TLBHashMapE::<DictKeyAdapterUint<_>, DictValAdapterNum<_, 150>>::new(8).read(&mut parser)?;
    assert_eq!(expected_data, parsed_data);
    let mut builder = TonCell::builder();
    builder.write_bits(&some_data, 96)?;
    TLBHashMapE::<DictKeyAdapterUint<_>, DictValAdapterNum<_, 150>>::new(8).write(&mut builder, &expected_data)?;
    let constructed_cell = builder.build()?;
    assert_eq!(dict_cell, constructed_cell);
    Ok(())
}

#[test]
fn test_dict_key_bits_len_bigger_than_key() -> anyhow::Result<()> {
    let data = HashMap::from([
        (0u16, BigUint::from(4u32)),
        (1, BigUint::from(5u32)),
        (2, BigUint::from(6u32)),
        (10u16, BigUint::from(7u32)),
        (127, BigUint::from(8u32)),
    ]);

    for key_len_bits in [8u32, 16, 32, 64, 111] {
        let mut builder = TonCell::builder();
        TLBHashMapE::<DictKeyAdapterUint<_>, DictValAdapterNum<_, 150>>::new(key_len_bits)
            .write(&mut builder, &data)?;
        let dict_cell = builder.build()?;
        let mut parser = dict_cell.parser();
        let parsed =
            TLBHashMapE::<DictKeyAdapterUint<_>, DictValAdapterNum<_, 150>>::new(key_len_bits).read(&mut parser)?;
        assert_eq!(data, parsed, "key_len_bits: {key_len_bits}");
    }
    Ok(())
}
