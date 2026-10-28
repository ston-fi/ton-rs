use super::*;

#[test]
fn test_all_bits_same() {
    for val in [BigUint::from(0u32), BigUint::from(0b1111u32), BigUint::from(0b1000u32)] {
        assert!(all_bits_same(&val));
    }

    let val = BigUint::from(0b1011u32);
    assert!(!all_bits_same(&val));
}

#[test]
fn test_common_prefix_len() {
    let a = BigUint::from(0b1011u32);
    let b = BigUint::from(0b1010u32);
    assert_eq!(common_prefix_len(&a, &b), 2);

    let a = BigUint::from(0b1011u32);
    let b = BigUint::from(0b1011u32);
    assert_eq!(common_prefix_len(&a, &b), 3);
}

#[test]
fn test_remove_leading_bit() {
    let val = BigUint::from(0b1011u32);
    assert_eq!(remove_leading_bit(&val), BigUint::from(0b011u32));

    let val = BigUint::from(0b1111u32);
    assert_eq!(remove_leading_bit(&val), BigUint::from(0b111u32));

    let val = BigUint::from(0b1u32);
    assert_eq!(remove_leading_bit(&val), BigUint::from(0u32));
}

#[test]
fn test_add_leading_bit() {
    let val = BigUint::from(0b1011u32);
    assert_eq!(add_leading_bit(&val, 4), BigUint::from(0b11011u32));

    let val = BigUint::from(0b1111u32);
    assert_eq!(add_leading_bit(&val, 4), BigUint::from(0b11111u32));

    let val = BigUint::from(0u32);
    assert_eq!(add_leading_bit(&val, 1), BigUint::from(0b10u32));
}
