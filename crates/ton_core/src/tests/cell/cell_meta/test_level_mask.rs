use super::*;

#[test]
fn test_level_mask() {
    let mask = LevelMask::new(0b1010);
    assert_eq!(mask.level(), 4);
    assert_eq!(mask.hash_index(), 2);
    assert_eq!(mask.hash_count(), 3);
    assert_eq!(mask.apply(0), LevelMask::new(0));
    assert!(!mask.is_significant(1));
    assert!(mask.is_significant(2));
    assert!(!mask.is_significant(3));
    assert!(mask.is_significant(4));

    assert_eq!(mask | 0b100, LevelMask::new(0b1110));
    assert_eq!(mask >> 1, LevelMask::new(0b101));
}
