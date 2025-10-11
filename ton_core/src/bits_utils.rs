// return false if left or right is too short for bits_len

use bitvec::prelude::Msb0;
use bitvec::view::BitView;

pub struct BitsUtils;

impl BitsUtils {
    pub fn equal(left: &[u8], right: &[u8], bits_len: usize) -> bool {
        let bytes_len = bits_len.div_ceil(8);
        if left.len() < bytes_len || right.len() < bytes_len {
            return false;
        }
        let rest = bits_len % 8;
        if rest == 0 {
            return left[0..bytes_len] == right[0..bytes_len];
        }

        let left_bits = &left[0..bytes_len - 1];
        let right_bits = &right[0..bytes_len - 1];
        if left_bits != right_bits {
            return false;
        }
        let left_last_byte = left[bytes_len - 1] >> (8 - bits_len % 8);
        let right_last_byte = right[bytes_len - 1] >> (8 - bits_len % 8);
        left_last_byte == right_last_byte
    }

    // return false if preconditions are not met
    pub fn rewrite(
        src: &[u8],
        src_offset_bits: usize,
        dst: &mut [u8],
        dst_offset_bits: usize,
        len_bits: usize,
    ) -> bool {
        // Calculate total bits available in source and destination
        let src_total_bits = src.len() * 8;
        let dst_total_bits = dst.len() * 8;

        // Check preconditions
        if src_offset_bits + len_bits > src_total_bits || dst_offset_bits + len_bits > dst_total_bits {
            return false;
        }

        for i in 0..len_bits {
            // Calculate the source bit position and extract the bits
            let src_bit_pos = src_offset_bits + i;
            let src_byte_index = src_bit_pos / 8;
            let src_bit_offset = 7 - (src_bit_pos % 8); // MSB is bit 7
            let src_bit = (src[src_byte_index] >> src_bit_offset) & 1;

            // Calculate the destination bit position and write the bit
            let dst_bit_pos = dst_offset_bits + i;
            let dst_byte_index = dst_bit_pos / 8;
            let dst_bit_offset = 7 - (dst_bit_pos % 8); // MSB is bit 7

            // Clear the target bit and set it to the source bit value
            dst[dst_byte_index] &= !(1 << dst_bit_offset); // Clear the bit
            dst[dst_byte_index] |= src_bit << dst_bit_offset; // Set the bit
        }

        true
    }

    // return false if preconditions are not met
    // if dst is larger than len_bits, value of the rest bits is undefined
    pub fn read_with_offset(src: &[u8], dst: &mut [u8], offset_bits: usize, len_bits: usize) -> bool {
        // Calculate total bits available in source and destination
        let src_total_bits = src.len() * 8;
        let dst_total_bits = dst.len() * 8;

        // Check preconditions
        if offset_bits > src_total_bits || offset_bits + len_bits > src_total_bits || len_bits > dst_total_bits {
            return false;
        }

        let src_bits = &src.view_bits::<Msb0>()[offset_bits..offset_bits + len_bits];
        let dst_bits = &mut dst.view_bits_mut::<Msb0>()[..len_bits];
        dst_bits.copy_from_bitslice(src_bits);
        true
    }
}

#[cfg(test)]
mod tests {
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
}
