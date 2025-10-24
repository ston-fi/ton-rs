use crate::block_tlb::TVMStack;
use crate::contracts::TonContract;
use crate::errors::TonError;
use crate::tep::tvm_results::{GetNFTDataResult, TVMResult};
use async_trait::async_trait;

#[async_trait]
pub trait NFTItemMethods: TonContract {
    async fn get_nft_data(&self) -> Result<GetNFTDataResult, TonError> {
        let stack_boc = self.emulate_get_method("get_nft_data", &TVMStack::EMPTY).await?;
        Ok(GetNFTDataResult::from_boc(stack_boc)?)
    }
}
