use crate::block_tlb::{ConfigParam18, GlobalVersion};
use crate::tlb_adapters::{DictKeyAdapterInto, DictValAdapterTLB, TLBHashMap};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use ton_lib_core::bail_ton_core_data;
use ton_lib_core::cell::{CellBuilder, CellParser, TonCell, TonCellRef, TonHash};
use ton_lib_core::errors::TonCoreError;
use ton_lib_core::traits::tlb::TLB;

// https://github.com/ton-blockchain/ton/blame/6f745c04daf8861bb1791cffce6edb1beec62204/crypto/block/block.tlb#L543
#[derive(Debug, Default)]
pub struct ConfigParams {
    pub config_addr: TonHash,
    pub config: HashMap<u32, TonCellRef>,
    storage_prices: RwLock<Option<Arc<ConfigParam18>>>,
    global_version: RwLock<Option<Arc<GlobalVersion>>>,
}

#[rustfmt::skip]
impl ConfigParams {
    // lazy_load for params
    pub fn storage_prices(&self) -> Result<Arc<ConfigParam18>, TonCoreError> { self.load_param(18, &self.storage_prices) }
    pub fn global_version(&self) -> Result<Arc<GlobalVersion>, TonCoreError> { self.load_param(8, &self.global_version) }

    fn load_param<T: TLB>(&self, index: u32, dst: &RwLock<Option<Arc<T>>>) -> Result<Arc<T>, TonCoreError> {
        if let Some(param) = dst.read().deref() {
            return Ok(param.clone());
        }

        let mut lock = dst.write();
        if let Some(param) = lock.deref() {
            return Ok(param.clone());
        }
        let value = match self.config.get(&index) {
            Some(cell) => Arc::new(T::from_cell(cell)?),
            None => bail_ton_core_data!("Config param with index {index} not found"),
        };
        *lock = Some(value.clone());
        Ok(value)
    }
}

impl PartialEq for ConfigParams {
    fn eq(&self, other: &Self) -> bool { self.config_addr == other.config_addr && self.config == other.config }
}

impl Clone for ConfigParams {
    fn clone(&self) -> Self {
        Self {
            config_addr: self.config_addr.clone(),
            config: self.config.clone(),
            storage_prices: RwLock::new(None),
            global_version: RwLock::new(None),
        }
    }
}

impl TLB for ConfigParams {
    fn read_definition(parser: &mut CellParser) -> Result<Self, TonCoreError> {
        let config_addr = TLB::read(parser)?;
        let config_ref = parser.read_next_ref()?;
        let config =
            TLBHashMap::<DictKeyAdapterInto, DictValAdapterTLB, _, _>::new(32).read(&mut config_ref.parser())?;
        Ok(Self {
            config_addr,
            config,
            ..Default::default()
        })
    }

    fn write_definition(&self, dst: &mut CellBuilder) -> Result<(), TonCoreError> {
        self.config_addr.write(dst)?;
        let mut config_cell = TonCell::builder();
        TLBHashMap::<DictKeyAdapterInto, DictValAdapterTLB, _, _>::new(32).write(&mut config_cell, &self.config)?;
        dst.write_ref(config_cell.build()?.into_ref())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::block_tlb::_test_block_data::CONFIG_BOC_HEX;
    use crate::block_tlb::{ConfigParam18, ConfigParams, GlobalVersion, StoragePrices};
    use std::collections::HashMap;
    use std::ops::Deref;
    use ton_lib_core::traits::tlb::TLB;

    #[test]
    fn test_config_params() -> anyhow::Result<()> {
        let config_params = ConfigParams::from_boc_hex(CONFIG_BOC_HEX)?;
        let serialized = config_params.to_boc()?;
        let parsed_back = ConfigParams::from_boc(&serialized)?;
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
}
