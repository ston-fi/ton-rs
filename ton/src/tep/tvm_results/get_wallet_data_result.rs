use crate::block_tlb::TVMStack;
use crate::tep::tvm_results::TVMResult;
use num_bigint::BigInt;
use std::ops::Deref;
use ton_lib_core::cell::TonCellRef;
use ton_lib_core::errors::TonCoreError;
use ton_lib_core::traits::tlb::TLB;
use ton_lib_core::types::TonAddress;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetWalletDataResult {
    pub balance: BigInt,
    pub owner: TonAddress,
    pub master: TonAddress,
    pub wallet_code: TonCellRef,
}

impl TVMResult for GetWalletDataResult {
    fn from_stack(stack: &mut TVMStack) -> Result<Self, TonCoreError> {
        let wallet_code = stack.pop_cell()?;
        let master = TonAddress::from_cell(stack.pop_cell()?.deref())?;
        let owner = TonAddress::from_cell(stack.pop_cell()?.deref())?;
        let balance = stack.pop_int_or_tiny_int()?;

        Ok(Self {
            balance,
            owner,
            master,
            wallet_code,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_get_wallet_data_result_from_stack_balance_tiny_int() -> anyhow::Result<()> {
        let result = GetWalletDataResult::from_boc_hex("b5ee9c7201010701009800020800000403010202090452e5602003040842028f452d7a4dfd74066b682365177259ed05734435be76b5fd4bd5d8af2b7c3d680209041014b020050400950701e31ccb92173980047f52f3231e3fce05b722017f087d0f8e02bd99a7348e43d36d7582e178099c7002c44ea652d4092859c67da44e4ca3add6565b0e2897d640a2c51bfb370d8877fa0112010001e31ccb921739060000")?;
        assert_eq!(result.owner, TonAddress::from_str("UQAj-peZGPH-cC25EAv4Q-h8cBXszTmkch6ba6wXC8BM40qt")?);
        assert_eq!(result.master, TonAddress::from_str("EQCxE6mUtQJKFnGfaROTKOt1lZbDiiX1kCixRv7Nw2Id_sDs")?);
        Ok(())
    }

    #[test]
    fn test_get_wallet_data_result_from_stack_bigint_balance_int() -> anyhow::Result<()> {
        // https://tonviewer.com/EQBybcAXkDY5fgIE0Qcgb6mGd2fez3B0EhkBQ2aLv69jWfnU
        let result = GetWalletDataResult::from_boc_hex("b5ee9c72010217010003b500020800000403010602090457e6a02002030209041515f0200403019ba175b123bafde570530af801069fae849f3b52ebf6166ada3dd547f29eee41576670a41ded9c833101faf76d0025838f8f471489d2c12938f5e028e3be84d58072c0ba05b9267a38edf1ee398fe0060144020000000000000000000000000000000000000000000000175b123bafde570530af0500000114ff00f4a413f4bcf2c80b0702016208090202cc0a0b001ba0f605da89a1f401f481f481a8610201d410110201200c0d0201200e0f0083d40106b90f6a2687d007d207d206a1802698fc1080bc6a28ca9105d41083deecbef09dd0958f97162e99f98fd001809d02811e428027d012c678b00e78b6664f6aa401f1503d33ffa00fa4021f001ed44d0fa00fa40fa40d4305136a1522ac705f2e2c128c2fff2e2c254344270542013541403c85004fa0258cf1601cf16ccc922c8cb0112f400f400cb00c920f9007074c8cb02ca07cbffc9d004fa40f40431fa0020d749c200f2e2c4778018c8cb055008cf1670fa0217cb6b13cc812020120131400c30831c02497c138007434c0c05c6c2544d7c0fc03383e903e900c7e800c5c75c87e800c7e800c1cea6d0000b4c7e08403e29fa954882ea54c4d167c0278208405e3514654882ea58c511100fc02b80d60841657c1ef2ea4d67c02f817c12103fcbc2000113e910c1c2ebcb85360009e8210178d4519c8cb1f19cb3f5007fa0222cf165006cf1625fa025003cf16c95005cc2391729171e25008a813a08209c9c380a014bcf2e2c504c98040fb001023c85004fa0258cf1601cf16ccc9ed5402f73b51343e803e903e90350c0234cffe80145468017e903e9014d6f1c1551cdb5c150804d50500f214013e809633c58073c5b33248b232c044bd003d0032c0327e401c1d3232c0b281f2fff274140371c1472c7cb8b0c2be80146a2860822625a019ad822860822625a028062849e5c412440e0dd7c138c34975c2c060151600d73b51343e803e903e90350c01f4cffe803e900c145468549271c17cb8b049f0bffcb8b08160824c4b402805af3cb8b0e0841ef765f7b232c7c572cfd400fe8088b3c58073c5b25c60063232c14933c59c3e80b2dab33260103ec01004f214013e809633c58073c5b3327b552000705279a018a182107362d09cc8cb1f5230cb3f58fa025007cf165007cf16c9718010c8cb0524cf165006fa0215cb6a14ccc971fb0010241023007cc30023c200b08e218210d53276db708010c8cb055008cf165004fa0216cb6a12cb1f12cb3fc972fb0093356c21e203c85004fa0258cf1601cf16ccc9ed54")?;
        assert_eq!(result.owner, TonAddress::from_str("EQCDT9dCT52pdfsLNW0e6qP5T3cgq7M4Ug72zkGYgP17tsWD")?);
        assert_eq!(result.master, TonAddress::from_str("EQCWDj49HFInSwSk49eAo476E1YBywLoFuSZ6OO3x7jmP2jn")?);
        Ok(())
    }
}
