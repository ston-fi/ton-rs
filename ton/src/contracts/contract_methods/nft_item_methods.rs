use crate::block_tlb::TVMStack;
use crate::contracts::TonContract;
use crate::errors::TonError;
use crate::tep::tvm_results::{GetNFTDataResult, TVMResult};
use async_trait::async_trait;

#[async_trait]
pub trait NFTItemMethods: TonContract {
    async fn get_nft_data(&self) -> Result<GetNFTDataResult, TonError> {
        self.emulate_get_method::<_, GetNFTDataResult>("get_nft_data", &TVMStack::EMPTY, None).await
    }
}
