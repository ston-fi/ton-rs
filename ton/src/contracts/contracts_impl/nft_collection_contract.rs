use crate::block_tlb::TVMStack;
use crate::contracts::TonContract;
use crate::errors::TonResult;
use crate::tep::tvm_result::{GetCollectionDataResult, GetNFTAddressByIndexResult, GetNFTContentResult};
use crate::ton_contract;
use async_trait::async_trait;
use fastnum::I512;
use ton_core::cell::{CellParser, TonCell};
use ton_core::errors::TonCoreResult;
use ton_core::traits::tlb::TLB;
use ton_core::types::TonAddress;
use ton_macros::ton_method;

ton_contract!(NFTCollectionContract: NFTCollectionMethods);

#[async_trait]
pub trait NFTCollectionMethods: TonContract {
    #[ton_method]
    async fn get_collection_data(&self) -> TonResult<GetCollectionDataResult>;

    #[ton_method]
    async fn get_nft_content<T: Into<I512> + Send>(&self, index: T, individual_content: TonCell) -> TonResult<GetNFTContentResult>;

    #[ton_method]
    async fn get_nft_address_by_index<T: Into<I512> + Send>(&self, index: T) -> TonResult<GetNFTAddressByIndexResult>;
}
