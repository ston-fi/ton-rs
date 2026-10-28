use super::*;
use crate::cell::TonCell;
use tokio_test::assert_ok;
use ton_macros::TLB;

#[derive(Debug, PartialEq, TLB)]
struct TestType1(i32);

#[derive(Debug, PartialEq, TLB)]
struct TestType2(i64);

#[test]
fn test_either() -> anyhow::Result<()> {
    let obj1: TLBEither<TestType1, TestType2> = TLBEither::Left(TestType1(1));
    let obj2: TLBEither<TestType1, TestType2> = TLBEither::Right(TestType2(2));
    let mut builder = TonCell::builder();
    obj1.write(&mut builder)?;
    obj2.write(&mut builder)?;
    let cell = builder.build()?;
    let mut parser = cell.parser();
    let parsed_obj1 = TLB::read(&mut parser)?;
    let parsed_obj2 = TLB::read(&mut parser)?;
    assert_eq!(obj1, parsed_obj1);
    assert_eq!(obj2, parsed_obj2);

    // check raw data
    let mut parser = cell.parser();
    assert!(!parser.read_bit()?);
    assert_ok!(parser.read_bits(32)); // skipping
    assert!(parser.read_bit()?);
    Ok(())
}
