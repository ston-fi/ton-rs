use crate::block_tlb::TVMStack;
use crate::contracts::ton_contract::TonContract;
use crate::errors::TonError;
use crate::tep::tvm_results::GetNFTDataResult;
use crate::tep::tvm_results::TVMResult;
use async_trait::async_trait;

#[async_trait]
pub trait GetNFTData: TonContract {
    async fn get_nft_data(&self) -> Result<GetNFTDataResult, TonError> {
        let stack_boc = self.emulate_get_method("get_nft_data", &TVMStack::EMPTY).await?;
        Ok(GetNFTDataResult::from_boc(&stack_boc)?)
    }
}
