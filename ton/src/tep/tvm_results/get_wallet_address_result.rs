use crate::block_tlb::TVMStack;
use crate::errors::TonResult;
use crate::tep::tvm_results::tvm_result::TVMResult;
use ton_core::TVMResult;
use ton_core::types::TonAddress;

#[derive(Debug, Clone, PartialEq, Eq, TVMResult)]
pub struct GetWalletAddressResult {
    pub address: TonAddress,
}
