use super::*;
use sha2::{Digest, Sha256};
use ton::ton_core::{
    cell::TonHash,
    types::{TonAddress, tlb_core::MsgAddress},
};
fn hint(cell: &TonCell, id: u32) -> anyhow::Result<Vec<u8>> {
    let v = recognize::hints(cell, SigningPolicy::ClearOnly).map_err(|e| anyhow::anyhow!("hint {id}: {e}"))?;
    assert_eq!(v[0], 1);
    assert_eq!(u32::from_be_bytes(v[1..5].try_into()?), id);
    Ok(v[7..].to_vec())
}
fn start(op: u32) -> anyhow::Result<ton::ton_core::cell::CellBuilder> {
    let mut b = TonCell::builder();
    b.write_num(&op, 32)?;
    if op != 0 {
        b.write_num(&0u64, 64)?;
    }
    Ok(b)
}
#[test]
fn test_all_hint_families_and_dns_empty_capabilities() -> anyhow::Result<()> {
    let addr = TonAddress::ZERO;
    let mut b = start(0)?;
    b.write_bits(b"hello", 40)?;
    assert_eq!(hint(&b.build()?, 0)?, b"hello");
    for (op, id) in [(0x0f8a7ea5, 1), (0x5fcc3d14, 2), (0x595f07bc, 3)] {
        let mut b = start(op)?;
        if id != 2 {
            TLBCoins::ONE.write(&mut b)?;
        }
        if id != 3 {
            addr.to_msg_address_int().write(&mut b)?;
        }
        addr.to_msg_address_int().write(&mut b)?;
        b.write_bit(false)?;
        if id != 3 {
            TLBCoins::ZERO.write(&mut b)?;
            b.write_bit(false)?;
        }
        hint(&b.build()?, id)?;
    }
    for (op, id) in [(0x7258a69b, 4), (0x1001, 6)] {
        let mut b = start(op)?;
        addr.to_msg_address_int().write(&mut b)?;
        hint(&b.build()?, id)?;
    }
    let mut b = start(0x1000)?;
    TLBCoins::ONE.write(&mut b)?;
    hint(&b.build()?, 5)?;
    let mut b = start(0x47d54391)?;
    b.write_num(&9u64, 64)?;
    assert_eq!(hint(&b.build()?, 7)?, [0, 1, 0, 0, 0, 0, 0, 0, 0, 9]);
    let mut b = start(0x69fb306c)?;
    addr.to_msg_address_int().write(&mut b)?;
    b.write_num(&1_700_000_000u64, 48)?;
    b.write_bit(true)?;
    b.write_bit(false)?;
    let data = hint(&b.build()?, 8)?;
    assert_eq!(&data[34..], &hex::decode("00006553f1000100")?);
    let mut b = start(0x4eb1f0f9)?;
    b.write_bits(Sha256::digest(b"wallet"), 256)?;
    let mut r = TonCell::builder();
    r.write_num(&0x9fd3u16, 16)?;
    addr.to_msg_address_int().write(&mut r)?;
    r.write_num(&1u8, 8)?;
    r.write_bit(false)?;
    b.write_ref(r.build()?)?;
    let data = hint(&b.build()?, 9)?;
    assert_eq!(&data[data.len() - 2..], &[1, 0]);
    let mut b = start(8)?;
    TonHash::ZERO.write(&mut b)?;
    hint(&b.build()?, 10)?;
    for (op, id) in [(0x7bcd1fefu32, 11), (0xda803efd, 12)] {
        let mut b = TonCell::builder();
        b.write_num(&op, 32)?;
        b.write_num(&1u64, 64)?;
        TLBCoins::ONE.write(&mut b)?;
        if id == 12 {
            TLBCoins::ONE.write(&mut b)?;
        }
        hint(&b.build()?, id)?;
    }
    let mut comment = start(0)?;
    comment.write_bits(b"hi", 16)?;
    let mut msg = Msg::new(CommonMsgInfoInt::new(addr.to_msg_address_int().into(), TLBCoins::ONE), comment.build()?);
    msg.body.layout = EitherRefLayout::Native;
    let mut b = start(0xa7733acd)?;
    b.write_num(&3u8, 8)?;
    b.write_ref(msg.to_cell()?)?;
    hint(&b.build()?, 13)?;
    Ok(())
}
#[test]
fn test_layout_policy_and_trailing_data() -> anyhow::Result<()> {
    let mut b = start(0x1000)?;
    TLBCoins::ONE.write(&mut b)?;
    b.write_bit(false)?;
    let invalid = b.build()?;
    assert!(recognize::hints(&invalid, SigningPolicy::ClearOnly).is_err());
    assert_eq!(recognize::hints(&invalid, SigningPolicy::AllowOpaque)?, [0]);
    let mut b = start(0)?;
    b.write_bits([0x7f], 8)?;
    assert!(recognize::hints(&b.build()?, SigningPolicy::ClearOnly).is_err());
    let mut msg = Msg::new(
        CommonMsgInfoInt::new(TonAddress::ZERO.to_msg_address_int().into(), TLBCoins::ONE),
        TonCell::empty().clone(),
    );
    let version = WalletVersion::V4R2;
    let id = 698983191;
    // Present-empty reference is a distinct opaque cell, never silently omitted.
    let body = WalletVersion::build_ext_in_body(version, 2, 1, id, vec![msg.to_cell()?])?;
    assert!(transaction(version, id, &body, SigningPolicy::ClearOnly).is_err());
    assert!(transaction(version, id, &body, SigningPolicy::AllowOpaque).is_ok());
    msg.body.layout = EitherRefLayout::ToCell;
    let body = WalletVersion::build_ext_in_body(version, 2, 1, id, vec![msg.to_cell()?])?;
    transaction(version, id, &body, SigningPolicy::ClearOnly)?;
    if let CommonMsgInfo::Int(i) = &mut msg.info {
        i.src = MsgAddress::from(TonAddress::ZERO.to_msg_address_int());
    }
    let body = WalletVersion::build_ext_in_body(version, 2, 1, id, vec![msg.to_cell()?])?;
    assert!(transaction(version, id, &body, SigningPolicy::AllowOpaque).is_err());
    Ok(())
}

#[test]
fn test_upstream_firmware_python_vectors() -> anyhow::Result<()> {
    for line in include_str!("../../tests/fixtures/payloads.tsv").lines() {
        let fields: Vec<_> = line.split('\t').collect();
        let cell = TonCell::from_boc_hex(fields[1])?;
        assert_eq!(hex::encode(cell.cell_hash()?.as_slice()), fields[3]);
        assert_eq!(
            hex::encode(&recognize::hints(&cell, SigningPolicy::ClearOnly)?[1..]),
            fields[2],
            "hint {}",
            fields[0]
        );
    }
    for line in include_str!("../../tests/fixtures/transactions.tsv").lines() {
        let fields: Vec<_> = line.split('\t').collect();
        let cell = TonCell::from_boc_hex(fields[1])?;
        assert_eq!(hex::encode(cell.cell_hash()?.as_slice()), fields[3]);
        let version = if fields[0] == "1" { WalletVersion::V4R2 } else { WalletVersion::V3R2 };
        assert_eq!(
            hex::encode(transaction(version, 0xf1234567u32 as i32, &cell, SigningPolicy::ClearOnly)?),
            fields[2]
        );
    }
    Ok(())
}

#[test]
fn test_tep_custom_payload_encoding_and_policy() -> anyhow::Result<()> {
    let mut bytes32 = TonCell::builder();
    bytes32.write_bits([0xab; 32], 256)?;
    let mut bytes33 = TonCell::builder();
    bytes33.write_bits([0xab; 33], 264)?;
    let mut partial_byte = TonCell::builder();
    partial_byte.write_bit(true)?;
    let mut nested = TonCell::builder();
    nested.write_ref(TonCell::empty().clone())?;

    // Burn alone can display up to 32 whole bytes. Other custom payloads stay
    // opaque, including present-empty references in transfer and NFT messages.
    for custom in [
        TonCell::empty().clone(),
        bytes32.build()?,
        bytes33.build()?,
        partial_byte.build()?,
        nested.build()?,
    ] {
        for (op, id) in [(0x0f8a7ea5, 1), (0x5fcc3d14, 2), (0x595f07bc, 3)] {
            let mut b = start(op)?;
            if id != 2 {
                TLBCoins::ONE.write(&mut b)?;
            }
            if id != 3 {
                TonAddress::ZERO.to_msg_address_int().write(&mut b)?;
            }
            TonAddress::ZERO.to_msg_address_int().write(&mut b)?;
            b.write_bit(true)?;
            b.write_ref(custom.clone())?;
            if id != 3 {
                TLBCoins::ZERO.write(&mut b)?;
                b.write_bit(false)?;
            }
            let cell = b.build()?;
            let clear = recognize::hints(&cell, SigningPolicy::ClearOnly);
            let embedded = id == 3 && matches!(custom.data_len_bits(), 0 | 256) && custom.refs().is_empty();
            assert_eq!(clear.is_ok(), embedded, "hint {id}, custom {custom:?}");
            let encoded = recognize::hints(&cell, SigningPolicy::AllowOpaque)?;
            assert_eq!(encoded[0], 1, "recognized hints must survive opaque permission");
            assert_eq!(u32::from_be_bytes(encoded[1..5].try_into()?), id);
            if embedded {
                // Header (7), zero query (1), one coin (2), standard address (33).
                assert_eq!(&encoded[43..45], &[2, (custom.data_len_bits() / 8) as u8]);
                assert_eq!(&encoded[45..], vec![0xab; custom.data_len_bits() / 8]);
            }
        }
    }
    Ok(())
}
