use crate::cell::{TonCell, TonHash};
use crate::errors::TonCoreResult;
use crate::traits::tlb::TLB;
use crate::types::TonAddress;
use crate::types::tlb_core::adapters::InternalAddress;
use crate::types::tlb_core::{Anycast, MsgAddress, MsgAddressExt, MsgAddressIntStd, MsgAddressIntVar};

#[derive(Debug, PartialEq, ton_macros::TLB)]
struct Payment {
    #[tlb(adapter = "InternalAddress")]
    recipient: TonAddress,
    query_id: u64,
}

#[test]
fn test_internal_address_zero_and_inline_fields() -> TonCoreResult<()> {
    let payment = Payment {
        recipient: TonAddress::ZERO,
        query_id: 7,
    };
    let cell = payment.to_cell()?;
    assert_eq!(cell.data_len_bits(), 267 + 64);
    assert_eq!(Payment::from_cell(&cell)?, payment);
    let mut parser = cell.parser();
    assert_eq!(parser.read_num::<u8>(2)?, 0b10);
    assert!(!parser.read_bit()?);
    assert_eq!(parser.read_num::<i8>(8)?, 0);
    assert_eq!(TonHash::read(&mut parser)?, TonHash::ZERO);
    assert_eq!(parser.read_num::<u64>(64)?, 7);
    parser.ensure_empty()?;
    // The existing TonAddress contract remains unchanged.
    assert_eq!(TonAddress::ZERO.to_cell()?.data_len_bits(), 2);
    Ok(())
}

#[test]
fn test_internal_address_workchain_bounds() -> TonCoreResult<()> {
    for workchain in [-128, -1, 0, 127] {
        let address = TonAddress::new(workchain, TonHash::from_slice_sized(&[0x42; 32]));
        let mut builder = TonCell::builder();
        InternalAddress.write(&mut builder, &address)?;
        let cell = builder.build()?;
        assert_eq!(cell, address.to_cell()?);
        let mut parser = cell.parser();
        assert_eq!(InternalAddress.read(&mut parser)?, address);
        parser.ensure_empty()?;
    }
    for workchain in [-129, 128, i32::MIN, i32::MAX] {
        let mut builder = TonCell::builder();
        assert!(InternalAddress.write(&mut builder, &TonAddress::new(workchain, TonHash::ZERO)).is_err());
    }
    Ok(())
}

#[test]
fn test_internal_address_rejects_other_encodings() -> TonCoreResult<()> {
    let invalid = [
        MsgAddress::NONE.to_cell()?,
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
        assert!(InternalAddress.read(&mut cell.parser()).is_err());
    }
    let mut builder = TonCell::builder();
    builder.write_num(&0b10u8, 2)?;
    assert!(InternalAddress.read(&mut builder.build()?.parser()).is_err());
    Ok(())
}
