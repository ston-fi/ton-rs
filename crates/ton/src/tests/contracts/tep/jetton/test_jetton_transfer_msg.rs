use super::*;

use std::str::FromStr;
use ton_core::traits::tlb::TLB;
use ton_core::types::TonAddress;
use ton_core::types::tlb_core::EitherRefLayout;

#[test]
fn test_jetton_transfer_msg() -> anyhow::Result<()> {
    // int_msg from https://tonviewer.com/transaction/18679bed03915803746469e9fe498add0ffecd76ae3056bb9c3777c9f722becd
    let msg_boc = "b5ee9c720101020100650001b40f8a7ea55ecf57d735066d2460246139ca800800f52547902494daa24c332ecb41067ee9b6bae7b244a68ce0c5007ddc22f4b01f001f5d9cc275e5514e8386836ef59caa82e043c006d404f512ab7ee893e38f5f8d8847868c0101000be8e8e46c0020";
    let msg = JettonTransferMsg::<TonCell, TonCell>::from_boc_hex(msg_boc)?;

    let mut pl_builder = TonCell::builder();
    pl_builder.write_bits([232, 232, 228, 108, 0, 0], 42)?;
    let payload = pl_builder.build()?;

    let exp_msg = JettonTransferMsg {
        query_id: 6831775741563530532,
        amount: TLBCoins::from_str("2500000000000")?,
        dst: TonAddress::from_str("0:7a92a3c8124a6d5126199765a0833f74db5d73d92253467062803eee117a580f")?
            .to_msg_address_int()
            .into(),
        response_dst: TonAddress::from_str("0:7d767309d795453a0e1a0dbbd672aa0b810f001b5013d44aadfba24f8e3d7e36")?
            .to_msg_address_int()
            .into(),
        custom_payload: None,
        forward_ton_amount: TLBCoins::new(600000000u128),
        forward_payload: TLBEitherRef {
            value: payload,
            layout: EitherRefLayout::ToRef,
        },
    };
    assert_eq!(exp_msg, msg);
    Ok(())
}
