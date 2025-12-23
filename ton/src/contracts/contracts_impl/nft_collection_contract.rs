use crate::contracts::TonContract;
use crate::errors::TonResult;
use crate::tep::tvm_result::{GetCollectionDataResult, GetNFTAddressByIndexResult, GetNFTContentResult};
use crate::ton_contract;
use async_trait::async_trait;
use fastnum::I512;
use ton_core::cell::TonCell;
use ton_macros::ton_methods;

ton_contract!(NFTCollectionContract: NFTCollectionMethods);

#[async_trait]
#[ton_methods]
pub trait NFTCollectionMethods: TonContract {
    async fn get_collection_data(&self) -> TonResult<GetCollectionDataResult>;
    async fn get_nft_content<T>(&self, index: T, individual_content: TonCell) -> TonResult<GetNFTContentResult>
    where
        T: Into<I512> + Send;

    async fn get_nft_address_by_index<T: Into<I512> + Send>(&self, index: T) -> TonResult<GetNFTAddressByIndexResult>;
}
