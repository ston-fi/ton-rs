use crate::block_tlb::TVMStack;
use crate::errors::TonResult;
use crate::tep::tvm_results::tvm_result::TVMResult;
use ton_core::traits::tlb::TLB;
use ton_core::types::TonAddress;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetWalletAddressResult {
    pub address: TonAddress,
}

impl TVMResult for GetWalletAddressResult {
    fn from_stack(stack: &mut TVMStack) -> TonResult<Self> {
        let address = TonAddress::from_cell(&stack.pop_cell()?)?;
        Ok(Self { address })
    }
}
