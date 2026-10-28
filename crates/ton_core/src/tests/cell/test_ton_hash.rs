use super::*;
use std::str::FromStr;
use tokio_test::assert_err;

#[test]
fn test_ton_hash_display() -> anyhow::Result<()> {
    let data = [255u8; 32];
    let hash = TonHash::from(data);
    assert_eq!(format!("{hash}"), "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF");
    assert_eq!(format!("{hash:X}"), "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF");
    Ok(())
}

#[test]
fn test_ton_hash_from() -> anyhow::Result<()> {
    let data = [1u8; 32];
    let hash = TonHash::from(data);
    assert_eq!(hash.as_slice(), &data);
    Ok(())
}

#[test]
fn test_ton_hash_slice() -> anyhow::Result<()> {
    let data = [1u8; 32];
    let hash = TonHash::from_slice(&data)?;
    assert_eq!(hash.as_slice(), &data);

    let wrong_data = [1u8; 31];
    assert_err!(TonHash::from_slice(&wrong_data));
    Ok(())
}

#[test]
fn test_ton_hash_from_slice_sized() -> anyhow::Result<()> {
    let data = [1u8; 32];
    let hash = TonHash::from_slice_sized(&data);
    assert_eq!(hash.as_slice(), &data);
    Ok(())
}

#[test]
fn test_ton_hash_from_vec() -> anyhow::Result<()> {
    let data = vec![1u8; 32];
    let hash = TonHash::from_vec(data.clone())?;
    assert_eq!(hash.as_slice(), &data);
    Ok(())
}

#[test]
fn test_ton_hash_from_hex() -> anyhow::Result<()> {
    let data = [255u8; 32];
    for hex_str in &[
        "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
        "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
        "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
        "0Xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
    ] {
        let hash = TonHash::from_str(hex_str)?;
        assert_eq!(hash.as_slice(), &data);
    }
    Ok(())
}

#[test]
fn test_ton_hash_from_base64() -> anyhow::Result<()> {
    let data = [
        159, 115, 38, 150, 26, 81, 188, 250, 200, 211, 46, 142, 240, 183, 144, 7, 187, 83, 144, 183, 68, 163, 90, 117,
        106, 189, 241, 66, 113, 59, 99, 240,
    ];
    let base64_str = "n3MmlhpRvPrI0y6O8LeQB7tTkLdEo1p1ar3xQnE7Y/A=";
    let hash = TonHash::from_str(base64_str)?;
    assert_eq!(hash.as_slice(), &data);
    Ok(())
}

#[test]
fn test_ton_hash_rewrite_first_bits() -> anyhow::Result<()> {
    let hash_one = TonHash::from_slice_sized(&[255u8; 32]);
    let mut hash = hash_one;
    hash.rewrite_first_bits(0u8, 0)?;
    assert_eq!(hash, hash_one);

    hash.rewrite_first_bits(0u8, 1)?;
    assert_eq!(hash.as_slice_sized()[0], 0b0111_1111);

    hash.rewrite_first_bits(1u8, 1)?;
    assert_eq!(hash.as_slice_sized()[0], 0b1111_1111);

    hash.rewrite_first_bits(0u8, 2)?;
    assert_eq!(hash.as_slice_sized()[0], 0b0011_1111);

    hash.rewrite_first_bits(5u8, 3)?;
    assert_eq!(hash.as_slice_sized()[0], 0b1011_1111);

    hash.rewrite_first_bits(0u8, 8)?;
    assert_eq!(hash.as_slice_sized()[0], 0b0000_0000);

    let mut hash = TonHash::ZERO;
    hash.rewrite_first_bits(u16::MAX, 9)?;
    assert_eq!(hash.as_slice_sized()[0], 0b1111_1111);
    assert_eq!(hash.as_slice_sized()[1], 0b1000_0000);

    let mut hash = TonHash::ZERO;
    hash.rewrite_first_bits(u128::MAX, 128)?;
    for i in 0..16 {
        assert_eq!(hash.as_slice_sized()[i], 0xFF);
    }

    let mut hash = TonHash::ZERO;
    assert_err!(hash.rewrite_first_bits(1u8, 129));
    Ok(())
}

#[test]
fn test_ton_hash_to_num() -> anyhow::Result<()> {
    let zero_num = TonHash::ZERO.to_u256();
    assert_eq!(zero_num, U256::from(0u8));

    let max_hash = TonHash::from_slice_sized(&[255u8; 32]);
    let max_num = max_hash.to_u256();
    assert_eq!(max_num, U256::MAX);
    Ok(())
}
