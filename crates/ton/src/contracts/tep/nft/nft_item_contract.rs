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
mod tests {
    use super::*;
    use crate::block_tlb::FromTVMStack;
    use ton_core::traits::tlb::TLB;

    #[test]
    fn test_get_nft_data_result() -> anyhow::Result<()> {
        // NFT EQBUXuQI612W1e71Gk5atugejGqteQeDa8hA9tTwREcXWQiv Plush Pepe 298
        let result = GetNFTDataResult::from_stack_boc_hex(
            "b5ee9c7201020c0100012900020800000503010b0209040010b02002030209044020b02004050243800ff871ab7ff40fbb13c42d16e4ed204c78cfeed4d8aa8726a2316b60d9860afd6806070144020025a4c2e585379af593ec3ec86a6c380963c7edc0a648c69f730fa85542b3007308008325a4c2e585379af593ec3ec86a6c380963c7edc0a648c69f730fa85542b300738008df41d350c802832d4bcfacde3f07ffb621e50b377e5d5375d577e29b39c1aa100201400b09004b00050064800d1e740eda68a3431fa83c0b8e3698040a8ba8d64eae0c9ccb04bbda18937e0590011201ffffffffffffffff0a00200d706c757368706570652d3239380100000000620168747470733a2f2f6e66742e667261676d656e742e636f6d2f676966742f706c757368706570652d3239382e6a736f6e",
        )?;
        assert!(result.init);
        assert_eq!(
            result.index,
            I512::from_str("17026683442852985036293000817890672620529067535828542797724775561309021470835")?
        );

        assert_eq!(
            result.collection_address,
            TonAddress::from_boc_hex(
                "b5ee9c720101010100240000438008df41d350c802832d4bcfacde3f07ffb621e50b377e5d5375d577e29b39c1aa10"
            )?
        );
        assert_eq!(
            result.individual_content,
            MetadataContent::from_boc_hex(
                "b5ee9c720101010100330000620168747470733a2f2f6e66742e667261676d656e742e636f6d2f676966742f706c757368706570652d3239382e6a736f6e"
            )?
        );

        Ok(())
    }
}
