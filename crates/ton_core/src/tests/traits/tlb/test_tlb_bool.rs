use super::*;
use crate::cell::TonCell;

#[test]
fn test_bool() -> anyhow::Result<()> {
    let mut builder = TonCell::builder();
    true.write(&mut builder)?;
    false.write(&mut builder)?;
    let cell = builder.build()?;
    assert_eq!(cell.data_len_bits(), 2);
    assert_eq!(cell.underlying_storage(), vec![0b10000000]);
    Ok(())
}
