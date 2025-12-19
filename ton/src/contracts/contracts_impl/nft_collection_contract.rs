use crate::block_tlb::TVMStack;
use crate::contracts::TonContract;
use crate::errors::TonResult;
use crate::tep::tvm_result::{GetCollectionDataResult, GetNFTAddressByIndexResult, GetNFTContentResult};
use crate::ton_contract;
use async_trait::async_trait;
use fastnum::I512;
use ton_core::cell::TonCell;

ton_contract!(NFTCollectionContract: NFTCollectionMethods);

#[async_trait]
pub trait NFTCollectionMethods: TonContract {
    async fn get_collection_data(&self) -> TonResult<GetCollectionDataResult> {
        self.emulate_get_method("get_collection_data", &TVMStack::EMPTY, None).await
    }

    async fn get_nft_content<T: Into<I512> + Send>(
        &self,
        index: T,
        individual_content: TonCell,
    ) -> TonResult<GetNFTContentResult> {
        let mut stack = TVMStack::default();
        stack.push_int(index.into());
        stack.push_cell(individual_content);

        self.emulate_get_method("get_nft_content", &stack, None).await
    }

    async fn get_nft_address_by_index<T: Into<I512> + Send>(&self, index: T) -> TonResult<GetNFTAddressByIndexResult> {
        let mut stack = TVMStack::default();
        stack.push_int(index.into());

        self.emulate_get_method("get_nft_address_by_index", &stack, None).await
    }
}
