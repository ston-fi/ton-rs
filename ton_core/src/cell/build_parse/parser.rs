use crate::bail_ton_core_data;
use crate::cell::ton_cell::{TonCell, TonCellRef};
use crate::cell::ton_cell_num::TonCellNum;
use crate::errors::TonCoreError;
use bitstream_io::{BigEndian, BitRead, BitReader};
use num_traits::Zero;
use std::io::{Cursor, SeekFrom};

#[derive(Debug)]
pub struct CellParser<'a> {
    pub cell: &'a TonCell,
    pub next_ref_pos: usize,
    data_reader: BitReader<Cursor<&'a [u8]>, BigEndian>,
}

impl<'a> CellParser<'a> {
    pub fn new(cell: &'a TonCell) -> Self {
        let cursor = Cursor::new(cell.data.as_slice());
        let data_reader = BitReader::endian(cursor, BigEndian);

        Self {
            cell,
            data_reader,
            next_ref_pos: 0,
        }
    }

    pub fn lookup_bits(&mut self, bits_len: usize) -> Result<u128, TonCoreError> {
        let value = self.read_num(bits_len)?;
        self.seek_bits(-(bits_len as i32))?;
        Ok(value)
    }

    pub fn read_bit(&mut self) -> Result<bool, TonCoreError> {
        self.ensure_enough_bits(1)?;
        Ok(self.data_reader.read_bit()?)
    }

    pub fn read_bits(&mut self, bits_len: usize) -> Result<Vec<u8>, TonCoreError> {
        self.ensure_enough_bits(bits_len)?;
        let mut dst = vec![0; bits_len.div_ceil(8)];
        let full_bytes = bits_len / 8;
        let remaining_bits = bits_len % 8;

        self.data_reader.read_bytes(&mut dst[..full_bytes])?;

        if remaining_bits != 0 {
            let last_byte = self.data_reader.read_var::<u8>(remaining_bits as u32)?;
            dst[full_bytes] = last_byte << (8 - remaining_bits);
        }
        Ok(dst)
    }

    pub fn read_num<N: TonCellNum>(&mut self, bits_len: usize) -> Result<N, TonCoreError> {
        if bits_len == 0 {
            return Ok(N::tcn_from_primitive(N::Primitive::zero()));
        }
        self.ensure_enough_bits(bits_len)?;
        if N::IS_PRIMITIVE {
            let primitive = self.data_reader.read_var::<N::Primitive>(bits_len as u32)?;
            return Ok(N::tcn_from_primitive(primitive));
        }
        let bytes = self.read_bits(bits_len)?;
        let res = N::tcn_from_bytes(&bytes);
        if bits_len % 8 != 0 {
            return Ok(res.tcn_shr(8 - bits_len % 8));
        }
        Ok(res)
    }

    pub fn read_cell(&mut self) -> Result<TonCell, TonCoreError> {
        let bits_left = self.data_bits_remaining()?;
        let data = self.read_bits(bits_left)?;

        let mut builder = TonCell::builder();
        builder.write_bits(data, bits_left)?;

        while let Ok(cell_ref) = self.read_next_ref() {
            builder.write_ref(cell_ref.clone())?;
        }
        builder.build()
    }

    /// Ranges are handled like [start; end) - the same as iterators in c++
    pub fn read_cell_slice(
        &mut self,
        start_bit: usize,
        end_bit: usize,
        start_ref: usize,
        end_ref: usize,
    ) -> Result<TonCell, TonCoreError> {
        if self.data_reader.position_in_bits()? != 0 && self.next_ref_pos != 0 {
            bail_ton_core_data!(
                "CellParser must be at the beginning of the cell to read a slice: bit_pos {}, next_ref_pos {}",
                self.data_reader.position_in_bits()? as usize,
                self.next_ref_pos
            );
        }
        self.read_bits(start_bit)?; // skip
        let slice_data_bits_len = end_bit - start_bit;
        let mut builder = TonCell::builder();
        builder.write_bits(self.read_bits(slice_data_bits_len)?, slice_data_bits_len)?;
        for _ in 0..start_ref {
            self.read_next_ref()?; // skip
        }
        for _ in start_ref..end_ref {
            builder.write_ref(self.read_next_ref()?.clone())?;
        }
        builder.build()
    }

    pub fn read_next_ref(&mut self) -> Result<&TonCellRef, TonCoreError> {
        if self.next_ref_pos == self.cell.refs.len() {
            bail_ton_core_data!(
                "No more refs in cell: next_ref_pos {}, refs_len {}",
                self.next_ref_pos,
                self.cell.refs.len()
            );
        }
        let cell_ref = &self.cell.refs[self.next_ref_pos];
        self.next_ref_pos += 1;
        Ok(cell_ref)
    }

    pub fn data_bits_remaining(&mut self) -> Result<usize, TonCoreError> {
        let reader_pos = self.data_reader.position_in_bits()? as usize;
        Ok(self.cell.data_bits_len - reader_pos)
    }

    pub fn seek_bits(&mut self, offset: i32) -> Result<(), TonCoreError> {
        let new_pos = self.data_reader.position_in_bits()? as i32 + offset;
        let data_bits_len = self.cell.data_bits_len;
        if new_pos < 0 || new_pos as usize > (data_bits_len - 1) {
            bail_ton_core_data!("Bad seek position in cell: new_pos {new_pos}, data_bits_len {data_bits_len}");
        }
        self.data_reader.seek_bits(SeekFrom::Current(offset as i64))?;
        Ok(())
    }

    pub fn ensure_empty(&mut self) -> Result<(), TonCoreError> {
        let bits_left = self.data_bits_remaining()?;
        let refs_left = self.cell.refs.len() - self.next_ref_pos;
        if bits_left == 0 && refs_left == 0 {
            return Ok(());
        }
        bail_ton_core_data!("Cell is not empty: {bits_left} bits left, {refs_left} refs left");
    }

    // returns remaining bits
    fn ensure_enough_bits(&mut self, bit_len: usize) -> Result<usize, TonCoreError> {
        let bits_left = self.data_bits_remaining()?;

        if bit_len <= bits_left {
            return Ok(bits_left);
        }
        bail_ton_core_data!("Not enough bits in cell: required {bit_len}, left {bits_left}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::{BigInt, BigUint};
    use tokio_test::{assert_err, assert_ok};

    fn make_test_cell(data: &[u8], bits_len: usize) -> anyhow::Result<TonCell> {
        let mut builder = TonCell::builder();
        builder.write_bits(data, bits_len)?;
        Ok(builder.build()?)
    }

    #[test]
    fn test_parser_seek_bits() -> anyhow::Result<()> {
        let cell_slice = make_test_cell(&[0b10101001, 0b01010100], 10)?;
        let mut parser = CellParser::new(&cell_slice);
        assert_ok!(parser.seek_bits(3));
        assert_eq!(parser.data_reader.position_in_bits()? as usize, 3);
        assert_ok!(parser.seek_bits(-2));
        assert_eq!(parser.data_reader.position_in_bits()? as usize, 1);
        assert_ok!(parser.seek_bits(0));
        assert_eq!(parser.data_reader.position_in_bits()? as usize, 1);
        assert_ok!(parser.seek_bits(-1));
        assert_eq!(parser.data_reader.position_in_bits()? as usize, 0);
        assert_err!(parser.seek_bits(-1));
        assert_eq!(parser.data_reader.position_in_bits()? as usize, 0);
        assert_ok!(parser.seek_bits(cell_slice.data_bits_len as i32 - 1));
        assert_eq!(parser.data_reader.position_in_bits()? as usize, cell_slice.data_bits_len - 1);
        assert_err!(parser.seek_bits(1));
        assert_eq!(parser.data_reader.position_in_bits()? as usize, cell_slice.data_bits_len - 1);

        assert_err!(parser.seek_bits(20));
        Ok(())
    }

    #[test]
    fn test_parser_lookup_bits() -> anyhow::Result<()> {
        let cell_slice = make_test_cell(&[0b10101010, 0b01010101], 16)?;
        let mut parser = CellParser::new(&cell_slice);
        assert_eq!(parser.lookup_bits(3)?, 0b101);
        assert_eq!(parser.data_reader.position_in_bits()?, 0);
        assert!(assert_ok!(parser.read_bit()));
        assert_eq!(parser.data_reader.position_in_bits()?, 1);
        assert_eq!(parser.lookup_bits(3)?, 0b010);
        assert_eq!(parser.data_reader.position_in_bits()?, 1);
        Ok(())
    }

    #[test]
    fn test_parser_read_bit() -> anyhow::Result<()> {
        let cell_slice = make_test_cell(&[0b10101010, 0b01010101], 16)?;
        let mut parser = CellParser::new(&cell_slice);
        for i in 0..8 {
            assert_eq!(assert_ok!(parser.read_bit()), i % 2 == 0);
        }
        for i in 0..8 {
            assert_eq!(assert_ok!(parser.read_bit()), i % 2 != 0);
        }
        Ok(())
    }

    #[test]
    fn test_parser_ensure_enough_bits() -> anyhow::Result<()> {
        let cell_slice = make_test_cell(&[0b10101010, 0b01010101], 10)?;
        let mut parser = CellParser::new(&cell_slice);
        assert_eq!(parser.data_reader.position_in_bits()?, 0);
        assert_ok!(parser.ensure_enough_bits(0));
        assert_ok!(parser.ensure_enough_bits(1));
        assert_ok!(parser.ensure_enough_bits(6));
        assert_ok!(parser.ensure_enough_bits(10));
        assert_err!(parser.ensure_enough_bits(11));
        Ok(())
    }

    #[test]
    fn test_parser_read_ref() -> anyhow::Result<()> {
        let mut ref_builder = TonCell::builder();
        ref_builder.write_num(&0b11110000, 8)?;
        let cell_ref = ref_builder.build()?.into_ref();

        let mut cell_builder = TonCell::builder();
        cell_builder.write_ref(cell_ref.clone())?;
        cell_builder.write_ref(cell_ref.clone())?;
        let cell = cell_builder.build()?.into_ref();

        let mut parser = CellParser::new(&cell);
        assert_eq!(parser.read_next_ref()?.data, cell_ref.data);
        assert_eq!(parser.read_next_ref()?.data, cell_ref.data);
        assert!(parser.read_next_ref().is_err());
        Ok(())
    }

    #[test]
    fn test_parser_read_bits() -> anyhow::Result<()> {
        let cell_slice = make_test_cell(&[0b10101010, 0b01010101], 16)?;
        let mut parser = CellParser::new(&cell_slice);
        let dst = parser.read_bits(3)?;
        assert_eq!(dst, [0b10100000]);
        let dst = parser.read_bits(6)?;
        assert_eq!(dst, [0b01010000]);
        Ok(())
    }

    #[test]
    fn test_parser_read_num() -> anyhow::Result<()> {
        let cell_slice = make_test_cell(&[0b10101010, 0b01010101], 16)?;
        let mut parser = CellParser::new(&cell_slice);
        assert_eq!(parser.read_num::<u8>(3)?, 0b101);
        assert_eq!(parser.data_reader.position_in_bits()?, 3);
        assert_eq!(parser.read_num::<u32>(3)?, 0b010);
        assert_eq!(parser.data_reader.position_in_bits()?, 6);
        assert_eq!(parser.read_num::<u64>(3)?, 0b100);
        assert_eq!(parser.data_reader.position_in_bits()?, 9);
        Ok(())
    }

    #[test]
    fn test_parser_read_num_unaligned() -> anyhow::Result<()> {
        let cell_slice = make_test_cell(&[0b0001_0001, 0b0000_0000, 0b1010_0000], 19)?;
        let mut parser = CellParser::new(&cell_slice);
        assert_eq!(parser.read_num::<u8>(4)?, 1);
        assert_eq!(parser.data_reader.position_in_bits()?, 4);
        assert_eq!(parser.read_num::<u16>(5)?, 2);
        assert_eq!(parser.data_reader.position_in_bits()?, 9);
        assert_eq!(parser.read_num::<u32>(10)?, 5);
        assert_eq!(parser.data_reader.position_in_bits()?, 19);
        Ok(())
    }

    #[test]
    fn test_parser_read_cell() -> anyhow::Result<()> {
        let data = [0b0001_0001, 0b0000_0000, 0b1010_0000];
        let cell = make_test_cell(&data, 19)?;
        assert_eq!(CellParser::new(&cell).read_cell()?, cell);
        Ok(())
    }

    #[test]
    fn test_builder_write_cell_slice() -> anyhow::Result<()> {
        let mut builder = TonCell::builder();
        builder.write_bits([255, 0, 255, 0], 24)?;

        for i in 0..3 {
            let mut ref_builder = TonCell::builder();
            ref_builder.write_bits([i], 8)?;
            builder.write_ref(ref_builder.build()?.into_ref())?;
        }

        let orig_cell = builder.build()?;
        let cell_slice = orig_cell.parser().read_cell_slice(4, 20, 1, 3)?;

        assert_eq!(cell_slice.data, vec![0b1111_0000, 0b0000_1111]);
        assert_eq!(cell_slice.data_bits_len, 16);
        assert_eq!(cell_slice.refs.len(), 2);
        assert_eq!(cell_slice.refs[0], orig_cell.refs[1]);
        assert_eq!(cell_slice.refs[1], orig_cell.refs[2]);
        Ok(())
    }

    #[test]
    fn test_parser_read_bigint() -> anyhow::Result<()> {
        let cell_slice = make_test_cell(&[0b111_01010, 0b01101011, 0b10000000, 0b00000001], 32)?;
        let mut parser = CellParser::new(&cell_slice);
        assert_eq!(parser.read_num::<BigInt>(3)?, (-1).into());
        assert_eq!(parser.data_reader.position_in_bits()?, 3);
        assert_eq!(parser.read_num::<BigInt>(5)?, 10.into()); // finish with first byte
        assert_eq!(parser.data_reader.position_in_bits()?, 8);
        parser.read_bit()?; // skip 1 bit
        assert_eq!(parser.read_num::<BigInt>(7)?, (-21).into()); // finish with second byte
        assert_eq!(parser.data_reader.position_in_bits()?, 16);
        assert_eq!(parser.read_num::<BigInt>(16)?, (-32767).into());
        Ok(())
    }

    #[test]
    fn test_parser_read_bigint_unaligned() -> anyhow::Result<()> {
        let cell_slice = make_test_cell(&[0b00011110, 0b11111111], 16)?;
        let mut parser = CellParser::new(&cell_slice);
        parser.seek_bits(3)?;
        assert_eq!(parser.read_num::<BigInt>(9)?, (-17).into());
        Ok(())
    }

    #[test]
    fn test_parser_read_biguint() -> anyhow::Result<()> {
        let cell_slice = make_test_cell(&[0b10101010, 0b01010101, 0b11111111, 0b11111111], 32)?;
        let mut parser = CellParser::new(&cell_slice);
        assert_eq!(parser.read_num::<BigUint>(3)?, 5u32.into());
        assert_eq!(parser.data_reader.position_in_bits()?, 3);
        assert_eq!(parser.read_num::<BigUint>(5)?, 10u32.into()); // finish with first byte
        assert_eq!(parser.data_reader.position_in_bits()?, 8);
        parser.read_bit()?; // skip 1 bit
        assert_eq!(parser.read_num::<BigUint>(7)?, 85u32.into()); // finish with second byte
        assert_eq!(parser.data_reader.position_in_bits()?, 16);
        assert_eq!(parser.read_num::<BigUint>(16)?, 65535u32.into());
        Ok(())
    }

    #[test]
    fn test_parser_ensure_empty() -> anyhow::Result<()> {
        let cell_ref = make_test_cell(&[0b10101010, 0b01010101], 16)?;
        let mut builder = TonCell::builder();
        builder.write_ref(cell_ref.into_ref())?;
        builder.write_num(&3, 3)?;
        let cell = builder.build()?;

        let mut parser = CellParser::new(&cell);
        assert_err!(parser.ensure_empty());
        parser.read_bits(3)?;
        assert_err!(parser.ensure_empty());
        parser.read_next_ref()?;
        assert_ok!(parser.ensure_empty());
        Ok(())
    }
}
