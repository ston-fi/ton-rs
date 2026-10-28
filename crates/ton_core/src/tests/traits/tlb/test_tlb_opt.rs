use super::*;
use crate::cell::TonCell;
use ton_macros::TLB;

#[derive(Debug, PartialEq, TLB)]
struct TestType1(i32);

#[test]
fn test_option() -> anyhow::Result<()> {
    let obj1 = Some(TestType1(1));
    let obj2: Option<TestType1> = None;
    let mut builder = TonCell::builder();
    obj1.write(&mut builder)?;
    obj2.write(&mut builder)?;

    let cell = builder.build()?;
    let mut parser = cell.parser();
    let parsed_obj1: Option<TestType1> = TLB::read(&mut parser)?;
    let parsed_obj2: Option<TestType1> = TLB::read(&mut parser)?;
    assert_eq!(obj1, parsed_obj1);
    assert_eq!(None, parsed_obj2);

    // check layout
    let mut parser = cell.parser();
    assert!(parser.read_bit()?); // Some
    parser.seek_bits(32)?; // skipping
    assert!(!parser.read_bit()?); // None
    Ok(())
}
