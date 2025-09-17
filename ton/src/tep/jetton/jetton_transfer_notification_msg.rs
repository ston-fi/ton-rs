use crate::block_tlb::Coins;
use ton_lib_core::cell::TonCell;
use ton_lib_core::types::tlb_core::TLBEitherRef;
use ton_lib_core::types::TonAddress;
use ton_lib_core::TLB;

/// ```raw
/// transfer_notification#7362d09c query_id:uint64 amount:(VarUInteger 16)
/// sender:MsgAddress forward_payload:(Either Cell ^Cell)
/// = InternalMsgBody;
/// ```
#[derive(Clone, Debug, PartialEq, TLB)]
#[tlb(prefix = 0x7362d09c, bits_len = 32, ensure_empty = true)]
pub struct JettonTransferNotificationMsg {
    pub query_id: u64,                          // should be equal with request's query_id
    pub amount: Coins,                          // amount of transferred jettons
    pub sender: TonAddress,                     // is address of the previous owner of transferred jettons
    pub forward_payload: TLBEitherRef<TonCell>, //  optional custom data that should be sent to the destination address.
}

impl JettonTransferNotificationMsg {
    pub fn new<C: Into<Coins>>(amount: C, sender: TonAddress, forward_payload: TonCell) -> Self {
        JettonTransferNotificationMsg {
            query_id: 0,
            amount: amount.into(),
            sender,
            forward_payload: TLBEitherRef::new(forward_payload),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use ton_lib_core::cell::TonCell;
    use ton_lib_core::traits::tlb::TLB;
    use ton_lib_core::types::TonAddress;

    #[test]
    fn test_jetton_transfer_notification_parser() -> anyhow::Result<()> {
        let notif_msg = JettonTransferNotificationMsg::from_boc_hex("b5ee9c720101020100a60001647362d09c000000d2c7ceef23401312d008003be20895401cd8539741eb7815d5e63b3429014018d7e5f7800de16a984f27730100dd25938561800f2465b65c76b1b562f32423676970b431319419d5f45ffd2eeb2155ce6ab7eacc78ee0250ef0300077c4112a8039b0a72e83d6f02babcc766852028031afcbef001bc2d5309e4ee700257a672371a90e149b7d25864dbfd44827cc1e8a30df1b1e0c4338502ade2ad96")?;

        let expected_addr = TonAddress::from_str("EQAd8QRKoA5sKcug9bwK6vMdmhSAoAxr8vvABvC1TCeTude5")?;
        let expected_payload = TonCell::from_boc_hex("b5ee9c720101010100710000dd25938561800f2465b65c76b1b562f32423676970b431319419d5f45ffd2eeb2155ce6ab7eacc78ee0250ef0300077c4112a8039b0a72e83d6f02babcc766852028031afcbef001bc2d5309e4ee700257a672371a90e149b7d25864dbfd44827cc1e8a30df1b1e0c4338502ade2ad96")?;
        let mut expected_msg = JettonTransferNotificationMsg::new(20000000u64, expected_addr, expected_payload);
        expected_msg.query_id = 905295359779;
        assert_eq!(notif_msg, expected_msg);
        let serialized = notif_msg.to_boc_hex()?;
        let parsed_back = JettonTransferNotificationMsg::from_boc_hex(&serialized)?;
        assert_eq!(parsed_back, notif_msg);
        Ok(())
    }
}
