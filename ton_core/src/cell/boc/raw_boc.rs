use crate::bail_ton_core_data;
use crate::cell::boc::raw_cell::{RawCell, RefPosStorage};
use crate::cell::boc::read_var_size::read_var_size;
use crate::cell::ton_cell::CellBytesReader;
use crate::errors::TonCoreError;
use bitstream_io::BigEndian;
use bitstream_io::{BitWrite, BitWriter, ByteRead};
use crc::Crc;
use std::io::Cursor;
use std::sync::Arc;

const GENERIC_BOC_MAGIC: u32 = 0xb5ee9c72;
const CRC_32_ISCSI: Crc<u32> = Crc::<u32>::new(&crc::CRC_32_ISCSI);

/// `cells` must be topologically sorted.
#[derive(PartialEq, Debug, Clone)]
pub(super) struct RawBoC {
    pub(super) cells: Vec<RawCell>,
    pub(super) roots_positions: RefPosStorage, // Usually one, sometimes two. Haven't seen more in practice.
}

impl RawBoC {
    // https://github.com/ton-blockchain/ton/blob/24dc184a2ea67f9c47042b4104bbb4d82289fac1/crypto/tl/boc.tlb#L25
    pub(super) fn from_bytes(data_storage: Arc<Vec<u8>>) -> Result<RawBoC, TonCoreError> {
        let cursor = Cursor::new(data_storage.as_slice());
        let mut reader = CellBytesReader::new(cursor);
        let magic = reader.read::<u32>()?;

        if magic != GENERIC_BOC_MAGIC {
            bail_ton_core_data!("Unexpected magic: {magic}");
        };

        let (has_idx, has_crc32c, _has_cache_bits, ref_pos_size_bytes) = {
            // has_idx:(## 1) has_crc32c:(## 1) has_cache_bits:(## 1) flags:(## 2) { flags = 0 }
            let header = reader.read::<u8>()?;
            let has_idx = (header & 0b1000_0000) != 0;
            let has_crc32c = (header & 0b0100_0000) != 0;
            let has_cache_bits = (header & 0b0010_0000) != 0;

            // size:(## 3) { size <= 4 }
            let ref_pos_size = header & 0b0000_0111;
            if ref_pos_size > 4 {
                bail_ton_core_data!("Invalid BoC header: ref_pos_size={ref_pos_size} (must be <= 4)");
            }

            (has_idx, has_crc32c, has_cache_bits, ref_pos_size)
        };

        //   off_bytes:(## 8) { off_bytes <= 8 }
        let off_bytes = reader.read::<u8>()?;
        if off_bytes > 8 {
            bail_ton_core_data!("Invalid BoC header: off_bytes({off_bytes}) <= 8");
        }
        //cells:(##(size * 8))
        let cells_cnt = read_var_size(&mut reader, ref_pos_size_bytes)?;
        //   roots:(##(size * 8)) { roots >= 1 }
        let roots_cnt = read_var_size(&mut reader, ref_pos_size_bytes)?;
        if roots_cnt < 1 {
            bail_ton_core_data!("Invalid BoC header: roots({roots_cnt}) >= 1");
        }
        //   absent:(##(size * 8)) { roots + absent <= cells }
        let absent = read_var_size(&mut reader, ref_pos_size_bytes)?;
        if roots_cnt + absent > cells_cnt {
            bail_ton_core_data!("Invalid header: roots({roots_cnt}) + absent({absent}) <= cells({cells_cnt})");
        }
        //   tot_cells_size:(##(off_bytes * 8))
        let _tot_cells_size = read_var_size(&mut reader, off_bytes)?;
        //   root_list:(roots * ##(size * 8))
        let mut roots_position = RefPosStorage::with_capacity(roots_cnt);
        for _ in 0..roots_cnt {
            roots_position.push(read_var_size(&mut reader, ref_pos_size_bytes)?)
        }
        //   index:has_idx?(cells * ##(off_bytes * 8))
        if has_idx {
            reader.skip(cells_cnt as u32 * off_bytes as u32)?;
        }
        //   cell_data:(tot_cells_size * [ uint8 ])
        let mut cells = Vec::with_capacity(cells_cnt);

        for _ in 0..cells_cnt {
            let cell = RawCell::new(&mut reader, ref_pos_size_bytes, data_storage.clone())?;
            cells.push(cell);
        }
        //   crc32c:has_crc32c?uint32
        let _crc32c = if has_crc32c { reader.read::<u32>()? } else { 0 };

        Ok(RawBoC {
            cells,
            roots_positions: roots_position,
        })
    }

    //Based on https://github.com/toncenter/tonweb/blob/c2d5d0fc23d2aec55a0412940ce6e580344a288c/src/boc/Cell.js#L198
    pub(super) fn to_bytes(&self, add_crc32: bool) -> Result<Vec<u8>, TonCoreError> {
        let root_count = self.roots_positions.len();
        let ref_size_bits = 32 - (self.cells.len() as u32).leading_zeros();
        let ref_pos_size_bytes = ref_size_bits.div_ceil(8);
        let has_idx = false;

        let mut full_size = 0u32;

        for cell in &self.cells {
            full_size += cell.size_in_boc_bytes(ref_pos_size_bytes);
        }

        let num_offset_bits = 32 - full_size.leading_zeros();
        let num_offset_bytes = num_offset_bits.div_ceil(8);

        let total_size = 4 + // magic
            1 + // flags and s_bytes
            1 + // offset_bytes
            3 * ref_pos_size_bytes + // cells_num, roots, complete
            num_offset_bytes + // full_size
            ref_pos_size_bytes + // root_idx
            (if has_idx { self.cells.len() as u32 * num_offset_bytes } else { 0 }) +
            full_size +
            (if add_crc32 { 4 } else { 0 });

        let mut writer = BitWriter::endian(Vec::with_capacity(total_size as usize), BigEndian);
        writer.write_var(32, GENERIC_BOC_MAGIC)?;
        writer.write_bit(has_idx)?;
        writer.write_bit(add_crc32)?;
        writer.write_bit(false)?; // has_cache_bits
        writer.write_var(2, 0)?; // flags
        writer.write_var(3, ref_pos_size_bytes)?;
        writer.write_var(8, num_offset_bytes)?;
        writer.write_var(8 * ref_pos_size_bytes, self.cells.len() as u32)?;
        writer.write_var(8 * ref_pos_size_bytes, root_count as u32)?;
        writer.write_var(8 * ref_pos_size_bytes, 0)?; // Complete BOCs only
        writer.write_var(8 * num_offset_bytes, full_size)?;

        for &root in &self.roots_positions {
            writer.write_var(8 * ref_pos_size_bytes, root as u32)?;
        }

        for cell in &self.cells {
            cell.write_to(&mut writer, ref_pos_size_bytes)?;
        }
        writer.byte_align()?;
        let mut bytes = writer.into_writer();
        if add_crc32 {
            bytes.extend(CRC_32_ISCSI.checksum(&bytes).to_le_bytes());
        }
        Ok(bytes)
    }
}
