use super::*;
use ton_core::traits::tlb::TLB;

#[test]
fn test_tvm_tiny_int_serialization() -> anyhow::Result<()> {
    let tiny_int = TVMTinyInt { value: 1 };
    let cell = tiny_int.to_cell()?;

    eprintln!("TinyInt cell BOC: {}", cell.to_boc_base64()?);

    let parsed = TVMTinyInt::from_cell(&cell)?;
    eprintln!("Parsed value: {}", parsed.value);

    assert_eq!(parsed.value, 1);
    Ok(())
}

#[test]
fn test_tvm_stack_value_enum_serialization() -> anyhow::Result<()> {
    let stack_value = TVMStackValue::TinyInt(TVMTinyInt { value: 1 });
    let cell = stack_value.to_cell()?;

    eprintln!("TVMStackValue cell BOC: {}", cell.to_boc_base64()?);

    let parsed = TVMStackValue::from_cell(&cell)?;
    eprintln!("Parsed value: {:?}", parsed);

    match parsed {
        TVMStackValue::TinyInt(val) => assert_eq!(val.value, 1),
        _ => panic!("Expected TinyInt, got {:?}", parsed),
    }
    Ok(())
}
