use super::*;
use crate::cell::TonCell;
use crate::traits::tlb::TLB;
use ton_macros::TLB;

#[test]
fn test_const_len() -> anyhow::Result<()> {
    let mut builder = TonCell::builder();
    ConstLen::<u32>::new(24).write(&mut builder, &1u32)?;
    let cell = builder.build()?;
    assert_eq!(cell.underlying_storage(), &[0, 0, 1]);
    let parsed = ConstLen::<u32>::new(24).read(&mut cell.parser())?;
    assert_eq!(parsed, 1u32);
    Ok(())
}

#[derive(TLB)]
struct TestType {
    #[tlb(bits_len = 4)]
    a: u32,
}

#[test]
fn test_cont_len_bits_len() -> anyhow::Result<()> {
    let mut builder = TonCell::builder();
    TestType { a: 1 }.write(&mut builder)?;
    let cell = builder.build()?;
    assert_eq!(cell.underlying_storage(), &[0b00010000]);
    let parsed = TestType::read(&mut cell.parser())?;
    assert_eq!(parsed.a, 1u32);
    Ok(())
}
