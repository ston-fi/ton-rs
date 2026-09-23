use super::*;
use sha2::{Digest, Sha256};
use ton::ton_core::{
    cell::TonHash,
    types::{TonAddress, tlb_core::MsgAddress},
};
fn hint(cell: &TonCell, id: u32) -> anyhow::Result<Vec<u8>> {
    let encoded =
        hints::encode(cell, SigningPolicy::ClearOnly).map_err(|error| anyhow::anyhow!("hint {id}: {error}"))?;
    assert_eq!(encoded[0], 1);
    assert_eq!(u32::from_be_bytes(encoded[1..5].try_into()?), id);
    Ok(encoded[7..].to_vec())
}
fn start(op: u32) -> anyhow::Result<ton::ton_core::cell::CellBuilder> {
    let mut builder = TonCell::builder();
    builder.write_num(&op, 32)?;
    if op != 0 {
        builder.write_num(&0u64, 64)?;
    }
    Ok(builder)
}
#[test]
fn test_all_hint_families_and_dns_empty_capabilities() -> anyhow::Result<()> {
    let addr = TonAddress::ZERO;
    let mut builder = start(0)?;
    builder.write_bits(b"hello", 40)?;
    assert_eq!(hint(&builder.build()?, 0)?, b"hello");
    for (op, id) in [(0x0f8a7ea5, 1), (0x5fcc3d14, 2), (0x595f07bc, 3)] {
        let mut builder = start(op)?;
        if id != 2 {
            TLBCoins::ONE.write(&mut builder)?;
        }
        if id != 3 {
            addr.to_msg_address_int().write(&mut builder)?;
        }
        addr.to_msg_address_int().write(&mut builder)?;
        builder.write_bit(false)?;
        if id != 3 {
            TLBCoins::ZERO.write(&mut builder)?;
            builder.write_bit(false)?;
        }
        hint(&builder.build()?, id)?;
    }
    for (op, id) in [(0x7258a69b, 4), (0x1001, 6)] {
        let mut builder = start(op)?;
        addr.to_msg_address_int().write(&mut builder)?;
        hint(&builder.build()?, id)?;
    }
    let mut builder = start(0x1000)?;
    TLBCoins::ONE.write(&mut builder)?;
    hint(&builder.build()?, 5)?;
    let mut builder = start(0x47d54391)?;
    builder.write_num(&9u64, 64)?;
    assert_eq!(hint(&builder.build()?, 7)?, [0, 1, 0, 0, 0, 0, 0, 0, 0, 9]);
    let mut builder = start(0x69fb306c)?;
    addr.to_msg_address_int().write(&mut builder)?;
    builder.write_num(&1_700_000_000u64, 48)?;
    builder.write_bit(true)?;
    builder.write_bit(false)?;
    let data = hint(&builder.build()?, 8)?;
    assert_eq!(&data[34..], &hex::decode("00006553f1000100")?);
    let mut builder = start(0x4eb1f0f9)?;
    builder.write_bits(Sha256::digest(b"wallet"), 256)?;
    let mut record_builder = TonCell::builder();
    record_builder.write_num(&0x9fd3u16, 16)?;
    addr.to_msg_address_int().write(&mut record_builder)?;
    record_builder.write_num(&1u8, 8)?;
    record_builder.write_bit(false)?;
    builder.write_ref(record_builder.build()?)?;
    let data = hint(&builder.build()?, 9)?;
    assert_eq!(&data[data.len() - 2..], &[1, 0]);
    let mut builder = start(8)?;
    TonHash::ZERO.write(&mut builder)?;
    hint(&builder.build()?, 10)?;
    for (op, id) in [(0x7bcd1fefu32, 11), (0xda803efd, 12)] {
        let mut builder = TonCell::builder();
        builder.write_num(&op, 32)?;
        builder.write_num(&1u64, 64)?;
        TLBCoins::ONE.write(&mut builder)?;
        if id == 12 {
            TLBCoins::ONE.write(&mut builder)?;
        }
        hint(&builder.build()?, id)?;
    }
    let mut comment = start(0)?;
    comment.write_bits(b"hi", 16)?;
    let mut msg = Msg::new(CommonMsgInfoInt::new(addr.to_msg_address_int().into(), TLBCoins::ONE), comment.build()?);
    msg.body.layout = EitherRefLayout::Native;
    let mut builder = start(0xa7733acd)?;
    builder.write_num(&3u8, 8)?;
    builder.write_ref(msg.to_cell()?)?;
    hint(&builder.build()?, 13)?;
    Ok(())
}
#[test]
fn test_layout_policy_and_trailing_data() -> anyhow::Result<()> {
    let mut builder = start(0x1000)?;
    TLBCoins::ONE.write(&mut builder)?;
    builder.write_bit(false)?;
    let invalid = builder.build()?;
    assert!(hints::encode(&invalid, SigningPolicy::ClearOnly).is_err());
    assert_eq!(hints::encode(&invalid, SigningPolicy::AllowOpaque)?, [0]);
    let mut builder = start(0)?;
    builder.write_bits([0x7f], 8)?;
    assert!(hints::encode(&builder.build()?, SigningPolicy::ClearOnly).is_err());
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
    if let CommonMsgInfo::Int(info) = &mut msg.info {
        info.src = MsgAddress::from(TonAddress::ZERO.to_msg_address_int());
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
        assert_eq!(hex::encode(&hints::encode(&cell, SigningPolicy::ClearOnly)?[1..]), fields[2], "hint {}", fields[0]);
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
            let mut builder = start(op)?;
            if id != 2 {
                TLBCoins::ONE.write(&mut builder)?;
            }
            if id != 3 {
                TonAddress::ZERO.to_msg_address_int().write(&mut builder)?;
            }
            TonAddress::ZERO.to_msg_address_int().write(&mut builder)?;
            builder.write_bit(true)?;
            builder.write_ref(custom.clone())?;
            if id != 3 {
                TLBCoins::ZERO.write(&mut builder)?;
                builder.write_bit(false)?;
            }
            let cell = builder.build()?;
            let clear = hints::encode(&cell, SigningPolicy::ClearOnly);
            let embedded = id == 3 && matches!(custom.data_len_bits(), 0 | 256) && custom.refs().is_empty();
            assert_eq!(clear.is_ok(), embedded, "hint {id}, custom {custom:?}");
            let encoded = hints::encode(&cell, SigningPolicy::AllowOpaque)?;
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

#[test]
fn test_enum_rejects_unknown_trailing_and_normalized_nft_payloads() -> anyhow::Result<()> {
    let unknown = start(0xdeadbeef)?.build()?;
    assert!(matches!(hints::encode(&unknown, SigningPolicy::ClearOnly), Err(TonLedgerError::OpaquePayload)));
    assert_eq!(hints::encode(&unknown, SigningPolicy::AllowOpaque)?, [0]);

    let mut extra_reference = start(0x1000)?;
    TLBCoins::ONE.write(&mut extra_reference)?;
    extra_reference.write_ref(TonCell::empty().clone())?;
    assert!(hints::encode(&extra_reference.build()?, SigningPolicy::ClearOnly).is_err());

    // Existing firmware vectors cover standard zero addresses. addr_none must
    // remain distinct even though NFTTransferMsg parses it as TonAddress::ZERO.
    let mut transfer = ton::contracts::tep::nft::nft_transfer_msg::NFTTransferMsg::new(&TonAddress::ZERO);
    transfer.forward_payload.layout = EitherRefLayout::ToCell;
    assert!(hints::encode(&transfer.to_cell()?, SigningPolicy::ClearOnly).is_err());
    Ok(())
}

#[test]
fn test_dns_capability_variants_and_malformed_records() -> anyhow::Result<()> {
    for (flags, has_wallet, terminator, expected) in [
        (0, None, false, Some(vec![0])),
        (1, Some(false), false, Some(vec![1, 0])),
        (1, Some(true), false, Some(vec![1, 1])),
        (1, Some(true), true, None),
        (2, None, false, None),
    ] {
        let mut record = TonCell::builder();
        record.write_num(&0x9fd3u16, 16)?;
        TonAddress::ZERO.to_msg_address_int().write(&mut record)?;
        record.write_num(&flags, 8)?;
        if let Some(has_wallet) = has_wallet {
            record.write_bit(has_wallet)?;
            if has_wallet {
                record.write_num(&0x2177u16, 16)?;
                record.write_bit(terminator)?;
            }
        }
        let mut message = start(0x4eb1f0f9)?;
        message.write_bits(Sha256::digest(b"wallet"), 256)?;
        message.write_ref(record.build()?)?;
        let cell = message.build()?;
        if let Some(expected) = expected {
            let data = hint(&cell, 9)?;
            assert_eq!(&data[36..], expected);
        } else {
            assert!(matches!(hints::encode(&cell, SigningPolicy::ClearOnly), Err(TonLedgerError::OpaquePayload)));
            assert_eq!(hints::encode(&cell, SigningPolicy::AllowOpaque)?, [0]);
        }
    }
    Ok(())
}

#[test]
fn test_nested_unsupported_records_stay_opaque_and_truncation_stays_cell_error() -> anyhow::Result<()> {
    for (record, unsupported) in [(start(0xdeadbeef)?.build()?, true), (TonCell::empty().clone(), false)] {
        let dns = tlb::DnsChangeRecord {
            query_id: 0,
            key: TonHash::from_slice(&Sha256::digest(b"wallet"))?,
            record: tlb::TrailingRecordRef(Some(record.clone())),
        };
        let internal_message =
            Msg::new(CommonMsgInfoInt::new(TonAddress::ZERO.to_msg_address_int().into(), TLBCoins::ONE), record);
        let vesting = tlb::Vesting {
            query: 0,
            mode: 3,
            message: internal_message.to_cell()?.into(),
        };
        for cell in [dns.to_cell()?, vesting.to_cell()?] {
            let error = hints::encode(&cell, SigningPolicy::ClearOnly)
                .err()
                .ok_or_else(|| anyhow::anyhow!("accepted unsupported record"))?;
            if unsupported {
                assert!(matches!(error, TonLedgerError::OpaquePayload), "{error}");
            } else {
                assert!(matches!(error, TonLedgerError::Cell(_)), "{error}");
            }
            assert_eq!(hints::encode(&cell, SigningPolicy::AllowOpaque)?, [0]);
        }
    }
    Ok(())
}
