use super::*;
use crate::cell::TonCell;
use num_bigint::BigUint;

#[test]
fn test_block_tlb_var_len_bits_num() -> anyhow::Result<()> {
    // len in bits
    let obj = VarLenBits::<u32, 8>::new(1u8, 4);
    let cell = obj.to_cell()?;

    // 8 bits of length (value = 4) + 4 bits of data (value = 1)
    assert_eq!(cell.underlying_storage(), &[0b00000100, 0b00010000]);
    let parsed = VarLenBits::<u32, 8>::from_cell(&cell)?;
    assert_eq!(obj, parsed);
    Ok(())
}

#[test]
fn test_block_tlb_var_len_bytes_primitive_num() -> anyhow::Result<()> {
    // len in bytes
    let obj = VarLenBytes::<u32, 16>::new(1u8, 24);
    let cell = obj.to_cell()?;
    // 16 bits of length (value = 3 bytes <==> 24 bits), and then 24 bits of data (value = 1)
    assert_eq!(cell.underlying_storage(), &[0b00000000, 0b00000011, 0b00000000, 0b00000000, 0b00000001]);
    let parsed = VarLenBytes::<u32, 16>::from_cell(&cell)?;
    assert_eq!(obj, parsed);

    // the same, additionally check builder state
    let mut builder = TonCell::builder();
    obj.write(&mut builder)?;
    assert_eq!(builder.data_bits_left(), 1023 - 40);

    Ok(())
}

#[test]
fn test_block_tlb_var_len_bytes_big_uint() -> anyhow::Result<()> {
    // len in bytes
    let obj = VarLenBytes::<BigUint, 16>::new(1u8, 24);
    let cell = obj.to_cell()?;
    // 16 bits of length (value = 3 bytes <==> 24 bits), and then 24 bits of data (value = 1)
    assert_eq!(cell.underlying_storage(), &[0b00000000, 0b00000011, 0b00000000, 0b00000000, 0b00000001]);
    let parsed = VarLenBytes::<BigUint, 16>::from_cell(&cell)?;
    assert_eq!(obj, parsed);

    // the same, additionally check builder state
    let mut builder = TonCell::builder();
    obj.write(&mut builder)?;
    assert_eq!(builder.data_bits_left(), 1023 - 40);

    Ok(())
}
