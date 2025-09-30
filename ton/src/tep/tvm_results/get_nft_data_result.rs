use crate::block_tlb::TVMStack;
use crate::tep::metadata::MetadataContent;
use crate::tep::tvm_results::tvm_result::TVMResult;
use num_bigint::BigInt;
use std::ops::Deref;
use ton_lib_core::errors::TonCoreError;
use ton_lib_core::traits::tlb::TLB;
use ton_lib_core::types::TonAddress;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetNFTDataResult {
    pub init: bool,
    pub index: BigInt,
    pub collection_address: TonAddress,
    pub owner_address: TonAddress,
    pub individual_content: MetadataContent,
}

impl TVMResult for GetNFTDataResult {
    fn from_stack(stack: &mut TVMStack) -> Result<Self, TonCoreError> {
        let individual_content = MetadataContent::from_cell(stack.pop_cell()?.deref())?;
        let owner_address: TonAddress = TonAddress::from_cell(stack.pop_cell()?.deref())?;
        let collection_address = TonAddress::from_cell(stack.pop_cell()?.deref())?;
        let index = stack.pop_int_or_tiny_int()?;
        let init = stack.pop_int_or_tiny_int()? != BigInt::ZERO;

        Ok(Self {
            init,
            index,
            collection_address,
            owner_address,
            individual_content,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_get_nft_data_result() -> anyhow::Result<()> {
        // NFT EQBUXuQI612W1e71Gk5atugejGqteQeDa8hA9tTwREcXWQiv Plush Pepe 298
        let result = GetNFTDataResult::from_boc_hex("b5ee9c7201020c0100012900020800000503010b0209040010b02002030209044020b02004050243800ff871ab7ff40fbb13c42d16e4ed204c78cfeed4d8aa8726a2316b60d9860afd6806070144020025a4c2e585379af593ec3ec86a6c380963c7edc0a648c69f730fa85542b3007308008325a4c2e585379af593ec3ec86a6c380963c7edc0a648c69f730fa85542b300738008df41d350c802832d4bcfacde3f07ffb621e50b377e5d5375d577e29b39c1aa100201400b09004b00050064800d1e740eda68a3431fa83c0b8e3698040a8ba8d64eae0c9ccb04bbda18937e0590011201ffffffffffffffff0a00200d706c757368706570652d3239380100000000620168747470733a2f2f6e66742e667261676d656e742e636f6d2f676966742f706c757368706570652d3239382e6a736f6e")?;
        assert!(result.init);
        assert_eq!(
            result.index,
            BigInt::from_str("17026683442852985036293000817890672620529067535828542797724775561309021470835")?
        );

        assert_eq!(
            result.collection_address,
            TonAddress::from_boc_hex(
                "b5ee9c720101010100240000438008df41d350c802832d4bcfacde3f07ffb621e50b377e5d5375d577e29b39c1aa10"
            )?
        );
        assert_eq!(result.individual_content, MetadataContent::from_boc_hex("b5ee9c720101010100330000620168747470733a2f2f6e66742e667261676d656e742e636f6d2f676966742f706c757368706570652d3239382e6a736f6e")?);

        Ok(())
    }
}
