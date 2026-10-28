use crate::bits_utils::BitsUtils;

#[test]
fn test_bits_equal() -> anyhow::Result<()> {
    let left = [0b11001100, 0b10101010, 0b11110100];
    let right = [0b11001100, 0b10101010, 0b11110000];
    assert!(BitsUtils::equal(&left, &right, 3));
    assert!(BitsUtils::equal(&left, &right, 8));
    assert!(BitsUtils::equal(&left, &right, 15));
    assert!(BitsUtils::equal(&left, &right, 20));
    assert!(BitsUtils::equal(&left, &right, 21));
    assert!(!BitsUtils::equal(&left, &right, 22));
    assert!(!BitsUtils::equal(&left, &right, 23));
    assert!(!BitsUtils::equal(&left, &right, 24));
    assert!(!BitsUtils::equal(&left, &right, 25));
    Ok(())
}

#[test]
fn test_rewrite_bits() {
    let src = vec![0b11001100, 0b10101010];
    let mut dst = vec![0b00000000, 0b00000000];
    assert!(BitsUtils::rewrite(&src, 4, &mut dst, 8, 8));
    assert_eq!(dst, vec![0b00000000, 0b11001010]);

    let src = vec![0b11001100, 0b10101010];
    let mut dst = vec![0b00000000, 0b00000000];
    assert!(BitsUtils::rewrite(&src, 0, &mut dst, 0, 16));
    assert_eq!(dst, src);

    let src = vec![0b11001100, 0b10101010];
    let mut dst = vec![0b00000000, 0b00000000];
    assert!(BitsUtils::rewrite(&src, 0, &mut dst, 0, 8));
    assert_eq!(dst[0], src[0]);
    assert_eq!(dst[1], 0b00000000);

    assert!(!BitsUtils::rewrite(&src, 14, &mut dst, 6, 10));
}

#[test]
fn test_read_with_offset() {
    let src = vec![0b11000011, 0b10100101];
    let mut dst = vec![0b00000000, 0b0000101010];
    assert!(BitsUtils::read_with_offset(&src, &mut dst, 4, 8));
    assert_eq!(dst, vec![0b00111010, 0b0000101010]);

    let src = vec![0b11001100, 0b10101010];
    let mut dst = vec![0b00000000, 0b0000010101];
    assert!(BitsUtils::read_with_offset(&src, &mut dst, 0, 16));
    assert_eq!(dst, src);

    let src = vec![0b11001100, 0b10101010];
    let mut dst = vec![0b00000000, 0b0000101011];
    assert!(BitsUtils::read_with_offset(&src, &mut dst, 0, 15));
    assert_eq!(dst[0], src[0]);
    assert_eq!(dst[1], 0b10101011);

    assert!(!BitsUtils::read_with_offset(&src, &mut dst, 14, 10));
}
