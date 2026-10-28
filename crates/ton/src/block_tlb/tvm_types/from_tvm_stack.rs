use crate::bail_ton;
use crate::block_tlb::{TVMStack, TVMStackValue};
use crate::contracts::tep::snake_data::SnakeData;
use crate::errors::{TonError, TonResult};
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use fastnum::*;
use std::sync::Arc;
use ton_core::cell::{TonCell, TonHash};
use ton_core::traits::tlb::TLB;
use ton_core::types::tlb_core::{MsgAddress, MsgAddressInt, TLBCoins, TLBRef};
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

impl FromTVMStack for TVMStack {
    fn from_stack(stack: &mut TVMStack) -> TonResult<Self> {
        Ok(stack.clone())
    }
}

impl FromTVMStack for bool {
    fn from_stack(stack: &mut TVMStack) -> TonResult<Self> {
        Ok(stack.pop_num()? != I512::ZERO)
    }
}

impl FromTVMStack for I512 {
    fn from_stack(stack: &mut TVMStack) -> TonResult<Self> {
        stack.pop_num()
    }
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
    fn from_stack(stack: &mut TVMStack) -> TonResult<Self> {
        stack.pop_cell()
    }
}

// A referenced raw-cell result still occupies one Cell/CellSlice stack entry.
impl FromTVMStack for TLBRef<TonCell> {
    fn from_stack(stack: &mut TVMStack) -> TonResult<Self> {
        Ok(Self::new(TonCell::from_stack(stack)?))
    }
}

impl FromTVMStack for TonAddress {
    fn from_stack(stack: &mut TVMStack) -> TonResult<Self> {
        let cell = match stack.pop_checked()? {
            TVMStackValue::Null(_) => return Ok(TonAddress::ZERO),
            TVMStackValue::Cell(cell) => cell.value.into_inner(),
            TVMStackValue::CellSlice(slice) => slice.to_cell()?,
            rest => {
                bail_ton!("Can't parse TonAddress from TVMStack: Expecting Cell, CellSlice or Null. Got {:?}", rest)
            },
        };
        Ok(TonAddress::from_cell(&cell)?)
    }
}

impl FromTVMStack for MsgAddress {
    fn from_stack(stack: &mut TVMStack) -> TonResult<Self> {
        let cell = match stack.pop_checked()? {
            TVMStackValue::Null(_) => return Ok(MsgAddress::NONE),
            TVMStackValue::Cell(cell) => cell.value.into_inner(),
            TVMStackValue::CellSlice(slice) => slice.to_cell()?,
            rest => {
                bail_ton!("Can't parse MsgAddress from TVMStack: Expecting Cell, CellSlice or Null. Got {:?}", rest)
            },
        };
        Ok(MsgAddress::from_cell(&cell)?)
    }
}

impl FromTVMStack for MsgAddressInt {
    fn from_stack(stack: &mut TVMStack) -> TonResult<Self> {
        match MsgAddress::from_stack(stack)? {
            MsgAddress::Int(address) => Ok(address),
            other => Err(TonError::UnexpectedValue {
                expected: "MsgAddressInt".into(),
                actual: format!("{other:?}"),
            }),
        }
    }
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
            },
        };
        Ok(hash)
    }
}

impl FromTVMStack for Coins {
    fn from_stack(stack: &mut TVMStack) -> TonResult<Self> {
        Ok(Coins::try_from(stack.pop_num()?)?)
    }
}

impl FromTVMStack for TLBCoins {
    fn from_stack(stack: &mut TVMStack) -> TonResult<Self> {
        Ok(TLBCoins::from_cell(&stack.pop_cell()?)?)
    }
}

impl FromTVMStack for SnakeData {
    fn from_stack(stack: &mut TVMStack) -> TonResult<Self> {
        Ok(SnakeData::from_cell(&stack.pop_cell()?)?)
    }
}

impl FromTVMStack for String {
    fn from_stack(stack: &mut TVMStack) -> TonResult<Self> {
        Ok(SnakeData::from_stack(stack)?.as_str().to_string())
    }
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
#[path = "../../tests/block_tlb/tvm_types/test_from_tvm_stack.rs"]
mod tests;
