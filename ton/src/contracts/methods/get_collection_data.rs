use crate::block_tlb::TVMStack;
use crate::contracts::ton_contract::TonContract;
use crate::errors::TonError;
use crate::tep::tvm_results::GetCollectionDataResult;
use crate::tep::tvm_results::TVMResult;
use async_trait::async_trait;

#[async_trait]
pub trait GetCollectionData: TonContract {
    async fn get_collection_data(&self) -> Result<GetCollectionDataResult, TonError> {
        let stack_boc = self.emulate_get_method("get_collection_data", &TVMStack::EMPTY).await?;
        Ok(GetCollectionDataResult::from_boc(&stack_boc)?)
    }
}
