use crate::block_tlb::TVMStack;
use crate::contracts::ton_contract::TonContract;
use crate::errors::TonError;
use crate::tep::tvm_results::GetWalletDataResult;
use crate::tep::tvm_results::TVMResult;
use async_trait::async_trait;

#[async_trait]
pub trait GetWalletData: TonContract {
    async fn get_wallet_data(&self) -> Result<GetWalletDataResult, TonError> {
        let stack_boc = self.emulate_get_method("get_wallet_data", &TVMStack::EMPTY).await?;
        Ok(GetWalletDataResult::from_boc(&stack_boc)?)
    }
}
