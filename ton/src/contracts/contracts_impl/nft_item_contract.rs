use crate::block_tlb::TVMStack;
use crate::contracts::NFTCollectionMethods;
use crate::contracts::{NFTCollectionContract, TonContract};
use crate::errors::TonError;
use crate::tep::metadata::MetadataContent;
use crate::tep::tvm_results::GetNFTDataResult;
use crate::ton_contract;
use async_trait::async_trait;
use ton_core::errors::TonCoreError;

ton_contract!(NFTItemContract: NFTItemMethods);

impl NFTItemContract {
    pub async fn ext_load_full_nft_data(&self) -> Result<GetNFTDataResult, TonCoreError> {
        let mut data = self.get_nft_data().await?;
        let MetadataContent::Unsupported(meta) = data.individual_content else {
            return Ok(data);
        };

        let collection_address = &data.collection_address;
        let collection = NFTCollectionContract::new(self.get_client(), collection_address, None).await?;
        let full_content = collection.get_nft_content(data.index, meta.cell).await?;
        data.individual_content = full_content.full_content;
        Ok(data)
    }
}

#[async_trait]
pub trait NFTItemMethods: TonContract {
    async fn get_nft_data(&self) -> Result<GetNFTDataResult, TonError> {
        self.emulate_get_method("get_nft_data", &TVMStack::EMPTY, None).await
    }
}
