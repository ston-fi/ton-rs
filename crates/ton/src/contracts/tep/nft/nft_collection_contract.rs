//! NFT collection contract wrapper, get-method trait, and result types.

use crate::contracts::TonContract;
use crate::contracts::tep::metadata::metadata_content::MetadataContent;
use crate::errors::TonResult;
use crate::ton_contract;
use async_trait::async_trait;
use fastnum::I512;
use ton_core::cell::TonCell;
use ton_core::types::TonAddress;
use ton_macros::{FromTVMStack, ton_methods};

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

/// Result of `get_collection_data`.
#[derive(Debug, Clone, PartialEq, FromTVMStack)]
#[from_tvm_stack(ensure_empty = true)]
#[non_exhaustive]
pub struct GetCollectionDataResult {
    pub next_item_index: i64,
    pub collection_content: MetadataContent,
    pub owner_address: TonAddress,
}

/// Result of `get_nft_address_by_index`.
#[derive(Debug, Clone, PartialEq, Eq, FromTVMStack)]
#[non_exhaustive]
pub struct GetNFTAddressByIndexResult {
    pub nft_address: TonAddress,
}

/// Result of `get_nft_content`.
#[derive(Debug, Clone, PartialEq, FromTVMStack)]
#[from_tvm_stack(ensure_empty = true)]
#[non_exhaustive]
pub struct GetNFTContentResult {
    pub full_content: MetadataContent,
}

#[cfg(test)]
#[path = "../../../tests/contracts/tep/nft/test_nft_collection_contract.rs"]
mod tests;
