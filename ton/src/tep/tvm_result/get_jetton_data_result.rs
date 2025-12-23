use crate::tep::metadata::MetadataContent;
use ton_core::cell::TonCell;
use ton_core::types::{Coins, TonAddress};
use ton_macros::FromTVMStack;

#[derive(Debug, Clone, PartialEq, FromTVMStack)]
#[from_tvm_stack(ensure_empty = true)]
pub struct GetJettonDataResult {
    pub total_supply: Coins,
    pub mintable: bool,
    pub admin: TonAddress,
    pub content: MetadataContent,
    pub wallet_code: TonCell,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::block_tlb::FromTVMStack;
    use std::str::FromStr;

    #[test]
    fn test_get_jetton_data_result() -> anyhow::Result<()> {
        let result = GetJettonDataResult::from_stack_boc_hex(
            "b5ee9c720102100100010100020800000503010e02020302030209040f1470200405010300c006011201ffffffffffffffff070253705148e3baabcb0800c881fc78d28207072c728a2e7896228f37e17369ae121cb0eef7b4b0385f3330400e08020120090a0112010005148e3baabcb00b01000f0143bff872ebdb514d9c97c283b7f0ae5179029e2b6119c39462719e4f46ed8f7413e6400c0143bff7407e978f01a40711411b1acb773a96bdd93fa83bb5ca8435013c8c4b3ac91f400d00000102000f000400360842028f452d7a4dfd74066b682365177259ed05734435be76b5fd4bd5d8af2b7c3d68003e68747470733a2f2f7465746865722e746f2f757364742d746f6e2e6a736f6e",
        )?;
        assert_eq!(result.total_supply, Coins::new(1429976002510000));
        assert!(result.mintable);
        assert_eq!(
            result.admin,
            TonAddress::from_str("0:6440fe3c69410383963945173c4b11479bf0b9b4d7090e58777bda581c2f9998")?
        );
        Ok(())
    }
}
