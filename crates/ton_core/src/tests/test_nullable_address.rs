use crate::cell::{TonCell, TonHash};
use crate::errors::TonCoreResult;
use crate::traits::tlb::TLB;
use crate::types::TonAddress;
use crate::types::tlb_core::adapters::NullableAddress;
use crate::types::tlb_core::{Anycast, MsgAddress, MsgAddressExt, MsgAddressIntStd, MsgAddressIntVar};

#[derive(Debug, PartialEq, ton_macros::TLB)]
struct Payment {
    #[tlb(adapter = "NullableAddress")]
    recipient: Option<TonAddress>,
    query_id: u64,
}

#[test]
fn test_nullable_address_tolk_wire_and_inline_fields() -> anyhow::Result<()> {
    let nonzero = TonAddress::new(-1, TonHash::from_slice_sized(&[0x42; 32]));
    for recipient in [None, Some(TonAddress::ZERO), Some(nonzero)] {
        let payment = Payment { recipient, query_id: 7 };
        let cell = payment.to_cell()?;
        // Build the independent Tolk wire using raw MsgAddress constructors.
        let mut expected = TonCell::builder();
        match recipient {
            None => MsgAddress::NONE.write(&mut expected)?,
            Some(address) => MsgAddressIntStd {
                anycast: None,
                workchain: i8::try_from(address.workchain)?,
                address: address.hash,
            }
            .write(&mut expected)?,
        }
        expected.write_num(&7u64, 64)?;
        assert_eq!(cell, expected.build()?);
        assert_eq!(cell.data_len_bits(), if recipient.is_none() { 2 + 64 } else { 267 + 64 });
        assert_eq!(Payment::from_cell(&cell)?, payment);
        let mut parser = cell.parser();
        assert_eq!(NullableAddress.read(&mut parser)?, recipient);
        assert_eq!(parser.read_num::<u64>(64)?, 7);
        parser.ensure_empty()?;
    }
    // Ordinary Option/TonAddress serialization remains unchanged.
    assert_eq!(Option::<TonAddress>::None.to_cell()?.data_len_bits(), 1);
    assert_eq!(Some(TonAddress::ZERO).to_cell()?.data_len_bits(), 3);
    Ok(())
}

#[test]
fn test_nullable_address_rejects_unsupported_and_truncated_encodings() -> TonCoreResult<()> {
    let invalid = [
        MsgAddressExt::new([0xab], 8).to_cell()?,
        MsgAddressIntVar {
            anycast: None,
            addr_bits_len: 256,
            workchain: 0,
            address: vec![0; 32],
        }
        .to_cell()?,
        MsgAddressIntStd {
            anycast: Some(Anycast::new(1, vec![0x80])),
            workchain: 0,
            address: TonHash::ZERO,
        }
        .to_cell()?,
    ];
    for cell in invalid {
        assert!(NullableAddress.read(&mut cell.parser()).is_err());
    }
    // Both a truncated null tag and a truncated present address must fail.
    for (tag, bits) in [(0u8, 0), (0, 1), (0b10, 2)] {
        let mut builder = TonCell::builder();
        builder.write_num(&tag, bits)?;
        assert!(NullableAddress.read(&mut builder.build()?.parser()).is_err());
    }
    Ok(())
}

#[test]
fn test_nullable_address_rejects_out_of_range_present_workchain() {
    for workchain in [-129, 128] {
        let mut builder = TonCell::builder();
        let address = Some(TonAddress::new(workchain, TonHash::ZERO));
        assert!(NullableAddress.write(&mut builder, &address).is_err());
    }
}
