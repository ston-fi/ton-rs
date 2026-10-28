use crate::bail_ton_core_data;
use crate::cell::CellMeta;
use crate::cell::cell_meta::CellType;
use crate::cell::ton_cell::{CellBorders, CellData, RefStorage, TonCell};
use crate::cell::ton_cell_num::TonCellNum;
use crate::errors::{TonCoreError, TonCoreResult};
use bitstream_io::{BigEndian, BitWrite, BitWriter, Integer};
use std::any::type_name;
use std::cmp::min;
use std::fmt::Display;
use std::ops::Deref;
use std::sync::Arc;

pub(crate) const INITIAL_STORAGE_CAPACITY: usize = 1024;

pub struct CellBuilder {
    cell_type: CellType,
    data_writer: BitWriter<Vec<u8>, BigEndian>,
    data_len_bits: usize,
    refs: RefStorage,
}

impl CellBuilder {
    /// use TonCell::builder() for creation
    pub(super) fn new(cell_type: CellType, initial_capacity: usize) -> Self {
        let data_store = Vec::with_capacity(initial_capacity);
        Self {
            cell_type,
            data_writer: BitWriter::endian(data_store, BigEndian),
            data_len_bits: 0,
            refs: RefStorage::new(),
        }
    }

    pub fn build(self) -> Result<TonCell, TonCoreError> {
        let (mut cell_data, bits_len) = build_cell_data(self.data_writer)?;
        cell_data.refs = self.refs;

        let borders = CellBorders {
            start_bit: 0,
            end_bit: bits_len,
            start_ref: 0,
            end_ref: cell_data.refs.len() as u8,
        };

        let cell = TonCell {
            cell_type: self.cell_type,
            cell_data: Arc::new(cell_data),
            borders,
            meta: Arc::new(CellMeta::default()),
        };
        // Consider it as a tech debt. We have validation embedded into CellMetaBuilder,
        // and it's the simples way to use it from builder
        cell.meta.validate(&cell)?;
        Ok(cell)
    }

    pub fn write_bit(&mut self, data: bool) -> Result<(), TonCoreError> {
        self.ensure_capacity(1)?;
        self.data_writer.write_bit(data)?;
        Ok(())
    }

    /// expecting data.len() * 8 >= (bits_offset + bits_len)
    pub fn write_bits_with_offset<T: AsRef<[u8]>>(
        &mut self,
        data: T,
        mut bits_offset: usize,
        mut bits_len: usize,
    ) -> Result<(), TonCoreError> {
        if bits_len == 0 {
            return Ok(());
        }

        self.ensure_capacity(bits_len)?;
        let mut data_ref = data.as_ref();

        if (bits_len + bits_offset).div_ceil(8) > data_ref.len() {
            bail_ton_core_data!(
                "Can't read {} bits from slice with only {} bytes",
                bits_len + bits_offset,
                data_ref.len()
            );
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
        let remaining_len_bits = bits_len % 8;
        if remaining_len_bits != 0 {
            self.data_writer.write_var(remaining_len_bits as u32, data_ref[full_bytes] >> (8 - remaining_len_bits))?;
        }
        Ok(())
    }

    pub fn write_bits<T: AsRef<[u8]>>(&mut self, data: T, bits_len: usize) -> Result<(), TonCoreError> {
        self.write_bits_with_offset(data, 0, bits_len)
    }

    pub fn write_cell(&mut self, cell: &TonCell) -> Result<(), TonCoreError> {
        let mut parser = cell.parser();
        let data_bits_len = parser.data_bits_left()?;
        let data = parser.read_bits(data_bits_len)?;
        self.write_bits(&data, data_bits_len)?;
        for cell_ref in cell.refs() {
            self.write_ref(cell_ref.to_owned())?;
        }
        Ok(())
    }

    pub fn write_ref<T: Into<TonCell>>(&mut self, cell: T) -> Result<(), TonCoreError> {
        if self.refs.len() >= TonCell::MAX_REFS_COUNT {
            bail_ton_core_data!("Can't add more refs: {} refs are written already", TonCell::MAX_REFS_COUNT);
        }
        self.refs.push(cell.into());
        Ok(())
    }

    pub fn write_num<N: TonCellNum>(
        &mut self,
        data: impl Deref<Target = N>,
        bits_len: usize,
    ) -> Result<(), TonCoreError> {
        if bits_len == 0 {
            // handling it like ton-core: https://github.com/ton-core/ton-core/blob/main/src/boc/BitBuilder.ts#L122
            if data.is_zero() {
                return Ok(());
            }
            bail_ton_core_data!("Can't write number {} in 0 bits", data.deref());
        }
        if data.tcn_min_bits_len() > bits_len {
            bail_ton_core_data!(
                "Can't write num {} in {bits_len} bits, min_bits_len {}",
                data.deref(),
                data.tcn_min_bits_len()
            );
        }
        data.tcn_write_bits(self, bits_len)
    }

    pub fn data_bits_left(&self) -> usize {
        TonCell::MAX_DATA_LEN_BITS - self.data_len_bits
    }

    pub fn refs_left(&self) -> usize {
        TonCell::MAX_REFS_COUNT - self.refs.len()
    }

    pub fn set_type(&mut self, cell_type: CellType) {
        self.cell_type = cell_type;
    }

    #[inline(always)]
    fn ensure_capacity(&mut self, bits_len: usize) -> Result<(), TonCoreError> {
        let new_bits_len = self.data_len_bits + bits_len;
        if new_bits_len <= TonCell::MAX_DATA_LEN_BITS {
            self.data_len_bits = new_bits_len;
            return Ok(());
        }
        bail_ton_core_data!("Can't write {bits_len} bits: only {} free bits available", self.data_bits_left())
    }

    // bitstream can't read/write signed properly: https://github.com/tuffy/bitstream-io/issues/26
    #[inline(always)]
    pub(crate) fn write_unsigned_primitive<T: Integer + Display + Copy>(
        &mut self,
        data: T,
        bits_len: usize,
    ) -> TonCoreResult<()> {
        self.ensure_capacity(bits_len)?;
        if let Err(err) = self.data_writer.write_var(bits_len as u32, data) {
            bail_ton_core_data!("Failed to write {data} ({}) in {bits_len} bits: {err}", type_name::<T>());
        }
        Ok(())
    }
}

fn build_cell_data(mut bit_writer: BitWriter<Vec<u8>, BigEndian>) -> TonCoreResult<(CellData, usize)> {
    let mut trailing_zeros = 0;
    while !bit_writer.byte_aligned() {
        bit_writer.write_bit(false)?;
        trailing_zeros += 1;
    }
    let data = bit_writer.into_writer();
    let bits_len = data.len() * 8 - trailing_zeros;
    let cell_data = CellData {
        data_storage: Arc::new(data),
        refs: Default::default(),
    };
    Ok((cell_data, bits_len))
}

#[cfg(test)]
#[path = "../tests/cell/test_cell_builder.rs"]
mod tests;
