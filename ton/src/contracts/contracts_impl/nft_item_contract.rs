use crate::contracts::ContractClient;
use crate::contracts::contract_methods::{NFTCollectionMethods, NFTItemMethods};
use crate::contracts::{NFTCollectionContract, TonContract};
use crate::tep::metadata::MetadataContent;
use crate::tep::tvm_results::GetNFTDataResult;
use crate::ton_contract;
use crate::ton_core::traits::contract_provider::TonContractState;
use crate::ton_core::traits::tlb::TLB;
use ton_core::cell::TonCell;
use ton_core::errors::TonCoreError;

ton_contract!(NFTItemContract: NFTItemMethods);
impl NFTItemContract<TonCell> {
    pub async fn load_full_nft_data(&self) -> Result<GetNFTDataResult, TonCoreError> {
        let mut data = self.get_nft_data().await?;
        if let MetadataContent::Unsupported(meta) = data.individual_content {
            let collection_address = &data.collection_address;
            let collection = NFTCollectionContract::new(self.get_client(), collection_address, None).await?;
            let full_content = collection.get_nft_content(data.index, meta.cell).await?;
            data.individual_content = full_content.full_content;
            Ok(data)
        } else {
            Ok(data)
        }
    }
}
