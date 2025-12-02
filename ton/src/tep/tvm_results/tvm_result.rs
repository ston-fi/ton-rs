use crate::block_tlb::TVMStack;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use fastnum::I512;
use std::sync::Arc;
use ton_core::cell::TonCell;
use ton_core::errors::TonCoreResult;
use ton_core::traits::tlb::TLB;

#[rustfmt::skip]
pub trait TVMResult: Sized {
    /// stack must be parsed in reverse order compare to tonviewer results
    fn from_stack(stack: &mut TVMStack) -> TonCoreResult<Self>;
    fn from_stack_boc<T: Into<Arc<Vec<u8>>>>(boc: T) -> TonCoreResult<Self> { Self::from_stack(&mut TVMStack::from_boc(boc)?) }
    fn from_stack_boc_hex(boc: &str) -> TonCoreResult<Self> { Self::from_stack_boc(hex::decode(boc)?) }
    fn from_stack_boc_base64(boc: &str) -> TonCoreResult<Self> { Self::from_stack_boc(BASE64_STANDARD.decode(boc)?) }
}

mod trait_impl {
    use super::*;
    use ton_core::types::TonAddress;

    impl TVMResult for bool {
        fn from_stack(stack: &mut TVMStack) -> TonCoreResult<Self> { Ok(stack.pop_number()? != I512::ZERO) }
    }

    impl TVMResult for i64 {
        fn from_stack(stack: &mut TVMStack) -> TonCoreResult<Self> { Ok(stack.pop_tiny_int()?) }
    }

    impl TVMResult for I512 {
        fn from_stack(stack: &mut TVMStack) -> TonCoreResult<Self> { Ok(stack.pop_number()?) }
    }

    impl TVMResult for TonCell {
        fn from_stack(stack: &mut TVMStack) -> TonCoreResult<Self> { Ok(stack.pop_cell()?) }
    }

    impl TVMResult for TonAddress {
        fn from_stack(stack: &mut TVMStack) -> TonCoreResult<Self> { TonAddress::from_cell(&stack.pop_cell()?) }
    }
}
