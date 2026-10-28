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
