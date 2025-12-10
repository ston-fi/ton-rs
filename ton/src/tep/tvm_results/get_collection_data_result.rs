use crate::block_tlb::TVMStack;
use crate::errors::TonResult;
use crate::tep::metadata::MetadataContent;
use crate::tep::tvm_results::tvm_result::TVMResult;
use ton_core::TVMResult;
use ton_core::types::TonAddress;

#[derive(Debug, Clone, PartialEq, TVMResult)]
pub struct GetCollectionDataResult {
    pub next_item_index: i64,
    pub collection_content: MetadataContent,
    pub owner_address: TonAddress,
}

#[cfg(test)]
mod test {
    use super::*;
    use ton_core::traits::tlb::TLB;

    #[test]
    fn test_get_jetton_data_result() -> anyhow::Result<()> {
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
}
