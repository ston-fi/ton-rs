use crate::bail_ton_core_data;
use crate::errors::TonCoreError;
use crate::errors::TonCoreResult;
use crate::types::tlb_core::VarLenBytes;
use num_traits::ToPrimitive;
use std::fmt::Debug;
use ton_macros::TLB;

/// https://github.com/ton-blockchain/ton/blob/050a984163a53df16fb03f66cc445c34bfed48ed/crypto/block/block.tlb#L116
#[derive(Clone, Copy, Debug, PartialEq, Eq, TLB)]
pub struct TLBCoins(VarLenBytes<u128, 4>);

impl TLBCoins {
    pub const ZERO: TLBCoins = TLBCoins(VarLenBytes {
        data: 0u128,
        bits_len: 0,
    });
    pub const ONE: TLBCoins = TLBCoins(VarLenBytes {
        data: 1u128,
        bits_len: 8,
    });

    pub const fn new(amount: u128) -> Self {
        let bits_len = (128 - amount.leading_zeros()).div_ceil(8) * 8;
        Self(VarLenBytes::from_value(amount, bits_len as usize))
    }

    pub fn from_num<T: ToPrimitive + Debug>(value: &T) -> TonCoreResult<Self> {
        match value.to_u128() {
            Some(v) => Ok(Self::new(v)),
            None => bail_ton_core_data!("Cannot convert given value {value:?} to TLBCoins: to_u128 failed"),
        }
    }

    pub fn to_u32(&self) -> TonCoreResult<u32> {
        match self.0.to_u32() {
            Some(v) => Ok(v),
            None => bail_ton_core_data!("Can't convert {} to u32", self.0.data),
        }
    }

    pub fn to_u64(&self) -> TonCoreResult<u64> {
        match self.0.to_u64() {
            Some(v) => Ok(v),
            None => bail_ton_core_data!("Can't convert {} to u64", self.0.data),
        }
    }

    pub fn to_u128(&self) -> u128 { self.0.data }
}

mod traits_impl {
    use crate::errors::TonCoreError;
    use crate::types::tlb_core::TLBCoins;
    use std::ops::{Deref, DerefMut};
    use std::str::FromStr;

    impl Deref for TLBCoins {
        type Target = u128;
        fn deref(&self) -> &Self::Target { &self.0 }
    }

    impl DerefMut for TLBCoins {
        fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
    }

    impl<T: Into<u128>> From<T> for TLBCoins {
        fn from(value: T) -> Self { TLBCoins::new(value.into()) }
    }

    impl FromStr for TLBCoins {
        type Err = TonCoreError;
        fn from_str(coins: &str) -> Result<Self, Self::Err> { Ok(Self::new(u128::from_str(coins)?)) }
    }

    impl Default for TLBCoins {
        fn default() -> Self { TLBCoins::ZERO }
    }
}
