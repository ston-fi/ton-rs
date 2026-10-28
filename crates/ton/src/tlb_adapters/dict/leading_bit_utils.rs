use num_bigint::BigUint;
use num_traits::{One, Zero};

/// All functions except `add_leading_bit` expect 1 extra leading bit in `val` set to 1
pub fn all_bits_same(val: &BigUint) -> bool {
    if val.is_zero() {
        return true;
    }
    let origin_bits = val.bits();
    let all_zero = (val - 1u32).bits() != origin_bits;
    let all_ones = (val + 1u32).bits() != origin_bits;
    all_zero || all_ones
}

pub fn common_prefix_len(a: &BigUint, b: &BigUint) -> usize {
    let xor = a ^ b;
    (a.bits() - xor.bits() - 1) as usize // don't forget leading zero
}

pub fn remove_leading_bit(val: &BigUint) -> BigUint {
    let bits = val.bits();
    let mask = BigUint::one() << (bits - 1);
    val ^ mask
}

pub fn add_leading_bit(val: &BigUint, val_bit_len: usize) -> BigUint {
    let leading_bit = BigUint::one() << val_bit_len;
    leading_bit | val
}

#[cfg(test)]
#[path = "../../tests/tlb_adapters/dict/test_leading_bit_utils.rs"]
mod tests;
