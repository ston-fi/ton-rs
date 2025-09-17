use crate::tep::tvm_results::GetNFTDataResult;
use crate::{
    contracts::{
        methods::{get_nft_content::GetNFTContent, get_nft_data::GetNFTData},
        nft_collection_contract::NFTCollectionContract,
        ton_contract::{ContractCtx, TonContract},
    },
    tep::metadata::MetadataContent,
};
use ton_lib_core::{errors::TonCoreError, ton_contract};

#[ton_contract]
pub struct NFTItemContract;
impl GetNFTData for NFTItemContract {}

impl NFTItemContract {
    pub async fn load_full_nft_data(&self) -> Result<GetNFTDataResult, TonCoreError> {
        let mut data = self.get_nft_data().await?;
        if let MetadataContent::Unsupported(meta) = data.individual_content {
            let collection =
                NFTCollectionContract::new(&self.ctx().client, data.collection_address.clone(), None).await?;
            let full_content = collection.get_nft_content(data.index.clone(), meta.cell.into_ref()).await?;
            data.individual_content = full_content.full_content;
            Ok(data)
        } else {
            Ok(data)
        }
    }
}
