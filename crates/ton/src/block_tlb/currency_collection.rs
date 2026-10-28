use crate::errors::TonError;
use crate::errors::TonResult;
use crate::tlb_adapters::DictKeyAdapterUint;
use crate::tlb_adapters::DictValAdapterTLB;
use crate::tlb_adapters::TLBHashMapE;
use num_bigint::BigUint;
use num_traits::ToPrimitive;
use std::collections::HashMap;
use std::fmt::Debug;
use std::str::FromStr;
use ton_core::TLB;
use ton_core::types::TonExtraCurrencyId;
use ton_core::types::tlb_core::TLBCoins;
use ton_core::types::tlb_core::VarLenBytes;

/// <https://github.com/ton-blockchain/ton/blob/050a984163a53df16fb03f66cc445c34bfed48ed/crypto/block/block.tlb#L124>
#[derive(Default, Clone, Debug, PartialEq, TLB)]
pub struct CurrencyCollection {
    pub coins: TLBCoins,
    #[tlb(adapter = "TLBHashMapE::<DictKeyAdapterUint<_>, DictValAdapterTLB<_>>::new(32)")]
    pub other: HashMap<TonExtraCurrencyId, VarLenBytes<BigUint, 5>>,
}

impl CurrencyCollection {
    /// Creates a collection with no extra currencies.
    pub fn new(coins: TLBCoins) -> Self {
        Self {
            coins,
            other: Default::default(),
        }
    }

    pub fn from_num<T: ToPrimitive + Debug>(coins: &T) -> TonResult<Self> {
        Ok(Self {
            coins: TLBCoins::from_num(coins)?,
            other: Default::default(),
        })
    }
}

mod traits_impl {
    use super::*;

    impl FromStr for CurrencyCollection {
        type Err = TonError;
        fn from_str(coins: &str) -> Result<Self, Self::Err> {
            Self::from_num(&u128::from_str(coins)?)
        }
    }
}

#[cfg(test)]
#[path = "../tests/block_tlb/test_currency_collection.rs"]
mod tests;
