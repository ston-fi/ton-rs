use crate::bail_ton;
use crate::errors::TonError;
use crate::errors::TonResult;
use crate::tlb_adapters::DictKeyAdapterUint;
use crate::tlb_adapters::DictValAdapterTLB;
use crate::tlb_adapters::TLBHashMapE;
use num_bigint::BigUint;
use num_traits::ToPrimitive;
use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use std::str::FromStr;
use ton_core::TLB;
use ton_core::types::TonExtraCurrencyId;
use ton_core::types::tlb_core::VarLenBytes;

/// https://github.com/ton-blockchain/ton/blob/050a984163a53df16fb03f66cc445c34bfed48ed/crypto/block/block.tlb#L116
#[derive(Clone, Copy, Debug, PartialEq, Eq, TLB)]
pub struct Coins(VarLenBytes<u128, 4>);

/// https://github.com/ton-blockchain/ton/blob/050a984163a53df16fb03f66cc445c34bfed48ed/crypto/block/block.tlb#L124
#[derive(Default, Clone, Debug, PartialEq, TLB)]
pub struct CurrencyCollection {
    pub coins: Coins,
    #[tlb(adapter = "TLBHashMapE::<DictKeyAdapterUint<_>, DictValAdapterTLB<_>>::new(32)")]
    pub other: HashMap<TonExtraCurrencyId, VarLenBytes<BigUint, 5>>,
}

impl Coins {
    pub const ZERO: Coins = Coins(VarLenBytes {
        data: 0u128,
        bits_len: 0,
    });
    pub const ONE: Coins = Coins(VarLenBytes {
        data: 1u128,
        bits_len: 8,
    });

    pub const fn new(amount: u128) -> Self {
        let bits_len = (128 - amount.leading_zeros()).div_ceil(8) * 8;
        Self(VarLenBytes::from_value(amount, bits_len as usize))
    }

    pub fn from_num<T: ToPrimitive + Debug>(value: &T) -> TonResult<Self> {
        match value.to_u128() {
            Some(v) => Ok(Self::new(v)),
            None => bail_ton!("Cannot convert given value {value:?} to Coins: to_u128 failed"),
        }
    }

    pub fn to_u32(&self) -> TonResult<u32> {
        match self.0.to_u32() {
            Some(v) => Ok(v),
            None => bail_ton!("Can't convert {} to u32", self.0.data),
        }
    }

    pub fn to_u64(&self) -> TonResult<u64> {
        match self.0.to_u64() {
            Some(v) => Ok(v),
            None => bail_ton!("Can't convert {} to u64", self.0.data),
        }
    }

    pub fn to_u128(&self) -> u128 { self.0.data }
}

impl CurrencyCollection {
    pub fn new(coins: Coins) -> Self {
        Self {
            coins,
            other: Default::default(),
        }
    }

    pub fn from_num<T: ToPrimitive + Debug>(grams: &T) -> TonResult<Self> {
        Ok(Self {
            coins: Coins::from_num(grams)?,
            other: Default::default(),
        })
    }
}

mod traits_impl {
    use super::*;

    impl FromStr for CurrencyCollection {
        type Err = TonError;
        fn from_str(grams: &str) -> Result<Self, Self::Err> { Self::from_num(&u128::from_str(grams)?) }
    }

    impl Deref for Coins {
        type Target = u128;
        fn deref(&self) -> &Self::Target { &self.0 }
    }

    impl DerefMut for Coins {
        fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
    }

    impl<T: Into<u128>> From<T> for Coins {
        fn from(value: T) -> Self { Coins::new(value.into()) }
    }

    impl FromStr for Coins {
        type Err = TonError;
        fn from_str(grams: &str) -> Result<Self, Self::Err> { Ok(Self::new(u128::from_str(grams)?)) }
    }

    impl Default for Coins {
        fn default() -> Self { Coins::ZERO }
    }
}

#[cfg(test)]
mod tests {
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
    fn test_currency_collection_zero_grams() -> anyhow::Result<()> {
        let currency = CurrencyCollection::from_num(&0u32)?;
        let cell = currency.to_cell()?;
        let parsed = CurrencyCollection::from_cell(&cell)?;
        assert_eq!(parsed.coins, 0u32.into());

        let cell_serial = parsed.to_cell()?;
        assert_eq!(cell_serial, cell);
        Ok(())
    }
}
