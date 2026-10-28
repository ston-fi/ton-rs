use crate::block_tlb::{FromTVMStack, TVMInt, TVMNull, TVMStack};
use crate::contracts::tep::snake_data::SnakeData;
use fastnum::I256;
use std::str::FromStr;
use tokio_test::assert_err;
use ton_core::traits::tlb::TLB;
use ton_core::types::TonAddress;
use ton_core::types::tlb_core::MsgAddress;
use ton_macros::FromTVMStack;

#[derive(FromTVMStack, Debug)]
#[from_tvm_stack(ensure_empty = true)]
struct TestStruct {
    pub field1: i64,
    pub field2: TonAddress,
    pub field3: bool,
}

#[test]
fn test_from_tvm_stack_derive_macro_works() -> anyhow::Result<()> {
    let mut stack = TVMStack::new(vec![]);
    stack.push_tiny_int(1);
    stack.push_cell(TonAddress::from_str("EQBiMfDMivebQb052Z6yR3jHrmwNhw1kQ5bcAUOBYsK_VPuK")?.to_cell()?);
    stack.push_tiny_int(0);

    let test_struct = TestStruct::from_stack(&mut stack)?;

    assert_eq!(test_struct.field1, 1i64);
    assert_eq!(test_struct.field2, TonAddress::from_str("EQBiMfDMivebQb052Z6yR3jHrmwNhw1kQ5bcAUOBYsK_VPuK")?);
    assert!(!test_struct.field3);
    Ok(())
}

#[test]
fn test_from_tvm_stack_ensure_empty_works() -> anyhow::Result<()> {
    let mut stack = TVMStack::new(vec![]);
    stack.push_tiny_int(1);
    stack.push_tiny_int(1);
    stack.push_tiny_int(1);
    stack.push_cell(TonAddress::from_str("EQBiMfDMivebQb052Z6yR3jHrmwNhw1kQ5bcAUOBYsK_VPuK")?.to_cell()?);
    stack.push_tiny_int(0);

    assert_err!(TestStruct::from_stack(&mut stack));
    Ok(())
}

#[test]
fn test_from_tvm_stack_numbers_impl() -> anyhow::Result<()> {
    let val = 42u32;
    let stack = TVMStack::new(vec![TVMInt { value: val.into() }.into()]);

    let parsed_u8 = u8::from_stack(&mut stack.clone())?;
    assert_eq!(parsed_u8, val as u8);

    let parsed_u32 = u32::from_stack(&mut stack.clone())?;
    assert_eq!(parsed_u32, val);

    let parsed_u128 = u128::from_stack(&mut stack.clone())?;
    assert_eq!(parsed_u128, val as u128);

    let parsed_i8 = i8::from_stack(&mut stack.clone())?;
    assert_eq!(parsed_i8, val as i8);
    let parsed_i64 = i64::from_stack(&mut stack.clone())?;
    assert_eq!(parsed_i64, val as i64);
    let parsed_i128 = i128::from_stack(&mut stack.clone())?;
    assert_eq!(parsed_i128, val as i128);

    let parsed_i256 = I256::from_stack(&mut stack.clone())?;
    assert_eq!(parsed_i256, I256::from(val));

    let parsed_u256 = fastnum::U256::from_stack(&mut stack.clone())?;
    assert_eq!(parsed_u256, fastnum::U256::from(val));

    let parsed_u1024 = fastnum::U1024::from_stack(&mut stack.clone())?;
    assert_eq!(parsed_u1024, fastnum::U1024::from(val));
    Ok(())
}

#[test]
fn test_from_tvm_stack_string() -> anyhow::Result<()> {
    let original = "Hello, TVMStack!".to_string();
    let snake_data = SnakeData::from_str(&original)?;
    let cell = snake_data.to_cell()?;

    let mut stack = TVMStack::new(vec![]);
    stack.push_cell(cell);

    let parsed: String = FromTVMStack::from_stack(&mut stack)?;
    assert_eq!(parsed, original);
    Ok(())
}

#[test]
fn test_from_tvm_stack_optional() -> anyhow::Result<()> {
    let mut stack = TVMStack::default();
    stack.push_tiny_int(11);
    stack.push(TVMNull.into());
    stack.push_tiny_int(33);
    #[derive(FromTVMStack, Debug)]
    struct TestStruct {
        pub opt_some: Option<u8>,
        pub opt_none: Option<u8>,
        pub another_field: u8,
    }

    let parsed = TestStruct::from_stack(&mut stack)?;
    assert_eq!(parsed.opt_some, Some(11));
    assert_eq!(parsed.opt_none, None);
    assert_eq!(parsed.another_field, 33);
    Ok(())
}

#[test]
fn test_msg_address_from_stack_accepts_null() -> anyhow::Result<()> {
    let mut stack = TVMStack::new(vec![TVMNull.into()]);

    assert_eq!(MsgAddress::from_stack(&mut stack)?, MsgAddress::NONE);
    assert!(stack.is_empty());
    Ok(())
}
