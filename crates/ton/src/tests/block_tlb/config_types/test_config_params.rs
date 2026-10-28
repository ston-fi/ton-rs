use crate::block_tlb::_test_block_data::CONFIG_BOC_HEX;
use crate::block_tlb::{ConfigParam18, ConfigParams, GlobalVersion, StoragePrices};
use std::collections::HashMap;
use std::ops::Deref;
use ton_core::traits::tlb::TLB;

#[test]
fn test_config_params() -> anyhow::Result<()> {
    let config_params = ConfigParams::from_boc_hex(CONFIG_BOC_HEX)?;
    let serialized = config_params.to_boc()?;
    let parsed_back = ConfigParams::from_boc(serialized)?;
    assert_eq!(config_params, parsed_back);
    Ok(())
}

#[test]
fn test_config_param_8() -> anyhow::Result<()> {
    let parsed_param = ConfigParams::from_boc_hex(CONFIG_BOC_HEX)?.global_version()?;
    let expected_param = GlobalVersion {
        version: 9,
        capabilities: 494,
    };
    assert_eq!(parsed_param.deref(), &expected_param);
    Ok(())
}

#[test]
fn test_config_param_18() -> anyhow::Result<()> {
    let parsed_param = ConfigParams::from_boc_hex(CONFIG_BOC_HEX)?.storage_prices()?;
    let expected_prices = StoragePrices {
        utime_since: 0,
        bit_price_ps: 1,
        cell_price_ps: 500,
        mc_bit_price_ps: 1000,
        mc_cell_price_ps: 500000,
    };
    let expected_param = ConfigParam18 {
        storage_prices: HashMap::from([(0, expected_prices.clone())]),
    };
    assert_eq!(parsed_param.deref(), &expected_param);
    assert_eq!(&expected_prices, parsed_param.get_first()?);

    Ok(())
}
