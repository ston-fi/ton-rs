use crate::block_tlb::CurrencyCollection;
use ton_core::TLB;
use ton_core::cell::{CellBuilder, CellParser, TonCell};
use ton_core::errors::TonCoreError;
use ton_core::traits::tlb::{TLB, TLBPrefix};

// https://github.com/ton-blockchain/ton/blob/6f745c04daf8861bb1791cffce6edb1beec62204/crypto/block/block.tlb#L474-L497
#[derive(Debug, Clone, PartialEq, TLB)]
pub enum ValueFlow {
    V1(ValueFlowV1),
    V2(ValueFlowV2),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValueFlowV1 {
    pub from_prev_blk: CurrencyCollection,
    pub to_next_blk: CurrencyCollection,
    pub imported: CurrencyCollection,
    pub exported: CurrencyCollection,
    pub fees_collected: CurrencyCollection,
    pub fees_imported: CurrencyCollection,
    pub recovered: CurrencyCollection,
    pub created: CurrencyCollection,
    pub minted: CurrencyCollection,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValueFlowV2 {
    pub from_prev_blk: CurrencyCollection,
    pub to_next_blk: CurrencyCollection,
    pub imported: CurrencyCollection,
    pub exported: CurrencyCollection,
    pub fees_collected: CurrencyCollection,
    pub burned: CurrencyCollection,
    pub fees_imported: CurrencyCollection,
    pub recovered: CurrencyCollection,
    pub created: CurrencyCollection,
    pub minted: CurrencyCollection,
}

impl ValueFlow {
    pub fn as_v1(&self) -> Option<&ValueFlowV1> {
        match self {
            ValueFlow::V1(value_flow) => Some(value_flow),
            ValueFlow::V2(_) => None,
        }
    }

    pub fn as_v2(&self) -> Option<&ValueFlowV2> {
        match self {
            ValueFlow::V1(_) => None,
            ValueFlow::V2(value_flow) => Some(value_flow),
        }
    }

    pub fn from_prev_blk(&self) -> &CurrencyCollection {
        match self {
            ValueFlow::V1(value_flow) => &value_flow.from_prev_blk,
            ValueFlow::V2(value_flow) => &value_flow.from_prev_blk,
        }
    }

    pub fn from_prev_blk_mut(&mut self) -> &mut CurrencyCollection {
        match self {
            ValueFlow::V1(value_flow) => &mut value_flow.from_prev_blk,
            ValueFlow::V2(value_flow) => &mut value_flow.from_prev_blk,
        }
    }

    pub fn to_next_blk(&self) -> &CurrencyCollection {
        match self {
            ValueFlow::V1(value_flow) => &value_flow.to_next_blk,
            ValueFlow::V2(value_flow) => &value_flow.to_next_blk,
        }
    }

    pub fn to_next_blk_mut(&mut self) -> &mut CurrencyCollection {
        match self {
            ValueFlow::V1(value_flow) => &mut value_flow.to_next_blk,
            ValueFlow::V2(value_flow) => &mut value_flow.to_next_blk,
        }
    }

    pub fn imported(&self) -> &CurrencyCollection {
        match self {
            ValueFlow::V1(value_flow) => &value_flow.imported,
            ValueFlow::V2(value_flow) => &value_flow.imported,
        }
    }

    pub fn imported_mut(&mut self) -> &mut CurrencyCollection {
        match self {
            ValueFlow::V1(value_flow) => &mut value_flow.imported,
            ValueFlow::V2(value_flow) => &mut value_flow.imported,
        }
    }

    pub fn exported(&self) -> &CurrencyCollection {
        match self {
            ValueFlow::V1(value_flow) => &value_flow.exported,
            ValueFlow::V2(value_flow) => &value_flow.exported,
        }
    }

    pub fn exported_mut(&mut self) -> &mut CurrencyCollection {
        match self {
            ValueFlow::V1(value_flow) => &mut value_flow.exported,
            ValueFlow::V2(value_flow) => &mut value_flow.exported,
        }
    }

    pub fn fees_collected(&self) -> &CurrencyCollection {
        match self {
            ValueFlow::V1(value_flow) => &value_flow.fees_collected,
            ValueFlow::V2(value_flow) => &value_flow.fees_collected,
        }
    }

    pub fn fees_collected_mut(&mut self) -> &mut CurrencyCollection {
        match self {
            ValueFlow::V1(value_flow) => &mut value_flow.fees_collected,
            ValueFlow::V2(value_flow) => &mut value_flow.fees_collected,
        }
    }

    pub fn burned(&self) -> Option<&CurrencyCollection> {
        match self {
            ValueFlow::V1(_) => None,
            ValueFlow::V2(value_flow) => Some(&value_flow.burned),
        }
    }

    pub fn burned_mut(&mut self) -> Option<&mut CurrencyCollection> {
        match self {
            ValueFlow::V1(_) => None,
            ValueFlow::V2(value_flow) => Some(&mut value_flow.burned),
        }
    }

    pub fn fees_imported(&self) -> &CurrencyCollection {
        match self {
            ValueFlow::V1(value_flow) => &value_flow.fees_imported,
            ValueFlow::V2(value_flow) => &value_flow.fees_imported,
        }
    }

    pub fn fees_imported_mut(&mut self) -> &mut CurrencyCollection {
        match self {
            ValueFlow::V1(value_flow) => &mut value_flow.fees_imported,
            ValueFlow::V2(value_flow) => &mut value_flow.fees_imported,
        }
    }

    pub fn recovered(&self) -> &CurrencyCollection {
        match self {
            ValueFlow::V1(value_flow) => &value_flow.recovered,
            ValueFlow::V2(value_flow) => &value_flow.recovered,
        }
    }

    pub fn recovered_mut(&mut self) -> &mut CurrencyCollection {
        match self {
            ValueFlow::V1(value_flow) => &mut value_flow.recovered,
            ValueFlow::V2(value_flow) => &mut value_flow.recovered,
        }
    }

    pub fn created(&self) -> &CurrencyCollection {
        match self {
            ValueFlow::V1(value_flow) => &value_flow.created,
            ValueFlow::V2(value_flow) => &value_flow.created,
        }
    }

    pub fn created_mut(&mut self) -> &mut CurrencyCollection {
        match self {
            ValueFlow::V1(value_flow) => &mut value_flow.created,
            ValueFlow::V2(value_flow) => &mut value_flow.created,
        }
    }

    pub fn minted(&self) -> &CurrencyCollection {
        match self {
            ValueFlow::V1(value_flow) => &value_flow.minted,
            ValueFlow::V2(value_flow) => &value_flow.minted,
        }
    }

    pub fn minted_mut(&mut self) -> &mut CurrencyCollection {
        match self {
            ValueFlow::V1(value_flow) => &mut value_flow.minted,
            ValueFlow::V2(value_flow) => &mut value_flow.minted,
        }
    }
}

impl TLB for ValueFlowV1 {
    const PREFIX: TLBPrefix = TLBPrefix::new(0xb8e48dfb, 32);

    fn read_definition(parser: &mut CellParser) -> Result<Self, TonCoreError> {
        let left_cell = parser.read_next_ref()?.clone();
        let right_cell = parser.read_next_ref()?.clone();
        let mut left = left_cell.parser();
        let mut right = right_cell.parser();
        Ok(Self {
            from_prev_blk: TLB::read(&mut left)?,
            to_next_blk: TLB::read(&mut left)?,
            imported: TLB::read(&mut left)?,
            exported: TLB::read(&mut left)?,
            fees_collected: TLB::read(parser)?,
            fees_imported: TLB::read(&mut right)?,
            recovered: TLB::read(&mut right)?,
            created: TLB::read(&mut right)?,
            minted: TLB::read(&mut right)?,
        })
    }

    fn write_definition(&self, builder: &mut CellBuilder) -> Result<(), TonCoreError> {
        let mut left = TonCell::builder();
        self.from_prev_blk.write(&mut left)?;
        self.to_next_blk.write(&mut left)?;
        self.imported.write(&mut left)?;
        self.exported.write(&mut left)?;
        builder.write_ref(left.build()?)?;

        self.fees_collected.write(builder)?;

        let mut right = TonCell::builder();
        self.fees_imported.write(&mut right)?;
        self.recovered.write(&mut right)?;
        self.created.write(&mut right)?;
        self.minted.write(&mut right)?;
        builder.write_ref(right.build()?)?;
        Ok(())
    }
}

impl TLB for ValueFlowV2 {
    const PREFIX: TLBPrefix = TLBPrefix::new(0x3ebf98b7, 32);

    fn read_definition(parser: &mut CellParser) -> Result<Self, TonCoreError> {
        let left_cell = parser.read_next_ref()?.clone();
        let right_cell = parser.read_next_ref()?.clone();
        let mut left = left_cell.parser();
        let mut right = right_cell.parser();
        Ok(Self {
            from_prev_blk: TLB::read(&mut left)?,
            to_next_blk: TLB::read(&mut left)?,
            imported: TLB::read(&mut left)?,
            exported: TLB::read(&mut left)?,
            fees_collected: TLB::read(parser)?,
            burned: TLB::read(parser)?,
            fees_imported: TLB::read(&mut right)?,
            recovered: TLB::read(&mut right)?,
            created: TLB::read(&mut right)?,
            minted: TLB::read(&mut right)?,
        })
    }

    fn write_definition(&self, builder: &mut CellBuilder) -> Result<(), TonCoreError> {
        let mut left = TonCell::builder();
        self.from_prev_blk.write(&mut left)?;
        self.to_next_blk.write(&mut left)?;
        self.imported.write(&mut left)?;
        self.exported.write(&mut left)?;
        builder.write_ref(left.build()?)?;

        self.fees_collected.write(builder)?;
        self.burned.write(builder)?;

        let mut right = TonCell::builder();
        self.fees_imported.write(&mut right)?;
        self.recovered.write(&mut right)?;
        self.created.write(&mut right)?;
        self.minted.write(&mut right)?;
        builder.write_ref(right.build()?)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
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
        assert_eq!(
            value_flow_v1.from_prev_blk.other.get(&239u32.into()).unwrap().data,
            BigUint::from_str("2333333332")?
        );
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
}
