use crate::block_tlb::TVMStack;
use crate::errors::TonResult;
use crate::tep::tvm_results::tvm_result::TVMResult;
use ton_core::TVMResult;
use ton_core::types::TonAddress;
use ton_macros::TLB;

#[derive(Debug, Clone, PartialEq, Eq, TVMResult, TLB)]
#[tvm_result(ensure_empty = true)]
pub struct GetWalletAddressResult {
    pub address: TonAddress,
}
