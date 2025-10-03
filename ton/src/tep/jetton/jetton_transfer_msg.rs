use crate::block_tlb::Coins;
use ton_lib_core::cell::{TonCell, TonCellRef};
use ton_lib_core::types::tlb_core::{MsgAddress, MsgAddressInt, TLBEitherRef};
use ton_lib_core::TLB;

///```raw
/// transfer#f8a7ea5 query_id:uint64 amount:(VarUInteger 16) destination:MsgAddress
/// response_destination:MsgAddress custom_payload:(Maybe ^Cell)
/// forward_ton_amount:(VarUInteger 16) forward_payload:(Either Cell ^Cell)
/// = InternalMsgBody;
/// ```

#[derive(Clone, Debug, PartialEq, TLB)]
#[tlb(prefix = 0xf8a7ea5, bits_len = 32)]
pub struct JettonTransferMsg {
    pub query_id: u64, // arbitrary number to identify the transfer
    pub amount: Coins, // amount of transferred jettons in elementary units
    pub dst: MsgAddress,
    pub response_dst: MsgAddress, // address where to send a response with confirmation of a successful transfer and the rest of the incoming message Toncoins.
    pub custom_payload: Option<TonCellRef>, // optional custom data (which is used by either sender or receiver jetton ton_wallet for inner logic).
    pub forward_ton_amount: Coins,          // the amount of nano-tons to be sent to the destination address.
    pub forward_payload: TLBEitherRef<TonCell>, // optional custom data that should be sent to the destination address.
}

impl JettonTransferMsg {
    pub fn new<T: Into<Coins>>(dst: MsgAddressInt, amount: T) -> Self {
        JettonTransferMsg {
            query_id: 0,
            amount: amount.into(),
            dst: dst.into(),
            response_dst: MsgAddress::NONE,
            custom_payload: None,
            forward_ton_amount: Coins::ZERO,
            forward_payload: TLBEitherRef::new(TonCell::EMPTY),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::str::FromStr;
    use ton_lib_core::traits::tlb::TLB;
    use ton_lib_core::types::tlb_core::EitherRefLayout;
    use ton_lib_core::types::TonAddress;

    #[test]
    fn test_jetton_transfer_msg() -> anyhow::Result<()> {
        // int_msg from https://tonviewer.com/transaction/18679bed03915803746469e9fe498add0ffecd76ae3056bb9c3777c9f722becd
        let msg_boc = "b5ee9c720101020100650001b40f8a7ea55ecf57d735066d2460246139ca800800f52547902494daa24c332ecb41067ee9b6bae7b244a68ce0c5007ddc22f4b01f001f5d9cc275e5514e8386836ef59caa82e043c006d404f512ab7ee893e38f5f8d8847868c0101000be8e8e46c0020";
        let msg = JettonTransferMsg::from_boc_hex(msg_boc)?;

        let mut pl_builder = TonCell::builder();
        pl_builder.write_bits([232, 232, 228, 108, 0, 0], 42)?;
        let payload = pl_builder.build()?;

        let exp_msg = JettonTransferMsg {
            query_id: 6831775741563530532,
            amount: Coins::from_str("2500000000000")?,
            dst: TonAddress::from_str("0:7a92a3c8124a6d5126199765a0833f74db5d73d92253467062803eee117a580f")?
                .to_msg_address_int()
                .into(),
            response_dst: TonAddress::from_str("0:7d767309d795453a0e1a0dbbd672aa0b810f001b5013d44aadfba24f8e3d7e36")?
                .to_msg_address_int()
                .into(),
            custom_payload: None,
            forward_ton_amount: Coins::new(600000000u128),
            forward_payload: TLBEitherRef {
                value: payload,
                layout: EitherRefLayout::ToRef,
            },
        };
        assert_eq!(exp_msg, msg);
        Ok(())
    }
}
