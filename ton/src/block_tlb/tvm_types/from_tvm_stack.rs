use crate::block_tlb::{TVMStack, TVMStackValue};
use crate::errors::{TonError, TonResult};
use crate::tep::snake_data::SnakeData;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use fastnum::*;
use std::sync::Arc;
use ton_core::cell::{TonCell, TonHash};
use ton_core::traits::tlb::TLB;
use ton_core::types::tlb_core::TLBCoins;
use ton_core::types::{Coins, TonAddress};

/// Trait allows reading data from TVMStack
/// stack must be parsed in reverse order compare to tonviewer results
/// use `#[derive(FromTVMStack)]` to auto-generate implementation for structs
#[rustfmt::skip]
pub trait FromTVMStack: Sized {
    fn from_stack(stack: &mut TVMStack) -> TonResult<Self>;
    fn from_stack_boc<T: Into<Arc<Vec<u8>>>>(boc: T) -> TonResult<Self> { Self::from_stack(&mut TVMStack::from_boc(boc)?) }
    fn from_stack_boc_hex(boc: &str) -> TonResult<Self> { Self::from_stack_boc(hex::decode(boc)?) }
    fn from_stack_boc_base64(boc: &str) -> TonResult<Self> { Self::from_stack_boc(BASE64_STANDARD.decode(boc)?) }
}

impl FromTVMStack for bool {
    fn from_stack(stack: &mut TVMStack) -> TonResult<Self> { Ok(stack.pop_num()? != I512::ZERO) }
}

impl FromTVMStack for I512 {
    fn from_stack(stack: &mut TVMStack) -> TonResult<Self> { stack.pop_num() }
}

macro_rules! from_tvm_stack_primitives_impl {
    ($($t:ty),*) => {
        $(
            impl FromTVMStack for $t {
                fn from_stack(stack: &mut TVMStack) -> TonResult<Self> {
                    let num512 = stack.pop_num()?;
                    num512.try_into().map_err(|_| TonError::UnexpectedValue {
                        expected: stringify!($t).to_string(),
                        actual: format!("num {num512}"),
                    })
                }
            }
        )*
    };
}
from_tvm_stack_primitives_impl!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);

macro_rules! from_tvm_stack_fastnum_impl {
    ($($t:ty),*) => {
        $(
            impl FromTVMStack for $t {
                fn from_stack(stack: &mut TVMStack) -> TonResult<Self> {
                    let num512 = stack.pop_num()?;
                    match <$t>::from_be_slice(num512.to_radix_be(256).as_slice()) {
                        Some(v) => Ok(v),
                        None => Err(TonError::UnexpectedValue {
                            expected: stringify!($t).to_string(),
                            actual: format!("num {num512}"),
                        }),
                    }
                }
            }
        )*
    };
}
from_tvm_stack_fastnum_impl!(U128, U256, U512, U1024, I128, I256, I1024);

impl FromTVMStack for TonCell {
    fn from_stack(stack: &mut TVMStack) -> TonResult<Self> { stack.pop_cell() }
}

impl FromTVMStack for TonAddress {
    fn from_stack(stack: &mut TVMStack) -> TonResult<Self> { Ok(TonAddress::from_cell(&stack.pop_cell()?)?) }
}

impl FromTVMStack for TonHash {
    fn from_stack(stack: &mut TVMStack) -> TonResult<Self> {
        let hash = match stack.pop_checked()? {
            TVMStackValue::Int(num) => Self::from_i512(&num.value)?,
            TVMStackValue::TinyInt(num) => Self::from_i512(&I512::from_i64(num.value))?,
            TVMStackValue::Cell(cell) => TonHash::from_cell(&cell.value)?,
            TVMStackValue::CellSlice(cell) => TonHash::from_cell(&cell.value)?,
            rest => {
                return Err(TonError::UnexpectedValue {
                    expected: "TonHash as Int or Cell".to_string(),
                    actual: format!("{rest:?}"),
                });
            }
        };
        Ok(hash)
    }
}

impl FromTVMStack for Coins {
    fn from_stack(stack: &mut TVMStack) -> TonResult<Self> { Ok(Coins::try_from(stack.pop_num()?)?) }
}

impl FromTVMStack for TLBCoins {
    fn from_stack(stack: &mut TVMStack) -> TonResult<Self> { Ok(TLBCoins::from_cell(&stack.pop_cell()?)?) }
}

impl FromTVMStack for SnakeData {
    fn from_stack(stack: &mut TVMStack) -> TonResult<Self> { Ok(SnakeData::from_cell(&stack.pop_cell()?)?) }
}

impl FromTVMStack for String {
    fn from_stack(stack: &mut TVMStack) -> TonResult<Self> { Ok(SnakeData::from_stack(stack)?.as_str().to_string()) }
}

impl<T: FromTVMStack> FromTVMStack for Option<T> {
    fn from_stack(stack: &mut TVMStack) -> TonResult<Self> {
        let Some(last) = stack.last() else {
            return Err(TonError::TVMStackEmpty);
        };
        if last.as_null().is_some() {
            let _ = stack.pop_checked()?; // drop value from stack
            return Ok(None);
        }
        Ok(Some(T::from_stack(stack)?))
    }
}

#[cfg(test)]
mod tests {
    use crate::block_tlb::{FromTVMStack, TVMInt, TVMNull, TVMStack};
    use crate::tep::snake_data::SnakeData;
    use fastnum::I256;
    use std::str::FromStr;
    use tokio_test::assert_err;
    use ton_core::traits::tlb::TLB;
    use ton_core::types::TonAddress;
    use ton_macros::FromTVMStack;

    #[derive(FromTVMStack, Debug)]
    #[from_tvm_stack(ensure_empty = true)]
    struct TestStruct {
        pub field1: i64,
        pub field2: TonAddress,
        pub field3: bool,
    }

    #[test]
    fn test_from_tvm_stack_derive_macro_works() -> anyhow::Result<()> {
        let mut stack = TVMStack::new(vec![]);
        stack.push_tiny_int(1);
        stack.push_cell(TonAddress::from_str("EQBiMfDMivebQb052Z6yR3jHrmwNhw1kQ5bcAUOBYsK_VPuK")?.to_cell()?);
        stack.push_tiny_int(0);

        let test_struct = TestStruct::from_stack(&mut stack)?;

        assert_eq!(test_struct.field1, 1i64);
        assert_eq!(test_struct.field2, TonAddress::from_str("EQBiMfDMivebQb052Z6yR3jHrmwNhw1kQ5bcAUOBYsK_VPuK")?);
        assert!(!test_struct.field3);
        Ok(())
    }

    #[test]
    fn test_from_tvm_stack_ensure_empty_works() -> anyhow::Result<()> {
        let mut stack = TVMStack::new(vec![]);
        stack.push_tiny_int(1);
        stack.push_tiny_int(1);
        stack.push_tiny_int(1);
        stack.push_cell(TonAddress::from_str("EQBiMfDMivebQb052Z6yR3jHrmwNhw1kQ5bcAUOBYsK_VPuK")?.to_cell()?);
        stack.push_tiny_int(0);

        assert_err!(TestStruct::from_stack(&mut stack));
        Ok(())
    }

    #[test]
    fn test_from_tvm_stack_numbers_impl() -> anyhow::Result<()> {
        let val = 42u32;
        let stack = TVMStack::new(vec![TVMInt { value: val.into() }.into()]);

        let parsed_u8 = u8::from_stack(&mut stack.clone())?;
        assert_eq!(parsed_u8, val as u8);

        let parsed_u32 = u32::from_stack(&mut stack.clone())?;
        assert_eq!(parsed_u32, val);

        let parsed_u128 = u128::from_stack(&mut stack.clone())?;
        assert_eq!(parsed_u128, val as u128);

        let parsed_i8 = i8::from_stack(&mut stack.clone())?;
        assert_eq!(parsed_i8, val as i8);
        let parsed_i64 = i64::from_stack(&mut stack.clone())?;
        assert_eq!(parsed_i64, val as i64);
        let parsed_i128 = i128::from_stack(&mut stack.clone())?;
        assert_eq!(parsed_i128, val as i128);

        let parsed_i256 = I256::from_stack(&mut stack.clone())?;
        assert_eq!(parsed_i256, I256::from(val));

        let parsed_u256 = fastnum::U256::from_stack(&mut stack.clone())?;
        assert_eq!(parsed_u256, fastnum::U256::from(val));

        let parsed_u1024 = fastnum::U1024::from_stack(&mut stack.clone())?;
        assert_eq!(parsed_u1024, fastnum::U1024::from(val));
        Ok(())
    }

    #[test]
    fn test_from_tvm_stack_string() -> anyhow::Result<()> {
        let original = "Hello, TVMStack!".to_string();
        let snake_data = SnakeData::from_str(&original)?;
        let cell = snake_data.to_cell()?;

        let mut stack = TVMStack::new(vec![]);
        stack.push_cell(cell);

        let parsed: String = FromTVMStack::from_stack(&mut stack)?;
        assert_eq!(parsed, original);
        Ok(())
    }

    #[test]
    fn test_from_tvm_stack_optional() -> anyhow::Result<()> {
        let mut stack = TVMStack::default();
        stack.push_tiny_int(11);
        stack.push(TVMNull.into());
        stack.push_tiny_int(33);
        #[derive(FromTVMStack, Debug)]
        struct TestStruct {
            pub opt_some: Option<u8>,
            pub opt_none: Option<u8>,
            pub another_field: u8,
        }

        let parsed = TestStruct::from_stack(&mut stack)?;
        assert_eq!(parsed.opt_some, Some(11));
        assert_eq!(parsed.opt_none, None);
        assert_eq!(parsed.another_field, 33);
        Ok(())
    }
}
