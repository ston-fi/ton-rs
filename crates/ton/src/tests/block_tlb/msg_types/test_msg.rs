use super::*;
use std::str::FromStr;
use tokio_test::assert_ok;
use ton_core::types::TonAddress;
use ton_core::types::tlb_core::{Anycast, MsgAddressExtern, MsgAddressIntStd, VarLenBits};

#[test]
fn test_common_msg_info_int() -> anyhow::Result<()> {
    let msg_cell = TonCell::from_boc_hex(
        "b5ee9c720101010100580000ab69fe00000000000000000000000000000000000000000000000000000000000000013fccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccd3050ec744000000617bc90dda80cf41ab8e40",
    )?;
    let parsed_msg = Msg::<TonCell>::from_cell(&msg_cell)?;
    assert!(parsed_msg.init.is_none());
    assert_eq!(parsed_msg.body.value.data_len_bits(), 0); // quite useless assert, but let it be here

    let info = match parsed_msg.info.clone() {
        CommonMsgInfo::Int(info) => info,
        _ => panic!("Expected CommonMsgInfo::Int"),
    };
    assert!(info.ihr_disabled);
    assert!(info.bounce);
    assert!(!info.bounced);

    let expected_src = TonAddress::from_str("Ef8AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAADAU")?;
    let expected_dest = TonAddress::from_str("Ef8zMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzM0vF")?;
    assert_eq!(TonAddress::from_msg_address(info.src)?, expected_src);
    assert_eq!(TonAddress::from_msg_address(info.dst)?, expected_dest);
    assert_eq!(info.value, CurrencyCollection::from_num(&3242439121u32)?);
    assert_eq!(info.ihr_fee, TLBCoins::ZERO);
    assert_eq!(info.fwd_fee, TLBCoins::ZERO);
    assert_eq!(info.created_lt, 53592141000000);
    assert_eq!(info.created_at, 1738593735u32);

    let serialized = parsed_msg.to_cell()?;
    let parsed_back = assert_ok!(Msg::from_cell(&serialized));
    assert_eq!(parsed_back, parsed_msg);
    Ok(())
}

#[test]
fn test_ext_in_msg_info() -> anyhow::Result<()> {
    let ext_in_msg_info = CommonMsgInfoExtIn::from_boc_hex(
        "b5ee9c7201010101002500004588010319f77e4d761f956e78f9c9fd45f1e914b7ffab9b5c1ea514858979c1560dee10",
    )?;
    let expected_dst = TonAddress::from_str("EQCBjPu_JrsPyrc8fOT-ovj0ilv_1c2uD1KKQsS84KsG90PM")?;
    let dst = TonAddress::from_msg_address(ext_in_msg_info.dst.clone())?;
    assert_eq!(dst, expected_dst);
    assert_eq!(ext_in_msg_info.import_fee, TLBCoins::ZERO);

    let cell = ext_in_msg_info.to_cell()?;
    let parsed = CommonMsgInfoExtIn::from_cell(&cell)?;
    assert_eq!(parsed, ext_in_msg_info);
    Ok(())
}

// reproducing https://github.com/tonkeeper/tongo/blob/5c0ce694d72b7024bcb62b3d0dcd008940a75419/tlb/messages_test.go#L23C1-L80C2
#[test]
fn test_ext_in_msg_hash_normalized() -> anyhow::Result<()> {
    let msg_info = CommonMsgInfo::ExtIn(CommonMsgInfoExtIn {
        src: MsgAddressExt::Extern(MsgAddressExtern {
            address: VarLenBits::new(vec![1, 2, 3], 16),
        }),
        dst: MsgAddressInt::Std(MsgAddressIntStd {
            anycast: Some(Anycast::new(16, vec![9, 12])),
            workchain: -1,
            address: TonHash::from_str("adfd5f1d28db13e50591d5c76a976c15d8ab6cad90554748ab254871390d9334")?,
        }),
        import_fee: TLBCoins::new(12364u128),
    });
    let mut body_value_builder = TonCell::builder();
    body_value_builder.write_num(&200u32, 32)?;
    let body_value = body_value_builder.build()?;

    let msg = Msg {
        info: msg_info,
        init: Some(TLBEitherRef {
            value: StateInit::new(TonCell::empty().to_owned(), TonCell::empty().to_owned()),
            layout: EitherRefLayout::ToRef,
        }),
        body: TLBEitherRef {
            value: body_value,
            layout: EitherRefLayout::ToRef,
        },
    };
    let hash_norm = msg.cell_hash_normalized()?;
    assert_eq!(hash_norm, TonHash::from_str("dfacc0b48826e33a5a127ee1def710a449d8ce79def7c19f43e57b7996e870df")?);

    Ok(())
}
