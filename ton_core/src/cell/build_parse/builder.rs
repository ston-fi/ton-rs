use crate::bail_ton_core_data;
use crate::cell::meta::CellMeta;
use crate::cell::meta::CellType;
use crate::cell::ton_cell::{TonCell, TonCellRef, TonCellStorage};
use crate::cell::ton_cell_num::TonCellNum;
use crate::errors::TonCoreError;
use bitstream_io::{BigEndian, BitWrite, BitWriter};
use std::cmp::min;
use std::ops::Deref;

pub struct CellBuilder {
    cell_type: CellType,
    data_writer: BitWriter<Vec<u8>, BigEndian>,
    data_bits_len: usize,
    refs: TonCellStorage,
}

impl CellBuilder {
    pub fn new(cell_type: CellType) -> Self {
        Self {
            cell_type,
            data_writer: BitWriter::endian(vec![], BigEndian),
            data_bits_len: 0,
            refs: TonCellStorage::new(),
        }
    }

    pub fn build(self) -> Result<TonCell, TonCoreError> {
        let (data, data_bits_len) = build_cell_data(self.data_writer)?;
        let cell = TonCell {
            cell_type: self.cell_type,
            data,
            data_bits_len,
            refs: self.refs,
            meta: CellMeta::default(),
        };
        // Consider it as a tech debt. We have validation embedded into CellMetaBuilder,
        // and it's the simples way to use it from builder
        cell.meta.validate(&cell)?;
        Ok(cell)
    }

    pub fn build_ref(self) -> Result<TonCellRef, TonCoreError> { Ok(self.build()?.into_ref()) }

    pub fn write_bit(&mut self, data: bool) -> Result<(), TonCoreError> {
        self.ensure_capacity(1)?;
        self.data_writer.write_bit(data)?;
        Ok(())
    }

    /// expecting data.len() * 8 >= (bits_offset + bits_len)
    pub fn write_bits_with_offset<T: AsRef<[u8]>>(
        &mut self,
        data: T,
        mut bits_len: usize,
        mut bits_offset: usize,
    ) -> Result<(), TonCoreError> {
        self.ensure_capacity(bits_len)?;
        let mut data_ref = data.as_ref();

        if (bits_len + bits_offset).div_ceil(8) > data_ref.len() {
            bail_ton_core_data!(
                "Can't read {} bits from slice with only {} bytes",
                bits_len + bits_offset,
                data_ref.len()
            );
        }

        if bits_len == 0 {
            return Ok(());
        }

        // skip bytes_offset, adjust borders
        data_ref = &data_ref[bits_offset / 8..];
        bits_offset %= 8;

        let first_byte_bits_len = min(bits_len, 8 - bits_offset);
        let mut first_byte_val = data_ref[0] << bits_offset >> bits_offset;
        if first_byte_bits_len == bits_len {
            first_byte_val >>= 8 - bits_offset - bits_len
        }
        self.data_writer.write_var(first_byte_bits_len as u32, first_byte_val)?;

        data_ref = &data_ref[1..];
        bits_len -= first_byte_bits_len;

        let full_bytes = bits_len / 8;
        self.data_writer.write_bytes(&data_ref[0..full_bytes])?;
        let rest_bits_len = bits_len % 8;
        if rest_bits_len != 0 {
            self.data_writer.write_var(rest_bits_len as u32, data_ref[full_bytes] >> (8 - rest_bits_len))?;
        }
        Ok(())
    }

    pub fn write_bits<T: AsRef<[u8]>>(&mut self, data: T, bits_len: usize) -> Result<(), TonCoreError> {
        self.write_bits_with_offset(data, bits_len, 0)
    }

    pub fn write_cell(&mut self, cell: &TonCell) -> Result<(), TonCoreError> {
        self.write_bits(&cell.data, cell.data_bits_len)?;
        cell.refs.iter().cloned().try_for_each(|r| self.write_ref(r))
    }

    pub fn write_ref(&mut self, cell: TonCellRef) -> Result<(), TonCoreError> {
        if self.refs.len() >= TonCell::MAX_REFS_COUNT {
            bail_ton_core_data!("Can't add more refs: {} refs are written already", TonCell::MAX_REFS_COUNT);
        }
        self.refs.push(cell);
        Ok(())
    }

    pub fn write_num<N, D>(&mut self, data: D, bits_len: usize) -> Result<(), TonCoreError>
    where
        N: TonCellNum,
        D: Deref<Target = N>,
    {
        let data_ref = data.deref();
        // handling it like ton-core
        // https://github.com/ton-core/ton-core/blob/main/src/boc/BitBuilder.ts#L122
        if bits_len == 0 {
            if data_ref.tcn_is_zero() {
                return Ok(());
            }
            bail_ton_core_data!("Can't write number {data_ref} in 0 bits");
        }

        if let Some(unsigned) = data_ref.tcn_to_unsigned_primitive() {
            self.ensure_capacity(bits_len)?;
            self.data_writer.write_var(bits_len as u32, unsigned)?;
            return Ok(());
        }

        let min_bits_len = data_ref.tcn_min_bits_len();
        if min_bits_len > bits_len {
            bail_ton_core_data!("Can't write number {} ({} bits) in {} bits", data_ref, min_bits_len, bits_len);
        }

        let data_bytes = data_ref.tcn_to_bytes();
        let padding_val: u8 = match (N::SIGNED, data_bytes[0] >> 7 != 0) {
            (true, true) => 255,
            _ => 0,
        };
        let padding_bits_len = bits_len.saturating_sub(min_bits_len);
        let padding_to_write = vec![padding_val; padding_bits_len.div_ceil(8)];
        self.write_bits(padding_to_write, padding_bits_len)?;

        let bits_offset = (data_bytes.len() * 8).saturating_sub(min_bits_len);
        self.write_bits_with_offset(data_bytes, bits_len - padding_bits_len, bits_offset)
    }

    pub fn data_bits_left(&self) -> usize { TonCell::MAX_DATA_BITS_LEN - self.data_bits_len }

    fn ensure_capacity(&mut self, bits_len: usize) -> Result<(), TonCoreError> {
        let new_bits_len = self.data_bits_len + bits_len;
        if new_bits_len <= TonCell::MAX_DATA_BITS_LEN {
            self.data_bits_len = new_bits_len;
            return Ok(());
        }
        bail_ton_core_data!("Can't write {bits_len} bits: only {} free bits available", self.data_bits_left())
    }
}

fn build_cell_data(mut bit_writer: BitWriter<Vec<u8>, BigEndian>) -> Result<(Vec<u8>, usize), TonCoreError> {
    let mut trailing_zeros = 0;
    while !bit_writer.byte_aligned() {
        bit_writer.write_bit(false)?;
        trailing_zeros += 1;
    }
    let data = bit_writer.into_writer();
    let bits_len = data.len() * 8 - trailing_zeros;
    Ok((data, bits_len))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::meta::LevelMask;
    use crate::cell::TonHash;
    use num_bigint::BigUint;
    use num_traits::FromPrimitive;
    use std::str::FromStr;
    use tokio_test::{assert_err, assert_ok};

    #[test]
    fn test_builder_write_bit() -> anyhow::Result<()> {
        let mut cell_builder = TonCell::builder();
        cell_builder.write_bit(true)?;
        cell_builder.write_bit(false)?;
        cell_builder.write_bit(true)?;
        cell_builder.write_bit(false)?;
        let cell = cell_builder.build()?;
        assert_eq!(cell.data, vec![0b1010_0000]);
        assert_eq!(cell.data_bits_len, 4);
        Ok(())
    }

    #[test]
    fn test_builder_write_bits_with_offset() -> anyhow::Result<()> {
        let mut cell_builder = TonCell::builder();
        cell_builder.write_bits_with_offset([0b1010_1010], 8, 0)?;
        cell_builder.write_bits_with_offset([0b0000_1111], 4, 4)?;
        cell_builder.write_bits_with_offset([0b1111_0011], 3, 4)?;
        let cell = cell_builder.build()?;
        assert_eq!(cell.data, vec![0b1010_1010, 0b1111_0010]);
        assert_eq!(cell.data_bits_len, 15);

        let mut cell_builder = TonCell::builder();
        cell_builder.write_bits_with_offset([0b1010_1010, 0b0000_1111], 3, 10)?;
        let cell = cell_builder.build()?;
        assert_eq!(cell.data, vec![0b0010_0000]);
        assert_eq!(cell.data_bits_len, 3);
        Ok(())
    }

    #[test]
    fn test_builder_write_bits() -> anyhow::Result<()> {
        let mut cell_builder = TonCell::builder();
        cell_builder.write_bit(true)?;
        cell_builder.write_bits([0b1010_1010], 8)?;
        cell_builder.write_bits([0b0101_0101], 4)?;
        let cell = cell_builder.build()?;
        assert_eq!(cell.data, vec![0b1101_0101, 0b0010_1000]);
        assert_eq!(cell.data_bits_len, 13);
        Ok(())
    }

    #[test]
    fn test_builder_write_data_overflow() -> anyhow::Result<()> {
        let mut cell_builder = TonCell::builder();
        cell_builder.write_bit(true)?;
        assert!(cell_builder.write_bits([0b1010_1010], TonCell::MAX_DATA_BITS_LEN).is_err());
        let cell = cell_builder.build()?;
        assert_eq!(cell.data, vec![0b1000_0000]);
        Ok(())
    }

    #[test]
    fn test_builder_write_num_positive() -> anyhow::Result<()> {
        let mut cell_builder = TonCell::builder();
        cell_builder.write_num(&0b1010_1010, 8)?;
        cell_builder.write_num(&0b0000_0101, 4)?;
        let cell = cell_builder.build()?;
        assert_eq!(cell.data, vec![0b1010_1010, 0b0101_0000]);
        Ok(())
    }

    #[test]
    fn test_builder_write_num_in_many_bits() -> anyhow::Result<()> {
        let mut cell_builder = TonCell::builder();
        assert_err!(cell_builder.write_num(&0b1010_1010u8, 16));
        Ok(())
    }

    #[test]
    fn test_builder_write_num_positive_unaligned() -> anyhow::Result<()> {
        let mut cell_builder = TonCell::builder();
        cell_builder.write_num(&1u8, 4)?;
        cell_builder.write_num(&2u16, 5)?;
        cell_builder.write_num(&5u32, 10)?;
        let cell = cell_builder.build()?;
        assert_eq!(cell.data, vec![0b0001_0001, 0b0000_0000, 0b1010_0000]);
        assert_eq!(cell.data_bits_len, 19);
        Ok(())
    }

    #[test]
    fn test_builder_write_num_negative() -> anyhow::Result<()> {
        let mut cell_builder = TonCell::builder();
        assert!(cell_builder.write_num(&-3i32, 3).is_err());
        assert!(cell_builder.write_num(&-3i32, 31).is_err());
        cell_builder.write_num(&-3i16, 16)?;
        cell_builder.write_num(&-3i8, 8)?;
        let cell = cell_builder.build()?;
        assert_eq!(cell.data, vec![0b1111_1111, 0b1111_1101, 0b1111_1101]);
        Ok(())
    }

    #[test]
    fn test_builder_write_num_negative_unaligned() -> anyhow::Result<()> {
        let mut cell_builder = TonCell::builder();
        cell_builder.write_bit(false)?;
        cell_builder.write_num(&-3i16, 16)?;
        cell_builder.write_num(&-3i8, 8)?;
        let cell = cell_builder.build()?;
        assert_eq!(cell.data, vec![0b0111_1111, 0b1111_1110, 0b1111_1110, 0b1000_0000]);
        Ok(())
    }

    #[test]
    fn test_builder_write_cell() -> anyhow::Result<()> {
        let mut ref_builder = TonCell::builder();
        ref_builder.write_bit(true)?;
        ref_builder.write_bits([1, 2, 3], 24)?;
        let ref_cell = ref_builder.build()?.into_ref();

        let mut cell_with_ref_builder = TonCell::builder();
        cell_with_ref_builder.write_bit(true)?;
        cell_with_ref_builder.write_ref(ref_cell.clone())?;
        let cell_with_ref = cell_with_ref_builder.build()?;

        let mut cell_builder = TonCell::builder();
        cell_builder.write_cell(&cell_with_ref)?;
        let cell = cell_builder.build()?;

        assert_eq!(cell, cell_with_ref);
        Ok(())
    }

    #[test]
    fn test_builder_write_refs() -> anyhow::Result<()> {
        let mut builder = TonCell::builder();
        builder.write_bits([0b1111_0000], 4)?;
        let cell_ref = builder.build()?.into_ref();
        let mut cell_builder = TonCell::builder();
        cell_builder.write_ref(cell_ref.clone())?;
        cell_builder.write_ref(cell_ref.clone())?;
        let cell = cell_builder.build()?;
        assert_eq!(cell.refs.len(), 2);
        assert_eq!(cell.refs[0].data, cell_ref.data);
        assert_eq!(cell.refs[1].data, cell_ref.data);
        Ok(())
    }

    #[test]
    fn test_builder_build_cell_ordinary_empty() -> anyhow::Result<()> {
        let cell_builder = TonCell::builder();
        let cell = cell_builder.build()?;
        assert_eq!(cell, TonCell::EMPTY);
        for level in 0..4 {
            assert_eq!(cell.hash_for_level(LevelMask::new(level))?, &TonCell::EMPTY_CELL_HASH);
        }
        Ok(())
    }

    #[test]
    fn test_builder_build_cell_ordinary_non_empty() -> anyhow::Result<()> {
        //          0
        //        /   \
        //       1     2
        //      /
        //    3 4
        //   /
        //  5
        let mut builder5 = TonCell::builder();
        builder5.write_num(&0x05, 8)?;
        let cell5 = builder5.build()?;

        let mut builder3 = TonCell::builder();
        builder3.write_num(&0x03, 8)?;
        builder3.write_ref(cell5.clone().into_ref())?;
        let cell3 = builder3.build()?;

        let mut builder4 = TonCell::builder();
        builder4.write_num(&0x04, 8)?;
        let cell4 = builder4.build()?;

        let mut builder2 = TonCell::builder();
        builder2.write_num(&0x02, 8)?;
        let cell2 = builder2.build()?;

        let mut builder1 = TonCell::builder();
        builder1.write_num(&0x01, 8)?;
        builder1.write_ref(cell3.clone().into_ref())?;
        builder1.write_ref(cell4.clone().into_ref())?;
        let cell1 = builder1.build()?;

        let mut builder0 = TonCell::builder();
        builder0.write_bit(true)?;
        builder0.write_num(&0b0000_0001, 8)?;
        builder0.write_num(&0b0000_0011, 8)?;
        builder0.write_ref(cell1.clone().into_ref())?;
        builder0.write_ref(cell2.clone().into_ref())?;
        let cell0 = builder0.build()?;

        assert_eq!(cell0.refs.len(), 2);
        assert_eq!(cell0.data_bits_len, 17);
        assert_eq!(cell0.data, vec![0b1000_0000, 0b1000_0001, 0b1000_0000]);

        let exp_hash = TonHash::from_str("5d64a52c76eb32a63a393345a69533f095f945f2d30f371a1f323ac10102c395")?;
        for level in 0..4 {
            assert_eq!(cell0.hash_for_level(LevelMask::new(level))?, &exp_hash);
            assert_eq!(cell0.depth_for_level(LevelMask::new(level))?, 3);
        }
        Ok(())
    }

    #[test]
    fn test_builder_build_cell_library() -> anyhow::Result<()> {
        let mut builder = TonCell::builder_typed(CellType::LibraryRef);
        builder.write_bits(TonHash::ZERO, TonHash::BITS_LEN)?;
        assert_err!(builder.build()); // no lib prefix

        let mut builder = TonCell::builder_typed(CellType::LibraryRef);
        builder.write_num(&2, 8)?; // adding lib prefix https://docs.ton.org/v3/documentation/data-formats/tlb/exotic-cells#library-reference
        builder.write_bits(TonHash::ZERO, TonHash::BITS_LEN)?;
        let lib_cell = assert_ok!(builder.build());

        let expected_hash = TonHash::from_str("6f3fd5de541ec62d350d30785ada554a2b13b887a3e4e51896799d0b0c46c552")?;
        for level in 0..4 {
            assert_eq!(lib_cell.hash_for_level(LevelMask::new(level))?, &expected_hash);
            assert_eq!(lib_cell.depth_for_level(LevelMask::new(level))?, 0);
        }
        Ok(())
    }

    #[test]
    fn test_builder_write_bits_not_enough() -> anyhow::Result<()> {
        let mut builder = TonCell::builder();
        let data = vec![1u8; 2];
        assert_err!(builder.write_bits(&data, 32));
        Ok(())
    }

    #[test]
    fn test_builder_write_bits_not_enough_unaligned() -> anyhow::Result<()> {
        let mut builder = TonCell::builder();
        let data = vec![1u8; 2];
        assert_err!(builder.write_bits(&data, 33));
        Ok(())
    }

    #[test]
    fn test_builder_write_num_bigint() -> anyhow::Result<()> {
        let prepare_cell = |num_str: &str, bits_len: usize| {
            let number = num_bigint::BigInt::from_str(num_str)?;
            let mut builder = TonCell::builder();
            builder.write_bits([0], 7)?; // for pretty printing
            builder.write_num(&number, bits_len)?;
            let cell = builder.build()?;
            Ok::<_, anyhow::Error>(cell)
        };

        let cell = prepare_cell("3", 33)?;
        assert_eq!(cell.data, [0, 0, 0, 0, 3]);

        // 256 bits (+ sign)
        let cell = prepare_cell("97887266651548624282413032824435501549503168134499591480902563623927645013201", 257)?;
        assert_eq!(
            cell.data,
            [
                0, 216, 106, 58, 195, 97, 8, 173, 64, 195, 26, 52, 186, 72, 230, 253, 248, 12, 245, 147, 137, 170, 38,
                117, 66, 220, 74, 104, 103, 119, 137, 4, 209
            ]
        );

        let cell = prepare_cell("-5", 257)?;
        let mut expected = [0xFF; 33];
        expected[0] = 1;
        expected[32] = 251;
        assert_eq!(cell.data, expected);

        let cell = prepare_cell("-5", 33)?;
        assert_eq!(cell.data, [1, 0xFF, 0xFF, 0xFF, 251]);

        let cell = prepare_cell("-5", 4)?;
        assert_eq!(cell.data, [1, 96]);

        let cell = prepare_cell("-5", 5)?;
        assert_eq!(cell.data, [1, 176]);
        Ok(())
    }

    fn prepare_cell_big_uint(num_str: &str, bits_len: usize) -> anyhow::Result<TonCell> {
        let number = BigUint::from_str(num_str)?;
        let mut builder = TonCell::builder();
        builder.write_bits([0], 7)?; // for pretty printing
        builder.write_num(&number, bits_len)?;
        let cell = builder.build()?;
        Ok(cell)
    }

    #[test]
    fn test_builder_write_num_biguint() -> anyhow::Result<()> {
        let prepare_cell = |num_str: &str, bits_len: usize| {
            let number = num_bigint::BigUint::from_str(num_str)?;
            let mut builder = TonCell::builder();
            builder.write_bits([0], 7)?; // for pretty printing
            builder.write_num(&number, bits_len)?;
            let cell = builder.build()?;
            Ok::<_, anyhow::Error>(cell)
        };

        let cell = prepare_cell("3", 33)?;
        assert_eq!(cell.data, [0, 0, 0, 0, 3]);

        // 256 bits (+ sign)
        let cell = prepare_cell_big_uint(
            "97887266651548624282413032824435501549503168134499591480902563623927645013201",
            257,
        )?;
        assert_eq!(
            cell.data,
            [
                0, 216, 106, 58, 195, 97, 8, 173, 64, 195, 26, 52, 186, 72, 230, 253, 248, 12, 245, 147, 137, 170, 38,
                117, 66, 220, 74, 104, 103, 119, 137, 4, 209
            ]
        );

        let mut builder = TonCell::builder();
        builder.write_num(&BigUint::from_u64(117146891372).unwrap(), 257)?;
        let cell = builder.build()?;
        assert_eq!(
            cell.data,
            [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 13, 163, 63, 218, 54,
                0
            ]
        );
        Ok(())
    }

    #[test]
    fn test_builder_write_bignum_zero() -> anyhow::Result<()> {
        let number = num_bigint::BigInt::from_str("0")?;
        let mut builder = TonCell::builder();
        assert_ok!(builder.write_num(&number, 0));
        assert_ok!(builder.write_num(&number, 1));
        assert_ok!(builder.write_num(&number, 2));

        let number = num_bigint::BigUint::from_str("0")?;
        let mut builder = TonCell::builder();
        assert_ok!(builder.write_num(&number, 0));
        assert_ok!(builder.write_num(&number, 1));
        assert_ok!(builder.write_num(&number, 2));

        Ok(())
    }

    #[test]
    fn test_builder_write_bits_with_offset_proper_data_bits_len() -> anyhow::Result<()> {
        let mut builder = TonCell::builder();
        let data = vec![0b1010_1010, 0b0000_1111];
        builder.write_bits_with_offset(&data, 8, 0)?;
        assert_eq!(builder.data_bits_len, 8);
        builder.write_bits_with_offset(&data, 4, 4)?;
        assert_eq!(builder.data_bits_len, 12);
        builder.write_bits_with_offset(&data, 3, 4)?;
        assert_eq!(builder.data_bits_len, 15);
        Ok(())
    }

    #[test]
    fn test_builder_data_bits_left() -> anyhow::Result<()> {
        let mut builder = TonCell::builder();
        builder.write_bits([0b1010_1010], 8)?;
        assert_eq!(builder.data_bits_left(), TonCell::MAX_DATA_BITS_LEN - 8);
        builder.write_bits([0b0000_1111], 4)?;
        assert_eq!(builder.data_bits_left(), TonCell::MAX_DATA_BITS_LEN - 12);
        builder.write_num(&BigUint::from(1u32), 4)?;
        assert_eq!(builder.data_bits_left(), TonCell::MAX_DATA_BITS_LEN - 16);
        Ok(())
    }
}
