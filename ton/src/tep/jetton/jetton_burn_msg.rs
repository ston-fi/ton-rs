use crate::block_tlb::Coins;
use ton_lib_core::cell::TonCellRef;
use ton_lib_core::types::tlb_core::MsgAddress;
use ton_lib_core::TLB;

/// ```raw
/// burn#595f07bc query_id:uint64 amount:(VarUInteger 16)
/// response_destination:MsgAddress
/// custom_payload:(Maybe ^Cell)
/// = InternalMsgBody;
/// ```
#[derive(Clone, Debug, PartialEq, TLB)]
#[tlb(prefix = 0x595f07bc, bits_len = 32, ensure_empty = true)]
pub struct JettonBurnMsg {
    pub query_id: u64,            // arbitrary request number
    pub amount: Coins,            // amount to burn
    pub response_dst: MsgAddress, // address to send confirmation
    pub custom_payload: Option<TonCellRef>,
}

impl JettonBurnMsg {
    pub fn new<T: Into<Coins>>(amount: T) -> Self {
        JettonBurnMsg {
            query_id: 0,
            amount: amount.into(),
            response_dst: MsgAddress::NONE,
            custom_payload: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    use crate::block_tlb::Coins;
    use ton_lib_core::traits::tlb::TLB;
    use ton_lib_core::types::TonAddress;

    #[test]
    fn test_jetton_burn_msg() -> anyhow::Result<()> {
        let burn_msg = JettonBurnMsg::from_boc_hex("b5ee9c72010101010033000062595f07bc0000009b5946deef3080f21800b026e71919f2c839f639f078d9ee6bc9d7592ebde557edf03661141c7c5f2ea2")?;

        let expected_msg = JettonBurnMsg {
            query_id: 667217747695,
            amount: Coins::new(528161u64),
            response_dst: TonAddress::from_str("EQBYE3OMjPlkHPsc-Dxs9zXk66yXXvKr9vgbMIoOPi-XUa-f")?
                .to_msg_address_int()
                .into(),
            custom_payload: None,
        };

        assert_eq!(burn_msg, expected_msg);
        let serialized = burn_msg.to_boc()?;
        let parsed_back = JettonBurnMsg::from_boc(serialized.as_slice())?;
        assert_eq!(expected_msg, parsed_back);

        let burn_notcoin_msg = JettonBurnMsg::from_boc_hex("b5ee9c72010101010035000066595f07bc0000000000000001545d964b800800cd324c114b03f846373734c74b3c3287e1a8c2c732b5ea563a17c6276ef4af30")?;

        let expected_burn_notcoin = JettonBurnMsg {
            query_id: 1,
            amount: Coins::new(300000000000u64),
            response_dst: TonAddress::from_str("EQBmmSYIpYH8IxubmmOlnhlD8NRhY5la9SsdC-MTt3pXmOSI")?
                .to_msg_address_int()
                .into(),
            custom_payload: None,
        };

        assert_eq!(burn_notcoin_msg, expected_burn_notcoin);
        Ok(())
    }
}
