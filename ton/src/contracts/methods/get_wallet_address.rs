use crate::block_tlb::TVMStack;
use crate::contracts::ton_contract::TonContract;
use crate::error::TLError;
use crate::tep::tvm_results::GetWalletAddressResult;
use async_trait::async_trait;
use ton_lib_core::traits::tlb::TLB;
use ton_lib_core::traits::tvm_result::TVMResult;
use ton_lib_core::types::TonAddress;

#[async_trait]
pub trait GetWalletAddress: TonContract {
    async fn get_wallet_address(&self, owner: &TonAddress) -> Result<GetWalletAddressResult, TLError> {
        let mut stack = TVMStack::default();
        stack.push_cell_slice(owner.to_cell_ref()?);
        let stack_boc = self.emulate_get_method("get_wallet_address", &stack).await?;
        Ok(GetWalletAddressResult::from_boc(&stack_boc)?)
    }
}
