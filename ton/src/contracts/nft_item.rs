use crate::block_tlb::TVMStack;
use crate::contracts::ton_contract::TonContract;
use crate::contracts::*;
use crate::errors::TonError;
use crate::tep::metadata::MetadataContent;
use crate::tep::tvm_results::{GetNFTDataResult, TVMResult};
use crate::ton_contract;
use crate::ton_core::traits::contract_provider::TonContractState;
use async_trait::async_trait;
use ton_core::errors::TonCoreError;

ton_contract!(NFTItemContract: NFTItemMethods);

#[async_trait]
pub trait NFTItemMethods: TonContract {
    async fn get_nft_data(&self) -> Result<GetNFTDataResult, TonError> {
        let stack_boc = self.emulate_get_method("get_nft_data", &TVMStack::EMPTY).await?;
        Ok(GetNFTDataResult::from_boc(stack_boc)?)
    }

    async fn load_full_nft_data(&self) -> Result<GetNFTDataResult, TonCoreError> {
        let mut data = self.get_nft_data().await?;
        if let MetadataContent::Unsupported(meta) = data.individual_content {
            let collection_address = &data.collection_address;
            let collection = NFTCollectionContract::new(self.get_client(), collection_address, None).await?;
            let full_content = collection.get_nft_content(data.index.clone(), meta.cell).await?;
            data.individual_content = full_content.full_content;
            Ok(data)
        } else {
            Ok(data)
        }
    }
}
