use super::*;
use tokio_test::assert_ok;

#[test]
fn test_block_tlb_msg_address_read_write() -> anyhow::Result<()> {
    // Anyhow read/write is covered under the hood
    let boc = "b5ee9c7201010101002800004bbe031053100134ea6c68e2f2cee9619bdd2732493f3a1361eccd7c5267a9eb3c5dcebc533bb6";
    let parsed = MsgAddress::from_boc_hex(boc)?;
    let expected = MsgAddressIntStd {
        anycast: Some(Anycast::new(30, vec![3, 16, 83, 16])),
        workchain: 0,
        address: [
            77, 58, 155, 26, 56, 188, 179, 186, 88, 102, 247, 73, 204, 146, 79, 206, 132, 216, 123, 51, 95, 20, 153,
            234, 122, 207, 23, 115, 175, 20, 206, 237,
        ]
        .into(),
    };
    assert_eq!(parsed, expected.into());

    let serial_cell = parsed.to_cell()?;
    let parsed_back = assert_ok!(MsgAddress::from_cell(&serial_cell));
    assert_eq!(parsed_back, parsed);
    Ok(())
}

#[test]
fn test_block_tlb_msg_address_int_read_i8_workchain() -> anyhow::Result<()> {
    let boc = "b5ee9c720101010100240000439fe00000000000000000000000000000000000000000000000000000000000000010";
    let parsed = assert_ok!(MsgAddress::from_boc_hex(boc));

    let expected = MsgAddressIntStd {
        anycast: None,
        workchain: -1,
        address: TonHash::from([0; 32]),
    };
    assert_eq!(parsed, expected.into());

    // don't support same layout, so check deserialized data again
    let serial_cell = parsed.to_cell()?;
    let parsed_back = assert_ok!(MsgAddress::from_cell(&serial_cell));
    assert_eq!(parsed, parsed_back);
    Ok(())
}

#[test]
fn test_block_tlb_msg_address_int_read() -> anyhow::Result<()> {
    let boc = "b5ee9c720101010100240000439fe00000000000000000000000000000000000000000000000000000000000000010";
    let parsed = assert_ok!(MsgAddressInt::from_boc_hex(boc));

    let expected = MsgAddressIntStd {
        anycast: None,
        workchain: -1,
        address: TonHash::from([0; 32]),
    };
    assert_eq!(parsed, expected.into());

    // don't support same layout, so check deserialized data again
    let serial_cell = parsed.to_cell()?;
    let parsed_back = assert_ok!(MsgAddressInt::from_cell(&serial_cell));
    assert_eq!(parsed, parsed_back);
    Ok(())
}

#[test]
fn test_block_tlb_msg_address_none() -> anyhow::Result<()> {
    let addr: MsgAddress = MsgAddressNone {}.into();
    let cell = addr.to_cell()?;
    let parsed = MsgAddress::from_cell(&cell)?;
    assert_eq!(parsed, addr);

    let addr_none = MsgAddressNone {};
    let addr_none_cell = addr_none.to_cell()?;
    let parsed_none = MsgAddress::from_cell(&addr_none_cell)?;
    let msg_addr_none: MsgAddress = addr_none.into();
    assert_eq!(parsed_none, msg_addr_none);
    Ok(())
}
