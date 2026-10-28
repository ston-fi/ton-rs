use super::*;
use crate::contracts::tep::jetton::jetton_provide_wallet_address_msg::JettonProvideWalletAddressMsg;
use ton_core::types::TonAddress;
#[test]
fn literal_addresses_and_owner_reference_round_trip() -> TonCoreResult<()> {
    let zero = MsgAddress::Int(TonAddress::ZERO.to_msg_address_int());
    assert_ne!(zero, MsgAddress::NONE);
    for address in [MsgAddress::NONE, zero] {
        for include in [false, true] {
            let request = JettonProvideWalletAddressMsg {
                query_id: u64::MAX,
                owner_address: address.clone(),
                include_address: include,
            };
            let cell = request.to_cell()?;
            assert_eq!(JettonProvideWalletAddressMsg::from_cell(&cell)?.to_cell()?, cell);
            let reply = JettonTakeWalletAddressMsg {
                query_id: 3,
                wallet_address: address.clone(),
                owner_address: include.then(|| TLBRef::new(address.clone())),
            };
            let cell = reply.to_cell()?;
            assert_eq!(JettonTakeWalletAddressMsg::from_cell_strict(&cell)?, reply);
        }
    }
    Ok(())
}
#[test]
fn rejects_malformed_root_and_owner_tails() -> TonCoreResult<()> {
    let reply = JettonTakeWalletAddressMsg {
        query_id: 3,
        wallet_address: MsgAddress::NONE,
        owner_address: None,
    };
    let cell = reply.to_cell()?;
    let mut root = TonCell::builder();
    root.write_cell(&cell)?;
    root.write_bit(false)?;
    assert!(JettonTakeWalletAddressMsg::from_cell_strict(&root.build()?).is_err());
    let mut owner = TonCell::builder();
    MsgAddress::NONE.write(&mut owner)?;
    owner.write_bit(false)?;
    let mut root = TonCell::builder();
    JettonTakeWalletAddressMsg::write_prefix(&mut root)?;
    3u64.write(&mut root)?;
    MsgAddress::NONE.write(&mut root)?;
    Some(TLBRef::new(owner.build()?)).write(&mut root)?;
    assert!(JettonTakeWalletAddressMsg::from_cell_strict(&root.build()?).is_err());
    assert!(JettonTakeWalletAddressMsg::from_cell_strict(TonCell::empty()).is_err());
    let request = JettonProvideWalletAddressMsg {
        query_id: 3,
        owner_address: MsgAddress::NONE,
        include_address: false,
    };
    assert!(JettonTakeWalletAddressMsg::from_cell_strict(&request.to_cell()?).is_err());
    Ok(())
}
