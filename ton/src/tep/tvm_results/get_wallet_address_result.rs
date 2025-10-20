use crate::block_tlb::TVMStack;
use crate::tep::tvm_results::tvm_result::TVMResult;
use ton_core::errors::TonCoreError;
use ton_core::traits::tlb::TLB;
use ton_core::types::TonAddress;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetWalletAddressResult {
    pub address: TonAddress,
}

impl TVMResult for GetWalletAddressResult {
    fn from_stack(stack: &mut TVMStack) -> Result<Self, TonCoreError> {
        let address = TonAddress::from_cell(&stack.pop_cell()?)?;
        Ok(Self { address })
    }
}
