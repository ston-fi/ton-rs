use crate::errors::TonCoreError;
use bitstream_io::{BigEndian, BitWrite, BitWriter};
use crc::Crc;

use super::{RawBoC, RawCell, GENERIC_BOC_MAGIC};

const CRC_32_ISCSI: Crc<u32> = Crc::<u32>::new(&crc::CRC_32_ISCSI);

impl RawBoC {
    //Based on https://github.com/toncenter/tonweb/blob/c2d5d0fc23d2aec55a0412940ce6e580344a288c/src/boc/Cell.js#L198
    pub fn to_bytes(&self, has_crc32: bool) -> Result<Vec<u8>, TonCoreError> {
        let root_count = self.roots_position.len();
        let num_ref_bits = 32 - (self.cells.len() as u32).leading_zeros();
        let num_ref_bytes = num_ref_bits.div_ceil(8);
        let has_idx = false;

        let mut full_size = 0u32;

        for cell in &self.cells {
            full_size += raw_cell_size(cell, num_ref_bytes);
        }

        let num_offset_bits = 32 - full_size.leading_zeros();
        let num_offset_bytes = num_offset_bits.div_ceil(8);

        let total_size = 4 + // magic
            1 + // flags and s_bytes
            1 + // offset_bytes
            3 * num_ref_bytes + // cells_num, roots, complete
            num_offset_bytes + // full_size
            num_ref_bytes + // root_idx
            (if has_idx { self.cells.len() as u32 * num_offset_bytes } else { 0 }) +
            full_size +
            (if has_crc32 { 4 } else { 0 });

        let mut writer = BitWriter::endian(Vec::with_capacity(total_size as usize), BigEndian);
        writer.write_var(32, GENERIC_BOC_MAGIC)?;
        writer.write_bit(has_idx)?;
        writer.write_bit(has_crc32)?;
        writer.write_bit(false)?; // has_cache_bits
        writer.write_var(2, 0)?; // flags
        writer.write_var(3, num_ref_bytes)?;
        writer.write_var(8, num_offset_bytes)?;
        writer.write_var(8 * num_ref_bytes, self.cells.len() as u32)?;
        writer.write_var(8 * num_ref_bytes, root_count as u32)?;
        writer.write_var(8 * num_ref_bytes, 0)?; // Complete BOCs only
        writer.write_var(8 * num_offset_bytes, full_size)?;

        for &root in &self.roots_position {
            writer.write_var(8 * num_ref_bytes, root as u32)?;
        }

        for cell in &self.cells {
            write_raw_cell(&mut writer, cell, num_ref_bytes)?;
        }
        writer.byte_align()?;
        let mut bytes = writer.into_writer();
        if has_crc32 {
            bytes.extend(CRC_32_ISCSI.checksum(bytes.as_slice()).to_le_bytes());
        }
        Ok(bytes)
    }
}

fn raw_cell_size(cell: &RawCell, ref_size_bytes: u32) -> u32 {
    let data_len = cell.data_bits_len.div_ceil(8);
    2 + data_len as u32 + cell.refs_positions.len() as u32 * ref_size_bytes
}

fn write_raw_cell(
    writer: &mut BitWriter<Vec<u8>, BigEndian>,
    cell: &RawCell,
    ref_size_bytes: u32,
) -> Result<(), TonCoreError> {
    let level = cell.level_mask;
    let is_exotic = cell.cell_type.is_exotic() as u32;
    let num_refs = cell.refs_positions.len() as u32;
    let d1 = num_refs + is_exotic * 8 + level.mask() as u32 * 32;

    let padding_bits = cell.data_bits_len % 8;
    let full_bytes = padding_bits == 0;
    let data_len_bytes = cell.data_bits_len.div_ceil(8);
    // data_len_bytes <= 128 by spec, but d2 must be u8 by spec as well
    let d2 = (data_len_bytes * 2 - if full_bytes { 0 } else { 1 }) as u8; //subtract 1 if the last byte is not full

    writer.write_var(8, d1)?;
    writer.write_var(8, d2)?;

    let full_bytes = cell.data_bits_len / 8;
    writer.write_bytes(&cell.data[0..full_bytes])?;
    let rest_bits_len = cell.data_bits_len % 8;
    if rest_bits_len != 0 {
        writer.write_var(8, cell.data[full_bytes] | (1 << (8 - rest_bits_len - 1)))?;
    }

    for r in &cell.refs_positions {
        writer.write_var(8 * ref_size_bytes, *r as u32)?;
    }

    Ok(())
}
