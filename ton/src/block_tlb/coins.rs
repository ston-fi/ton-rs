use crate::tlb_adapters::DictKeyAdapterInto;
use crate::tlb_adapters::DictValAdapterTLB;
use crate::tlb_adapters::TLBHashMapE;
use num_bigint::BigUint;
use num_traits::ToPrimitive;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::str::FromStr;
use ton_lib_core::errors::TonCoreError;
use ton_lib_core::types::tlb_core::VarLenBytes;
use ton_lib_core::{bail_ton_core_data, TLB};

/// https://github.com/ton-blockchain/ton/blob/050a984163a53df16fb03f66cc445c34bfed48ed/crypto/block/block.tlb#L116
#[derive(Clone, Debug, PartialEq, Eq, TLB)]
pub struct Coins(VarLenBytes<u128, 4>);

/// https://github.com/ton-blockchain/ton/blob/050a984163a53df16fb03f66cc445c34bfed48ed/crypto/block/block.tlb#L124
#[derive(Default, Clone, Debug, PartialEq, TLB)]
pub struct CurrencyCollection {
    pub grams: Coins,
    #[tlb(adapter = "TLBHashMapE::<DictKeyAdapterInto, DictValAdapterTLB, _, _>::new(32)")]
    pub other: HashMap<u32, VarLenBytes<BigUint, 5>>,
}

impl Coins {
    pub const ZERO: Coins = Coins(VarLenBytes {
        data: 0u128,
        bits_len: 0,
    });

    pub fn new<T: Into<u128>>(amount: T) -> Self {
        let amount = amount.into();
        let bits_len = (128 - amount.leading_zeros()).div_ceil(8) * 8;
        Self(VarLenBytes::new(amount, bits_len as usize))
    }
    pub fn from_signed<T: Into<i128>>(amount: T) -> Result<Self, TonCoreError> {
        let amount = amount.into();
        if amount < 0 {
            bail_ton_core_data!("Amount must be positive, got {amount}");
        }
        Ok(Self::new(amount as u128))
    }

    pub fn to_u32(&self) -> Result<u32, TonCoreError> {
        match self.0.to_u32() {
            Some(v) => Ok(v),
            None => bail_ton_core_data!("Can't convert {} to u32", self.0.data),
        }
    }

    pub fn to_u64(&self) -> Result<u64, TonCoreError> {
        match self.0.to_u64() {
            Some(v) => Ok(v),
            None => bail_ton_core_data!("Can't convert {} to u64", self.0.data),
        }
    }

    pub fn to_u128(&self) -> u128 { self.0.data }
}

impl CurrencyCollection {
    pub fn new<T: Into<u128>>(grams: T) -> Self {
        Self {
            grams: Coins::new(grams),
            other: Default::default(),
        }
    }
}

mod traits_impl {
    use super::*;

    impl Copy for Coins {}

    impl FromStr for CurrencyCollection {
        type Err = TonCoreError;
        fn from_str(grams: &str) -> Result<Self, Self::Err> { Ok(Self::new(u128::from_str(grams)?)) }
    }

    impl Deref for Coins {
        type Target = u128;
        fn deref(&self) -> &Self::Target { &self.0 }
    }

    impl DerefMut for Coins {
        fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
    }

    impl<T: Into<u128>> From<T> for Coins {
        fn from(value: T) -> Self { Coins::new(value) }
    }

    impl FromStr for Coins {
        type Err = TonCoreError;
        fn from_str(grams: &str) -> Result<Self, Self::Err> { Ok(Self::new(u128::from_str(grams)?)) }
    }

    impl Default for Coins {
        fn default() -> Self { Coins::ZERO }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ton_lib_core::traits::tlb::TLB;

    #[test]
    fn test_currency_collection() -> anyhow::Result<()> {
        let parsed = CurrencyCollection::from_boc_hex("b5ee9c720101010100070000094c143b1d14")?;
        assert_eq!(parsed.grams, 3242439121u32.into());

        let cell_serial = parsed.to_cell()?;
        let parsed_back = CurrencyCollection::from_cell(&cell_serial)?;
        assert_eq!(parsed, parsed_back);
        Ok(())
    }

    #[test]
    fn test_currency_collection_zero_grams() -> anyhow::Result<()> {
        let currency = CurrencyCollection::new(0u32);
        let cell = currency.to_cell()?;
        let parsed = CurrencyCollection::from_cell(&cell)?;
        assert_eq!(parsed.grams, 0u32.into());

        let cell_serial = parsed.to_cell()?;
        assert_eq!(cell_serial, cell);
        Ok(())
    }
}
