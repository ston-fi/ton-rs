use super::*;
use ton_core::traits::tlb::TLB;

#[test]
fn test_currency_collection() -> anyhow::Result<()> {
    let parsed = CurrencyCollection::from_boc_hex("b5ee9c720101010100070000094c143b1d14")?;
    assert_eq!(parsed.coins, 3242439121u32.into());

    let cell_serial = parsed.to_cell()?;
    let parsed_back = CurrencyCollection::from_cell(&cell_serial)?;
    assert_eq!(parsed, parsed_back);
    Ok(())
}

#[test]
fn test_currency_collection_zero_coins() -> anyhow::Result<()> {
    let currency = CurrencyCollection::from_num(&0u32)?;
    let cell = currency.to_cell()?;
    let parsed = CurrencyCollection::from_cell(&cell)?;
    assert_eq!(parsed.coins, 0u32.into());

    let cell_serial = parsed.to_cell()?;
    assert_eq!(cell_serial, cell);
    Ok(())
}
