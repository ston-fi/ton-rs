use crate::block_tlb::{FromTVMStack, TVMNull, TVMStack, ToTVMStack};
use fastnum::I512;
use ton_core::cell::TonCell;
use ton_core::traits::tlb::TLB;
use ton_core::types::TonAddress;
use ton_core::types::tlb_core::{MsgAddress, MsgAddressExt, MsgAddressInt, MsgAddressIntVar, TLBRef};

#[test]
fn test_native_integer_arguments_preserve_values_beyond_i64() -> anyhow::Result<()> {
    let mut stack = TVMStack::default();
    u8::MAX.to_stack(&mut stack)?;
    u64::MAX.to_stack(&mut stack)?;
    u128::MAX.to_stack(&mut stack)?;
    i128::MIN.to_stack(&mut stack)?;
    let mut stack = TVMStack::from_boc(stack.to_boc()?)?;
    assert_eq!(stack.pop_num()?, -(I512::ONE << 127u32));
    assert_eq!(stack.pop_num()?, (I512::ONE << 128u32) - I512::ONE);
    assert_eq!(stack.pop_num()?, (I512::ONE << 64u32) - I512::ONE);
    assert_eq!(stack.pop_num()?, I512::from(u8::MAX));
    stack.ensure_empty()?;
    Ok(())
}

#[test]
fn test_internal_address_results_preserve_variants_and_reject_non_internal() -> anyhow::Result<()> {
    let addresses = [
        TonAddress::ZERO.to_msg_address_int(),
        MsgAddressInt::Var(MsgAddressIntVar {
            anycast: None,
            workchain: 256,
            addr_bits_len: 3,
            address: vec![0xa0],
        }),
    ];
    for address in addresses {
        for as_slice in [false, true] {
            let mut stack = TVMStack::default();
            if as_slice {
                stack.push_cell_slice(address.to_cell()?);
            } else {
                stack.push_cell(address.to_cell()?);
            }
            assert_eq!(MsgAddressInt::from_stack(&mut stack)?, address);
            stack.ensure_empty()?;
        }
    }
    for address in [MsgAddress::NONE, MsgAddressExt::new(vec![0x80], 1).into()] {
        let mut stack = TVMStack::default();
        stack.push_cell_slice(address.to_cell()?);
        assert!(MsgAddressInt::from_stack(&mut stack).is_err());
    }
    let mut stack = TVMStack::new(vec![TVMNull.into()]);
    assert!(MsgAddressInt::from_stack(&mut stack).is_err());
    Ok(())
}

#[test]
fn test_derived_result_wraps_one_cell_or_slice_without_an_extra_reference() -> anyhow::Result<()> {
    #[derive(Debug, PartialEq, ton_macros::FromTVMStack)]
    #[from_tvm_stack(ensure_empty = true)]
    struct VersionedCode {
        version: u16,
        code: TLBRef<TonCell>,
    }

    let mut builder = TonCell::builder();
    builder.write_num(&0xabu8, 8)?;
    builder.write_ref(TonCell::empty().clone())?;
    let code = builder.build()?;
    for as_slice in [false, true] {
        let mut stack = TVMStack::default();
        stack.push_tiny_int(17);
        if as_slice {
            stack.push_cell_slice(code.clone());
        } else {
            stack.push_cell(code.clone());
        }
        let value = VersionedCode::from_stack_boc(stack.to_boc()?)?;
        assert_eq!(value.version, 17);
        assert_eq!(value.code.into_inner(), code);
    }
    let mut stack = TVMStack::default();
    stack.push_tiny_int(17);
    stack.push(TVMNull.into());
    assert!(VersionedCode::from_stack(&mut stack).is_err());
    assert!(Option::<TLBRef<TonCell>>::from_stack(&mut TVMStack::new(vec![TVMNull.into()]))?.is_none());
    Ok(())
}

#[test]
fn test_internal_address_arguments_preserve_zero_anycast_and_variable_forms() -> anyhow::Result<()> {
    use crate::block_tlb::TVMStackValue;
    use ton_core::cell::TonHash;
    use ton_core::types::tlb_core::{Anycast, MsgAddressIntStd};
    let addresses = [
        TonAddress::ZERO.to_msg_address_int(),
        MsgAddressInt::Std(MsgAddressIntStd {
            anycast: Some(Anycast::new(1, vec![0x80])),
            workchain: -1,
            address: TonHash::ZERO,
        }),
        MsgAddressInt::Var(MsgAddressIntVar {
            anycast: None,
            workchain: 256,
            addr_bits_len: 3,
            address: vec![0xa0],
        }),
    ];
    for address in addresses {
        let mut stack = TVMStack::default();
        stack.push_tiny_int(41);
        address.to_stack(&mut stack)?;
        assert_eq!(stack.len(), 2);
        assert!(matches!(stack.last(), Some(TVMStackValue::CellSlice(_))));
        let cell = stack.pop_cell()?;
        assert_eq!(cell, address.to_cell()?);
        assert_eq!(MsgAddressInt::from_cell(&cell)?, address);
        assert_eq!(stack.pop_num()?, I512::from(41));
    }
    Ok(())
}

#[test]
fn test_derived_address_adapters_preserve_order_zero_and_nullable_wire_values() -> anyhow::Result<()> {
    use ton_core::types::tlb_core::adapters::{InternalAddress, NullableAddress};
    #[derive(Debug, PartialEq, ton_macros::FromTVMStack)]
    #[from_tvm_stack(ensure_empty = true)]
    struct Owners {
        version: u8,
        #[from_tvm_stack(adapter = InternalAddress)]
        owner: TonAddress,
        #[from_tvm_stack(adapter = "NullableAddress")]
        pending_owner: Option<TonAddress>,
    }
    for as_slice in [false, true] {
        for pending in [MsgAddress::NONE, MsgAddress::Int(TonAddress::ZERO.to_msg_address_int())] {
            let mut stack = TVMStack::default();
            stack.push_tiny_int(7);
            for cell in [TonAddress::ZERO.to_msg_address_int().to_cell()?, pending.to_cell()?] {
                // Ordinary cell tails remain accepted by the selected TLB adapter.
                let mut builder = TonCell::builder();
                builder.write_cell(&cell)?;
                builder.write_bit(true)?;
                builder.write_ref(TonCell::empty().clone())?;
                if as_slice {
                    stack.push_cell_slice(builder.build()?);
                } else {
                    stack.push_cell(builder.build()?);
                }
            }
            let mut extra = stack.clone();
            extra.insert(0, TVMNull.into());
            assert!(Owners::from_stack(&mut extra).is_err());
            let mut missing = stack.clone();
            missing.remove(0);
            assert!(Owners::from_stack(&mut missing).is_err());
            let decoded = Owners::from_stack(&mut stack)?;
            assert_eq!(decoded.version, 7);
            assert_eq!(decoded.owner, TonAddress::ZERO);
            assert_eq!(decoded.pending_owner, if pending == MsgAddress::NONE { None } else { Some(TonAddress::ZERO) });
            stack.ensure_empty()?;
        }
    }
    Ok(())
}

#[test]
fn test_derived_adapters_reject_null_wrong_kinds_and_nonstandard_addresses() -> anyhow::Result<()> {
    use ton_core::cell::TonHash;
    use ton_core::types::tlb_core::adapters::{InternalAddress, NullableAddress};
    use ton_core::types::tlb_core::{Anycast, MsgAddressIntStd};
    #[derive(ton_macros::FromTVMStack)]
    struct Required {
        #[from_tvm_stack(adapter = InternalAddress)]
        _owner: TonAddress,
    }
    #[derive(ton_macros::FromTVMStack)]
    struct Nullable {
        #[from_tvm_stack(adapter = NullableAddress)]
        _owner: Option<TonAddress>,
    }
    let mut wrong_kinds = TVMStack::default();
    wrong_kinds.push_tiny_int(0);
    for stack in [TVMStack::new(vec![TVMNull.into()]), wrong_kinds] {
        assert!(Required::from_stack(&mut stack.clone()).is_err());
        assert!(Nullable::from_stack(&mut stack.clone()).is_err());
    }
    let invalid = [
        MsgAddressExt::new([0xab], 8).to_cell()?,
        MsgAddressIntVar {
            anycast: None,
            addr_bits_len: 3,
            workchain: 256,
            address: vec![0xa0],
        }
        .to_cell()?,
        MsgAddressIntStd {
            anycast: Some(Anycast::new(1, vec![0x80])),
            workchain: 0,
            address: TonHash::ZERO,
        }
        .to_cell()?,
        TonCell::empty().clone(),
    ];
    for cell in invalid {
        let mut stack = TVMStack::default();
        stack.push_cell_slice(cell);
        assert!(Required::from_stack(&mut stack.clone()).is_err());
        assert!(Nullable::from_stack(&mut stack).is_err());
    }
    let mut stack = TVMStack::default();
    stack.push_cell_slice(MsgAddress::NONE.to_cell()?);
    assert!(Required::from_stack(&mut stack).is_err());
    Ok(())
}

#[test]
fn test_derived_adapter_expression_configuration_and_stack_flags() -> anyhow::Result<()> {
    use ton_core::types::tlb_core::adapters::ConstLen;
    #[derive(Debug, PartialEq, ton_macros::FromTVMStack)]
    #[from_tvm_stack(allow_extra = true, ensure_empty = true)]
    struct Nibble {
        #[from_tvm_stack(adapter = ConstLen::<u8>::new(4))]
        value: u8,
    }
    let mut builder = TonCell::builder();
    builder.write_num(&9u8, 4)?;
    let mut stack = TVMStack::default();
    stack.push_cell(builder.build()?);
    stack.push_tiny_int(123);
    assert_eq!(Nibble::from_stack(&mut stack)?, Nibble { value: 9 });
    stack.ensure_empty()?;
    // Unannotated fields retain their existing NULL-to-zero convention.
    #[derive(ton_macros::FromTVMStack)]
    struct Native {
        owner: TonAddress,
    }
    assert_eq!(Native::from_stack(&mut TVMStack::new(vec![TVMNull.into()]))?.owner, TonAddress::ZERO);
    Ok(())
}
