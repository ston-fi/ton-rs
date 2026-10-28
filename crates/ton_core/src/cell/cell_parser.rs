use crate::bail_ton_core_data;
use crate::cell::TonCell;
use crate::cell::ton_cell::{CellBitsReader, CellBorders};
use crate::cell::ton_cell_num::TonCellNum;
use crate::errors::TonCoreError;
use crate::errors::TonCoreResult;
use bitstream_io::Integer;
use bitstream_io::{BigEndian, BitRead, BitReader};
use std::any::type_name;
use std::io::{Cursor, SeekFrom};

#[derive(Debug, Clone)]
pub struct CellParser<'a> {
    cell: &'a TonCell,
    data_reader: CellBitsReader<'a>,
    next_ref_pos: usize,
}

impl<'a> CellParser<'a> {
    /// use cell.parser() for creation
    pub(super) fn new(cell: &'a TonCell) -> Self {
        let cursor = Cursor::new(cell.cell_data.data_storage.as_slice());
        let mut data_reader = BitReader::endian(cursor, BigEndian);
        // unwrap is safe while borders are validated (we must support this invariant)
        data_reader.seek_bits(SeekFrom::Current(cell.borders.start_bit as i64)).unwrap();
        let next_ref_pos = cell.borders.start_ref as usize;
        Self {
            cell,
            data_reader,
            next_ref_pos,
        }
    }

    pub fn original_cell(&self) -> &'a TonCell {
        self.cell
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
        let mut dst = vec![0; bits_len.div_ceil(8)];
        self.read_bits_to(bits_len, &mut dst)?;
        Ok(dst)
    }

    pub fn read_bits_to(&mut self, bits_len: usize, dst: &mut [u8]) -> Result<(), TonCoreError> {
        if dst.len() * 8 < bits_len {
            bail_ton_core_data!("Can't write {bits_len} bits into {}-bytes buffer", dst.len());
        }
        self.ensure_enough_bits(bits_len)?;
        let full_bytes = bits_len / 8;
        let remaining_bits = bits_len % 8;

        self.data_reader.read_bytes(&mut dst[..full_bytes])?;

        if remaining_bits != 0 {
            let last_byte = self.data_reader.read_var::<u8>(remaining_bits as u32)?;
            dst[full_bytes] = last_byte << (8 - remaining_bits);
        }
        Ok(())
    }

    pub fn read_num<N: TonCellNum>(&mut self, bits_len: usize) -> Result<N, TonCoreError> {
        if bits_len == 0 {
            return Ok(N::zero());
        }
        self.ensure_enough_bits(bits_len)?;
        N::tcn_read_bits(self, bits_len)
    }

    pub fn read_cell(&mut self, bits_len: usize, refs_len: u8) -> Result<TonCell, TonCoreError> {
        let start_bit = self.data_reader.position_in_bits()? as usize - self.cell.borders.start_bit;
        let end_bit = start_bit + bits_len;
        let start_ref = self.next_ref_pos as u8 - self.cell.borders.start_ref;
        let end_ref = start_ref + refs_len;
        let borders = CellBorders {
            start_bit,
            end_bit,
            start_ref,
            end_ref,
        };
        let slice = self.cell.slice(borders)?; // validation will be done in .slice()
        self.seek_bits(bits_len as i32)?;
        self.next_ref_pos += refs_len as usize;
        Ok(slice)
    }

    pub fn read_remaining(&mut self) -> Result<TonCell, TonCoreError> {
        let bits_len = self.data_bits_left()?;
        let refs_len = self.refs_left();
        self.read_cell(bits_len, refs_len as u8)
    }

    pub fn read_next_ref(&mut self) -> Result<&TonCell, TonCoreError> {
        if self.next_ref_pos == self.cell.borders.end_ref as usize {
            bail_ton_core_data!(
                "No more refs in cell: next_ref_pos={}, end_ref_pos={}",
                self.next_ref_pos,
                self.cell.borders.end_ref,
            );
        }
        let cell_ref = &self.cell.cell_data.refs[self.next_ref_pos];
        self.next_ref_pos += 1;
        Ok(cell_ref)
    }

    pub fn data_bits_left(&mut self) -> Result<usize, TonCoreError> {
        let reader_pos = self.data_reader.position_in_bits()? as usize;
        Ok(self.cell.borders.end_bit - reader_pos)
    }

    pub fn refs_left(&mut self) -> usize {
        self.cell.borders.end_ref as usize - self.next_ref_pos
    }

    pub fn seek_bits(&mut self, offset: i32) -> Result<(), TonCoreError> {
        let new_pos = self.data_reader.position_in_bits()? as i32 + offset;
        let new_pos_unsigned = if new_pos < 0 {
            bail_ton_core_data!(
                "Bad seek position in slice: new_pos {new_pos}, start_bit {}, end_bit {}",
                self.cell.borders.start_bit,
                self.cell.borders.end_bit
            );
        } else {
            new_pos as usize
        };
        if new_pos_unsigned < self.cell.borders.start_bit || new_pos_unsigned > self.cell.borders.end_bit {
            bail_ton_core_data!(
                "Bad seek position in slice: new_pos {new_pos}, start_bit {}, end_bit {}",
                self.cell.borders.start_bit,
                self.cell.borders.end_bit
            );
        }
        self.data_reader.seek_bits(SeekFrom::Current(offset as i64))?;
        Ok(())
    }

    pub fn ensure_empty(&mut self) -> Result<(), TonCoreError> {
        let bits_left = self.data_bits_left()?;
        let refs_left = self.cell.borders.end_ref as usize - self.next_ref_pos;
        if bits_left == 0 && refs_left == 0 {
            return Ok(());
        }
        bail_ton_core_data!("Cell is not empty: {bits_left} bits left, {refs_left} refs left");
    }

    // returns remaining bits
    fn ensure_enough_bits(&mut self, bit_len: usize) -> Result<usize, TonCoreError> {
        let bits_left = self.data_bits_left()?;

        if bit_len <= bits_left {
            return Ok(bits_left);
        }
        bail_ton_core_data!("Not enough bits in cell: required {bit_len}, left {bits_left}");
    }

    // bitstream can't read/write signed properly: https://github.com/tuffy/bitstream-io/issues/26
    #[inline(always)]
    pub(crate) fn read_unsigned_primitive<I: Integer>(&mut self, bits_len: usize) -> TonCoreResult<I> {
        match self.data_reader.read_var(bits_len as u32) {
            Ok(value) => Ok(value),
            Err(err) => bail_ton_core_data!("Failed to read {} in {bits_len} bits: {err}", type_name::<I>()),
        }
    }

    pub fn get_position(&mut self) -> Result<ParserPosition, TonCoreError> {
        Ok(ParserPosition {
            bits_offset: self.data_reader.position_in_bits()?,
            next_ref_pos: self.next_ref_pos,
        })
    }

    pub fn set_position(&mut self, position: ParserPosition) -> Result<(), TonCoreError> {
        let cur_position = self.data_reader.position_in_bits()?;
        let offset = position.bits_offset as i32 - cur_position as i32;
        self.seek_bits(offset)?;

        if position.next_ref_pos < self.cell.borders.start_ref as usize
            || position.next_ref_pos > self.cell.borders.end_ref as usize
        {
            bail_ton_core_data!(
                "Bad seek position in slice: next_ref_pos {}, start_ref {}, end_ref {}",
                position.next_ref_pos,
                self.cell.borders.start_ref,
                self.cell.borders.end_ref
            );
        }
        self.next_ref_pos = position.next_ref_pos;
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub struct ParserPosition {
    pub bits_offset: u64,
    pub next_ref_pos: usize,
}

#[cfg(test)]
#[path = "../tests/cell/test_cell_parser.rs"]
mod tests;
