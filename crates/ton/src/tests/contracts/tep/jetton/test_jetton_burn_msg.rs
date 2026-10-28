use super::*;
use std::str::FromStr;

use ton_core::traits::tlb::TLB;
use ton_core::types::TonAddress;

#[test]
fn test_jetton_burn_msg() -> anyhow::Result<()> {
    let burn_msg: JettonBurnMsg = TLB::from_boc_hex(
        "b5ee9c72010101010033000062595f07bc0000009b5946deef3080f21800b026e71919f2c839f639f078d9ee6bc9d7592ebde557edf03661141c7c5f2ea2",
    )?;

    let expected_msg = JettonBurnMsg {
        query_id: 667217747695,
        amount: TLBCoins::new(528_161),
        response_dst: TonAddress::from_str("EQBYE3OMjPlkHPsc-Dxs9zXk66yXXvKr9vgbMIoOPi-XUa-f")?
            .to_msg_address_int()
            .into(),
        custom_payload: None,
    };

    assert_eq!(burn_msg, expected_msg);
    let serialized = burn_msg.to_boc()?;
    let parsed_back = JettonBurnMsg::from_boc(serialized)?;
    assert_eq!(expected_msg, parsed_back);

    let burn_notcoin_msg: JettonBurnMsg = TLB::from_boc_hex(
        "b5ee9c72010101010035000066595f07bc0000000000000001545d964b800800cd324c114b03f846373734c74b3c3287e1a8c2c732b5ea563a17c6276ef4af30",
    )?;

    let expected_burn_notcoin = JettonBurnMsg {
        query_id: 1,
        amount: TLBCoins::new(300_000_000_000),
        response_dst: TonAddress::from_str("EQBmmSYIpYH8IxubmmOlnhlD8NRhY5la9SsdC-MTt3pXmOSI")?
            .to_msg_address_int()
            .into(),
        custom_payload: None,
    };

    assert_eq!(burn_notcoin_msg, expected_burn_notcoin);
    Ok(())
}
