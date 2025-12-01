use crate::block_tlb::TVMStack;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use std::sync::Arc;
use ton_core::errors::TonCoreResult;
use ton_core::traits::tlb::TLB;

#[rustfmt::skip]
pub trait TVMResult: Sized {
    /// stack must be parsed in reverse order compare to tonviewer results
    fn from_stack(stack: &mut TVMStack) -> TonCoreResult<Self>;
    fn from_boc<T: Into<Arc<Vec<u8>>>>(boc: T) -> TonCoreResult<Self> { Self::from_stack(&mut TVMStack::from_boc(boc)?) }
    fn from_boc_hex(boc: &str) -> TonCoreResult<Self> { Self::from_boc(hex::decode(boc)?) }
    fn from_boc_base64(boc: &str) -> TonCoreResult<Self> { Self::from_boc(BASE64_STANDARD.decode(boc)?) }
}
