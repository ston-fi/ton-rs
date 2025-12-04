use crate::bail_ton_core_data;
use crate::errors::{TonCoreError, TonCoreResult};
use num_traits::{ToPrimitive, Zero};
use std::fmt::Debug;

/// A safe wrapper around u128 to represent coin amounts with checked arithmetic operations
/// Supports conversion from various numeric types and TLBCoins
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Coins(u128);

pub trait IntoCoins {
    fn into_coins(self) -> TonCoreResult<Coins>;
}

impl Coins {
    pub const ZERO: Coins = Coins(0);
    pub const ONE: Coins = Coins(1);

    pub const fn new(amount: u128) -> Self { Self(amount) }

    pub fn from_num<T: ToPrimitive + Debug>(amount: T) -> TonCoreResult<Self> {
        let Some(amount_u128) = amount.to_u128() else {
            bail_ton_core_data!("Can't convert {amount:?} to u128 for SCoins from_num")
        };
        Ok(Coins(amount_u128))
    }

    pub fn inner(&self) -> u128 { self.0 }

    pub fn checked_add<T: IntoCoins>(&self, other: T) -> TonCoreResult<Coins> {
        let other_coins = other.into_coins()?;
        let Some(res) = self.0.checked_add(other_coins.0) else {
            bail_ton_core_data!("overflow: {} + {}", self.0, other_coins.0)
        };
        Ok(Coins::new(res))
    }

    pub fn checked_add_assign<T: IntoCoins>(&mut self, other: T) -> TonCoreResult<()> {
        *self = self.checked_add(other)?;
        Ok(())
    }

    pub fn checked_sub<T: IntoCoins>(&self, other: T) -> TonCoreResult<Coins> {
        let other_coins = other.into_coins()?;
        let Some(res) = self.0.checked_sub(other_coins.0) else {
            bail_ton_core_data!("underflow: {} - {}", self.0, other_coins.0)
        };
        Ok(Coins::new(res))
    }

    pub fn checked_sub_assign<T: IntoCoins>(&mut self, other: T) -> TonCoreResult<()> {
        *self = self.checked_sub(other)?;
        Ok(())
    }

    pub fn checked_mul<T: IntoCoins>(&self, other: T) -> TonCoreResult<Coins> {
        let other_coins = other.into_coins()?;
        let Some(res) = self.0.checked_mul(other_coins.0) else {
            bail_ton_core_data!("overflow: {} * {}", self.0, other_coins.0)
        };
        Ok(Coins::new(res))
    }

    pub fn checked_mul_assign<T: IntoCoins>(&mut self, other: T) -> TonCoreResult<()> {
        *self = self.checked_mul(other)?;
        Ok(())
    }

    pub fn checked_div<T: IntoCoins>(&self, other: T) -> TonCoreResult<Coins> {
        let other_coins = other.into_coins()?.0;
        if other_coins.is_zero() {
            bail_ton_core_data!("division by zero: {} / 0", self.0);
        }
        let Some(res) = self.0.checked_div(other_coins) else {
            bail_ton_core_data!("div error: {} / {}", self.0, other_coins)
        };
        Ok(Coins::new(res))
    }

    pub fn checked_div_assign<T: IntoCoins>(&mut self, other: T) -> TonCoreResult<()> {
        *self = self.checked_div(other)?;
        Ok(())
    }
}

#[rustfmt::skip]
mod traits_impl {
    use fastnum::*;
    use num_bigint::{BigInt, BigUint};
    use crate::types::tlb_core::TLBCoins;
    use super::*;

    impl From<TLBCoins> for Coins {fn from(value: TLBCoins) -> Self { Coins::new(value.to_u128()) }}
    impl From<Coins> for TLBCoins {fn from(val: Coins) -> Self { TLBCoins::new(val.0) }}
    // impl From<SCoins> for SCoins {fn from(v: SCoins) -> Self { v }}

    impl ToPrimitive for Coins {
        fn to_i64(&self) -> Option<i64> { self.0.to_i64() }
        fn to_i128(&self) -> Option<i128> { self.0.to_i128() }
        fn to_u64(&self) -> Option<u64> { self.0.to_u64() }
        fn to_u128(&self) -> Option<u128> { Some(self.0) }
    }

    macro_rules! try_from_impl {
        ($($t:ty),*) => {
            $(
                impl TryFrom<$t> for Coins {
                    type Error = TonCoreError;
                    fn try_from(value: $t) -> Result<Self, Self::Error> {
                        Coins::from_num(value)
                    }
                }
            )*
        };

    }

    try_from_impl!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, usize, f32, f64, BigInt, BigUint, I128, I256, I512, I1024, U128, U256, U512, U1024, D256, D512);

    impl<T> IntoCoins for T
    where
        T: TryInto<Coins, Error = TonCoreError>,
    {
        fn into_coins(self) -> TonCoreResult<Coins> {
            self.try_into()
        }
    }

    impl IntoCoins for Coins {
        fn into_coins(self) -> TonCoreResult<Coins> {
            Ok(self)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coins_math() -> TonCoreResult<()> {
        let a = Coins::new(1_000_000);
        let b = Coins::new(500_000);

        let c = a.checked_add(b)?;
        assert_eq!(c, Coins::new(1_500_000));

        let d = a.checked_sub(500_000)?;
        assert_eq!(d, Coins::new(500_000));

        let e = a.checked_mul(3)?;
        assert_eq!(e, Coins::new(3_000_000));

        let f = a.checked_div(Coins::new(2))?;
        assert_eq!(f, Coins::new(500_000));

        Ok(())
    }
}
