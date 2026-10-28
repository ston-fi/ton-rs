use super::*;
use tokio_test::{assert_err, assert_ok};

#[test]
fn test_ton_address_to_string() -> anyhow::Result<()> {
    let bytes = TonHash::from_str("e4d954ef9f4e1250a26b5bbad76a1cdd17cfd08babad6f4c23e372270aef6f76")?;
    let addr = TonAddress::new(0, bytes);
    assert_eq!(addr.to_hex(), "0:e4d954ef9f4e1250a26b5bbad76a1cdd17cfd08babad6f4c23e372270aef6f76");
    assert_eq!(addr.to_base64(true, true, true), "EQDk2VTvn04SUKJrW7rXahzdF8_Qi6utb0wj43InCu9vdjrR");
    assert_eq!(addr.to_base64(true, true, false), "EQDk2VTvn04SUKJrW7rXahzdF8/Qi6utb0wj43InCu9vdjrR");
    assert_eq!(addr.to_base64(true, true, true), addr.to_string());

    let elector = TonAddress::from_str("-1:3333333333333333333333333333333333333333333333333333333333333333")?;
    assert_eq!(elector.to_base64(false, true, true), "kf8zMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzM_BP");
    Ok(())
}

#[test]
fn test_ton_address_from_str() -> anyhow::Result<()> {
    let bytes = TonHash::from_str("e4d954ef9f4e1250a26b5bbad76a1cdd17cfd08babad6f4c23e372270aef6f76")?;
    let addr = TonAddress::new(0, bytes);
    assert_eq!(TonAddress::from_str("0:e4d954ef9f4e1250a26b5bbad76a1cdd17cfd08babad6f4c23e372270aef6f76")?, addr);
    assert_eq!(TonAddress::from_str("EQDk2VTvn04SUKJrW7rXahzdF8_Qi6utb0wj43InCu9vdjrR")?, addr);
    assert_eq!(TonAddress::from_str("EQDk2VTvn04SUKJrW7rXahzdF8/Qi6utb0wj43InCu9vdjrR")?, addr);
    Ok(())
}

#[test]
fn test_ton_address_try_from_str() -> anyhow::Result<()> {
    let bytes = TonHash::from_str("e4d954ef9f4e1250a26b5bbad76a1cdd17cfd08babad6f4c23e372270aef6f76")?;
    let addr = TonAddress::new(0, bytes);
    assert_eq!(TonAddress::try_from("0:e4d954ef9f4e1250a26b5bbad76a1cdd17cfd08babad6f4c23e372270aef6f76")?, addr);
    assert_eq!(TonAddress::try_from("EQDk2VTvn04SUKJrW7rXahzdF8_Qi6utb0wj43InCu9vdjrR")?, addr);
    assert_eq!(TonAddress::try_from("EQDk2VTvn04SUKJrW7rXahzdF8/Qi6utb0wj43InCu9vdjrR".to_string())?, addr);
    Ok(())
}

#[test]
fn test_ton_address_crc_error() -> anyhow::Result<()> {
    assert_err!(TonAddress::from_str("EQDk2VTvn04SUKJrW7rXahzdF8_Qi6utb0wj43InCu9vdjra"));
    Ok(())
}

#[test]
fn test_ton_address_ord() -> anyhow::Result<()> {
    let address0 = TonAddress::from_str("EQBKwtMZSZurMxGp7FLZ_lM9t54_ECEsS46NLR3qfIwwTnKW")?;
    let address1 = TonAddress::from_str("EQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAM9c")?;
    assert_eq!(address0.partial_cmp(&address1), Some(Ordering::Less));
    Ok(())
}

#[test]
fn test_ton_address_to_msg_addr_int() -> anyhow::Result<()> {
    let address = TonAddress::from_str("EQDk2VTvn04SUKJrW7rXahzdF8_Qi6utb0wj43InCu9vdjrR")?;
    let msg_addr = address.to_msg_address_int();
    let expected = MsgAddressIntStd {
        anycast: None,
        workchain: 0,
        address: TonHash::from_str("e4d954ef9f4e1250a26b5bbad76a1cdd17cfd08babad6f4c23e372270aef6f76")?,
    }
    .into();
    assert_eq!(msg_addr, expected);
    Ok(())
}

#[test]
fn test_ton_address_to_msg_addr_none() -> anyhow::Result<()> {
    let address = TonAddress::from_str("EQDk2VTvn04SUKJrW7rXahzdF8_Qi6utb0wj43InCu9vdjrR")?;
    assert_err!(address.to_msg_address_none());
    let address = TonAddress::ZERO;
    assert_ok!(address.to_msg_address_none());
    Ok(())
}

#[test]
fn test_ton_address_tlb_type() -> anyhow::Result<()> {
    for addr in [
        "EQDk2VTvn04SUKJrW7rXahzdF8_Qi6utb0wj43InCu9vdjrR",
        "EQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAM9c",
    ] {
        let address = TonAddress::from_str(addr)?;
        let cell = address.to_cell()?;
        let parsed_address = TonAddress::from_cell(&cell)?;
        assert_eq!(address, parsed_address);
    }
    Ok(())
}

#[test]
fn test_ton_address_zero_to_string() -> anyhow::Result<()> {
    assert_eq!(TonAddress::ZERO.to_string(), "EQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAM9c");
    assert_eq!(TonAddress::NATIVE.to_string(), "EQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAM9c");
    assert_eq!(TonAddress::USDT.to_string(), "EQCxE6mUtQJKFnGfaROTKOt1lZbDiiX1kCixRv7Nw2Id_sDs");
    Ok(())
}
