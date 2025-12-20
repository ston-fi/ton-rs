use crate::contracts::NFTCollectionMethods;
use crate::contracts::{NFTCollectionContract, TonContract};
use crate::errors::TonResult;
use crate::tep::metadata::MetadataContent;
use crate::tep::tvm_result::GetNFTDataResult;
use crate::ton_contract;
use async_trait::async_trait;
use ton_macros::ton_method;

ton_contract!(NFTItemContract: NFTItemMethods);

impl NFTItemContract {
    pub async fn ext_load_full_nft_data(&self) -> TonResult<GetNFTDataResult> {
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
    #[ton_method]
    async fn get_nft_data(&self) -> TonResult<GetNFTDataResult>;
}
