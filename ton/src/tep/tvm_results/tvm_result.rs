use crate::block_tlb::TVMStack;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use ton_lib_core::errors::TonCoreError;
use ton_lib_core::traits::tlb::TLB;

#[rustfmt::skip]
pub trait TVMResult: Sized {
    fn from_stack(stack: &mut TVMStack) -> Result<Self, TonCoreError>;
    fn from_boc(boc: &[u8]) -> Result<Self, TonCoreError> { Self::from_stack(&mut TVMStack::from_boc(boc)?) }
    fn from_boc_hex(boc: &str) -> Result<Self, TonCoreError> { Self::from_boc(&hex::decode(boc)?) }
    fn from_boc_b64(boc: &str) -> Result<Self, TonCoreError> { Self::from_boc(&BASE64_STANDARD.decode(boc)?) }
}
