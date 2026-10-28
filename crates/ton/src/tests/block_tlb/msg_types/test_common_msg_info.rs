use super::*;
use ton_core::cell::TonHash;
use ton_core::types::tlb_core::MsgAddressIntStd;

#[test]
fn test_block_tlb_common_msg_info_enum_derive() -> anyhow::Result<()> {
    let mut common_msg_info = CommonMsgInfo::Int(CommonMsgInfoInt {
        ihr_disabled: false,
        bounce: false,
        bounced: false,
        src: MsgAddress::Int(MsgAddressInt::Std(MsgAddressIntStd {
            anycast: None,
            workchain: 0,
            address: TonHash::ZERO,
        })),
        dst: MsgAddress::NONE,
        value: CurrencyCollection::from_num(&0u32)?,
        ihr_fee: TLBCoins::ZERO,
        fwd_fee: TLBCoins::ZERO,
        created_lt: 0,
        created_at: 0,
    });
    assert!(common_msg_info.as_int().is_some());
    assert!(common_msg_info.as_ext_in().is_none());
    assert!(common_msg_info.as_ext_in_mut().is_none());

    let int = common_msg_info.as_int().unwrap();
    let common_msg_info_from = CommonMsgInfo::from(int.clone());
    assert_eq!(common_msg_info_from, common_msg_info);
    Ok(())
}
