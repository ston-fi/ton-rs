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
mod tests {
    use super::*;
    use crate::block_tlb::FromTVMStack;
    use std::str::FromStr;
    use ton_core::traits::tlb::TLB;

    #[test]
    fn test_get_collection_data_result() -> anyhow::Result<()> {
        // Plush pepes EQBG-g6ahkAUGWpefWbx-D_9sQ8oWbvy6puuq78U2c4NUDFS
        let result = GetCollectionDataResult::from_stack_boc_hex(
            "b5ee9c7201010601007b00020f000003044651b020010202020303040049bc82df6a2686900698fe9ffea6a6a00e8698380d5016b8c009880ea68881b2f833fc581094011201ffffffffffffffff0500660168747470733a2f2f6e66742e667261676d656e742e636f6d2f636f6c6c656374696f6e2f706c757368706570652e6a736f6e0000",
        )?;
        assert_eq!(result.next_item_index, -1);
        assert_eq!(
            result.collection_content,
            MetadataContent::from_boc_hex(
                "b5ee9c720101010100350000660168747470733a2f2f6e66742e667261676d656e742e636f6d2f636f6c6c656374696f6e2f706c757368706570652e6a736f6e"
            )?
        );
        assert_eq!(result.owner_address, TonAddress::from_boc_hex("b5ee9c7201010101000300000120")?);
        Ok(())
    }

    #[test]
    fn test_get_nft_address_by_index_result() -> anyhow::Result<()> {
        // Plush pepes 298 EQBUXuQI612W1e71Gk5atugejGqteQeDa8hA9tTwREcXWQiv, Collection EQBG-g6ahkAUGWpefWbx-D_9sQ8oWbvy6puuq78U2c4NUDFS
        let result = GetNFTAddressByIndexResult::from_stack_boc_hex(
            "b5ee9c7201010301003200020f000001040010b020010200000043800a8bdc811d6bb2dabddea349cb56dd03d18d55af20f06d79081eda9e0888e2eb30",
        )?;
        assert_eq!(result.nft_address, TonAddress::from_str("EQBUXuQI612W1e71Gk5atugejGqteQeDa8hA9tTwREcXWQiv")?);
        Ok(())
    }

    #[test]
    fn test_get_nft_full_content() -> anyhow::Result<()> {
        // EQAbNqfCuv4Chy6D-2UBKzi3qYvVPrB-STOzBGQo5AKh4P9u
        let result = GetNFTContentResult::from_stack_boc_hex(
            "b5ee9c72010105010055000208000001030102000001800168747470733a2f2f746f6e73746174696f6e2e6170702f6e66742d6170692f6170692f76312f6e6674732f544f4e25323073746174696f6e2532307362742f030100040006343131",
        )?;
        let Some(content) = result.full_content.as_external() else {
            anyhow::bail!("expected external NFT content");
        };
        assert_eq!(content.uri.as_str(), "https://tonstation.app/nft-api/api/v1/nfts/TON%20station%20sbt/411");
        Ok(())
    }
}
