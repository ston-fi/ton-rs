use std::ops::Deref;

use crate::block_tlb::TVMStack;
use crate::tep::metadata::MetadataContent;
use crate::tep::tvm_results::tvm_result::TVMResult;
use ton_lib_core::errors::TonCoreError;
use ton_lib_core::traits::tlb::TLB;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetNFTContentResult {
    pub full_content: MetadataContent,
}

impl TVMResult for GetNFTContentResult {
    fn from_stack(stack: &mut TVMStack) -> Result<Self, TonCoreError> {
        let full_content = MetadataContent::from_cell(stack.pop_cell()?.deref())?;
        Ok(Self { full_content })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_nft_full_content() -> anyhow::Result<()> {
        // EQAbNqfCuv4Chy6D-2UBKzi3qYvVPrB-STOzBGQo5AKh4P9u
        let result = GetNFTContentResult::from_boc_hex("b5ee9c72010105010055000208000001030102000001800168747470733a2f2f746f6e73746174696f6e2e6170702f6e66742d6170692f6170692f76312f6e6674732f544f4e25323073746174696f6e2532307362742f030100040006343131")?;
        assert_eq!(
            &result.full_content.as_external().unwrap().uri.as_str(),
            "https://tonstation.app/nft-api/api/v1/nfts/TON%20station%20sbt/411"
        );
        Ok(())
    }
}
