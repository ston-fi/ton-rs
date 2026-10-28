use super::*;
use crate::block_tlb::_test_block_data::TONVIEWER_BLOCK_0_8000000000000000_57314442_BOC_HEX;
use crate::block_tlb::Block;
use num_bigint::BigUint;
use std::str::FromStr;
use ton_core::traits::tlb::TLB;

#[test]
fn test_value_flow_from_recent_tonviewer_block() -> anyhow::Result<()> {
    let block = Block::from_boc_hex(TONVIEWER_BLOCK_0_8000000000000000_57314442_BOC_HEX)?;
    let value_flow = block.value_flow.into_inner();
    let value_flow_v1 = value_flow.as_v1().unwrap();

    assert_eq!(value_flow_v1.from_prev_blk.coins.to_u128(), 3117276220959844347);
    assert_eq!(value_flow_v1.to_next_blk.coins.to_u128(), 3117276220632691490);
    assert_eq!(value_flow_v1.imported.coins.to_u128(), 0);
    assert_eq!(value_flow_v1.exported.coins.to_u128(), 0);
    assert_eq!(value_flow_v1.fees_collected.coins.to_u128(), 1327152857);
    assert_eq!(value_flow_v1.fees_imported.coins.to_u128(), 0);
    assert_eq!(value_flow_v1.recovered.coins.to_u128(), 0);
    assert_eq!(value_flow_v1.created.coins.to_u128(), 1000000000);
    assert_eq!(value_flow_v1.minted.coins.to_u128(), 0);
    assert_eq!(value_flow_v1.from_prev_blk.other.get(&239u32.into()).unwrap().data, BigUint::from_str("2333333332")?);
    assert_eq!(
        value_flow_v1.from_prev_blk.other.get(&4294967279u32.into()).unwrap().data,
        BigUint::from_str("1555555554")?
    );
    assert_eq!(value_flow_v1.to_next_blk.other.get(&239u32.into()).unwrap().data, BigUint::from_str("2333333332")?);
    assert_eq!(
        value_flow_v1.to_next_blk.other.get(&4294967279u32.into()).unwrap().data,
        BigUint::from_str("1555555554")?
    );

    let cell = value_flow.to_cell()?;
    let parsed_back = ValueFlow::from_cell(&cell)?;
    assert_eq!(parsed_back, value_flow);
    Ok(())
}

#[test]
fn test_value_flow_getters() -> anyhow::Result<()> {
    let mut value_flow_v1 = ValueFlow::V1(ValueFlowV1 {
        from_prev_blk: CurrencyCollection::from_num(&1u32)?,
        to_next_blk: CurrencyCollection::from_num(&2u32)?,
        imported: CurrencyCollection::from_num(&3u32)?,
        exported: CurrencyCollection::from_num(&4u32)?,
        fees_collected: CurrencyCollection::from_num(&5u32)?,
        fees_imported: CurrencyCollection::from_num(&6u32)?,
        recovered: CurrencyCollection::from_num(&7u32)?,
        created: CurrencyCollection::from_num(&8u32)?,
        minted: CurrencyCollection::from_num(&9u32)?,
    });
    assert_eq!(value_flow_v1.from_prev_blk().coins.to_u128(), 1);
    assert_eq!(value_flow_v1.minted().coins.to_u128(), 9);
    assert!(value_flow_v1.burned().is_none());
    value_flow_v1.created_mut().coins = 80u32.into();
    assert_eq!(value_flow_v1.created().coins.to_u128(), 80);

    let mut value_flow_v2 = ValueFlow::V2(ValueFlowV2 {
        from_prev_blk: CurrencyCollection::from_num(&11u32)?,
        to_next_blk: CurrencyCollection::from_num(&12u32)?,
        imported: CurrencyCollection::from_num(&13u32)?,
        exported: CurrencyCollection::from_num(&14u32)?,
        fees_collected: CurrencyCollection::from_num(&15u32)?,
        burned: CurrencyCollection::from_num(&16u32)?,
        fees_imported: CurrencyCollection::from_num(&17u32)?,
        recovered: CurrencyCollection::from_num(&18u32)?,
        created: CurrencyCollection::from_num(&19u32)?,
        minted: CurrencyCollection::from_num(&20u32)?,
    });
    assert_eq!(value_flow_v2.fees_collected().coins.to_u128(), 15);
    assert_eq!(value_flow_v2.burned().unwrap().coins.to_u128(), 16);
    value_flow_v2.burned_mut().unwrap().coins = 160u32.into();
    assert_eq!(value_flow_v2.burned().unwrap().coins.to_u128(), 160);

    Ok(())
}
