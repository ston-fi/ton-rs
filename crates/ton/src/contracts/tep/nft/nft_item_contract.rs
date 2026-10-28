//! NFT item contract wrapper, get-method trait, and result type.

use super::nft_collection_contract::{NFTCollectionContract, NFTCollectionMethods};
use crate::contracts::TonContract;
use crate::contracts::tep::metadata::metadata_content::MetadataContent;
use crate::errors::TonResult;
use crate::ton_contract;
use async_trait::async_trait;
use fastnum::I512;
use ton_core::types::TonAddress;
use ton_macros::{FromTVMStack, ton_methods};

ton_contract!(NFTItemContract: NFTItemMethods);

impl NFTItemContract {
    /// Loads NFT data and resolves collection-relative content.
    pub async fn ext_load_full_nft_data(&self) -> TonResult<GetNFTDataResult> {
        let mut data = self.get_nft_data().await?;
        let MetadataContent::Unsupported(meta) = data.individual_content else {
            return Ok(data);
        };

        let collection_address = &data.collection_address;
        let collection = NFTCollectionContract::new(self.get_client(), collection_address, None);
        let full_content = collection.get_nft_content(data.index, meta.cell).await?;
        data.individual_content = full_content.full_content;
        Ok(data)
    }
}

#[async_trait]
#[ton_methods]
pub trait NFTItemMethods: TonContract {
    async fn get_nft_data(&self) -> TonResult<GetNFTDataResult>;
}

/// Result of `get_nft_data`.
#[derive(Debug, Clone, PartialEq, FromTVMStack)]
#[from_tvm_stack(ensure_empty = true)]
#[non_exhaustive]
pub struct GetNFTDataResult {
    pub init: bool,
    pub index: I512,
    pub collection_address: TonAddress,
    pub owner_address: TonAddress,
    pub individual_content: MetadataContent,
}

#[cfg(test)]
#[path = "../../../tests/contracts/tep/nft/test_nft_item_contract.rs"]
mod tests;
