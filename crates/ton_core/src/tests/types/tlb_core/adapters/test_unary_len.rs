use super::*;
use crate::cell::TonCell;

#[test]
fn test_block_tlb_unary() -> anyhow::Result<()> {
    let mut builder = TonCell::builder();
    let unary = UnaryLen(5);
    unary.write(&mut builder)?;
    let cell = builder.build()?;
    assert_eq!(cell.data_len_bits(), 6);
    assert_eq!(cell.underlying_storage(), vec![0b11111000]);
    let parsed_unary = UnaryLen::read(&mut cell.parser())?;
    assert_eq!(parsed_unary, unary);
    Ok(())
}
