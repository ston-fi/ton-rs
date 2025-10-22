use crate::block_tlb::TVMStack;
use crate::contracts::TonContract;
use crate::errors::TonResult;
use crate::tep::tvm_results::{GetCollectionDataResult, GetNFTAddressByIndexResult, GetNFTContentResult, TVMResult};
use async_trait::async_trait;
use num_bigint::BigInt;
use ton_core::cell::TonCell;

#[async_trait]
pub trait NFTCollectionMethods: TonContract {
    async fn get_collection_data(&self) -> TonResult<GetCollectionDataResult> {
        let stack_boc = self.emulate_get_method("get_collection_data", &TVMStack::EMPTY).await?;
        Ok(GetCollectionDataResult::from_boc(stack_boc)?)
    }

    async fn get_nft_content(&self, index: BigInt, individual_content: TonCell) -> TonResult<GetNFTContentResult> {
        let mut stack = TVMStack::default();
        stack.push_int(index);
        stack.push_cell(individual_content);

        let stack_boc = self.emulate_get_method("get_nft_content", &stack).await?;

        Ok(GetNFTContentResult::from_boc(stack_boc)?)
    }

    async fn get_nft_address_by_index<T: Into<BigInt> + Send>(
        &self,
        index: T,
    ) -> TonResult<GetNFTAddressByIndexResult> {
        let mut stack = TVMStack::default();
        stack.push_int(index.into());

        let stack_boc = self.emulate_get_method("get_nft_address_by_index", &stack).await?;
        Ok(GetNFTAddressByIndexResult::from_boc(stack_boc)?)
    }
}
