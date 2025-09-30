use crate::block_tlb::TVMStack;
use crate::contracts::ton_contract::ContractCtx;
use crate::contracts::ton_contract::TonContract;
use crate::contracts::*;
use crate::errors::TonError;
use crate::tep::metadata::MetadataContent;
use crate::tep::tvm_results::{GetNFTDataResult, TVMResult};
use async_trait::async_trait;
use ton_lib_core::{errors::TonCoreError, ton_contract};

#[ton_contract]
pub struct NFTItemContract;
impl NFTItemMethods for NFTItemContract {}

#[async_trait]
pub trait NFTItemMethods: TonContract {
    async fn get_nft_data(&self) -> Result<GetNFTDataResult, TonError> {
        let stack_boc = self.emulate_get_method("get_nft_data", &TVMStack::EMPTY).await?;
        Ok(GetNFTDataResult::from_boc(&stack_boc)?)
    }

    async fn load_full_nft_data(&self) -> Result<GetNFTDataResult, TonCoreError> {
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
