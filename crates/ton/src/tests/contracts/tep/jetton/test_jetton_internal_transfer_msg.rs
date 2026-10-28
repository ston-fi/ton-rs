use super::*;
use std::str::FromStr;
use ton_core::types::TonAddress;

#[test]
fn test_jetton_internal_transfer_msg() -> anyhow::Result<()> {
    let msg = JettonInternalTransferMsg::from_boc_hex(
        "b5ee9c720101020100aa0001af178d45190000005209ddeb9e440ee9390801e6ef228644c75beba08c8b8e2adf62f1e760e84861b5c33027f0433e19085713003cdde450c898eb7d74119171c55bec5e3cec1d090c36b86604fe0867c3210ae2501dcd65030100992593856180022a16a3164c4d5aa3133f3110ff10496e00ca8ac8abeffc5027e024d33480c3ea916f9f4a23003cdde450c898eb7d74119171c55bec5e3cec1d090c36b86604fe0867c3210ae250",
    )?;
    let payload = TonCell::from_boc_hex(
        "b5ee9c7201010101004f0000992593856180022a16a3164c4d5aa3133f3110ff10496e00ca8ac8abeffc5027e024d33480c3ea916f9f4a23003cdde450c898eb7d74119171c55bec5e3cec1d090c36b86604fe0867c3210ae250",
    )?;

    let expected = JettonInternalTransferMsg {
        query_id: 352352856990,
        amount: TLBCoins::new(1_089_377_168),
        from_address: TonAddress::from_str("UQDzd5FDImOt9dBGRccVb7F487B0JDDa4ZgT-CGfDIQriSB-")?.to_msg_address(),
        response_address: TonAddress::from_str("UQDzd5FDImOt9dBGRccVb7F487B0JDDa4ZgT-CGfDIQriSB-")?.to_msg_address(),
        forward_amount: TLBCoins::new(125_000_000),
        forward_payload: TLBEitherRef::new(payload),
    };

    assert_eq!(expected, msg);

    let serialized = expected.to_boc()?;
    let parsed_back = JettonInternalTransferMsg::from_boc(serialized)?;
    assert_eq!(expected, parsed_back);
    Ok(())
}
