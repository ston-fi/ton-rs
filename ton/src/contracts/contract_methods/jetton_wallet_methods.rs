use crate::block_tlb::TVMStack;
use crate::contracts::TonContract;
use crate::errors::TonError;
use crate::tep::tvm_results::GetWalletDataResult;
use async_trait::async_trait;

#[async_trait]
pub trait JettonWalletMethods: TonContract {
    async fn get_wallet_data(&self) -> Result<GetWalletDataResult, TonError> {
        self.emulate_get_method("get_wallet_data", &TVMStack::EMPTY, None).await
    }
}
