use crate::bail_ton_core_data;
use crate::cell::boc::raw_cell::{RawCell, RefPosStorage};
use crate::cell::boc::read_var_size::read_var_size;
use crate::cell::ton_cell::{CellBytesReader, RefStorage};
use crate::cell::{TonCell, TonHash};
use crate::errors::TonCoreError;
use bitstream_io::BigEndian;
use bitstream_io::{BitWrite, BitWriter, ByteRead};
use crc::Crc;
use smallvec::SmallVec;
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Cursor;
use std::ops::Deref;
use std::sync::Arc;

const GENERIC_BOC_MAGIC: u32 = 0xb5ee9c72;
const CRC_32_ISCSI: Crc<u32> = Crc::<u32>::new(&crc::CRC_32_ISCSI);

/// `cells` must be topologically sorted.
#[derive(PartialEq, Debug, Clone)]
pub(super) struct RawBoC {
    pub(super) raw_cells: Vec<RawCell>,
    pub(super) roots_pos: RefPosStorage, // Usually one, sometimes two. Haven't seen more in practice.
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
        let mut roots_pos = RefPosStorage::with_capacity(roots_cnt);
        for _ in 0..roots_cnt {
            roots_pos.push(read_var_size(&mut reader, ref_pos_size_bytes)?)
        }
        //   index:has_idx?(cells * ##(off_bytes * 8))
        if has_idx {
            reader.skip(cells_cnt as u32 * off_bytes as u32)?;
        }
        //   cell_data:(tot_cells_size * [ uint8 ])
        let mut cells = Vec::with_capacity(cells_cnt);

        for _ in 0..cells_cnt {
            let cell = RawCell::read(&mut reader, ref_pos_size_bytes, data_storage.clone())?;
            cells.push(cell);
        }
        //   crc32c:has_crc32c?uint32
        let _crc32c = if has_crc32c { reader.read::<u32>()? } else { 0 };

        Ok(RawBoC {
            raw_cells: cells,
            roots_pos,
        })
    }

    //Based on https://github.com/toncenter/tonweb/blob/c2d5d0fc23d2aec55a0412940ce6e580344a288c/src/boc/Cell.js#L198
    pub(super) fn to_bytes(&self, add_crc32: bool) -> Result<Vec<u8>, TonCoreError> {
        let root_count = self.roots_pos.len();
        let ref_size_bits = 32 - (self.raw_cells.len() as u32).leading_zeros();
        let ref_pos_size_bytes = ref_size_bits.div_ceil(8);
        let has_idx = false;

        let mut full_size = 0u32;

        for cell in &self.raw_cells {
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
            (if has_idx { self.raw_cells.len() as u32 * num_offset_bytes } else { 0 }) +
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
        writer.write_var(8 * ref_pos_size_bytes, self.raw_cells.len() as u32)?;
        writer.write_var(8 * ref_pos_size_bytes, root_count as u32)?;
        writer.write_var(8 * ref_pos_size_bytes, 0)?; // Complete BOCs only
        writer.write_var(8 * num_offset_bytes, full_size)?;

        for &root in &self.roots_pos {
            writer.write_var(8 * ref_pos_size_bytes, root as u32)?;
        }

        for cell in &self.raw_cells {
            cell.write_to(&mut writer, ref_pos_size_bytes)?;
        }
        writer.byte_align()?;
        let mut bytes = writer.into_writer();
        if add_crc32 {
            bytes.extend(CRC_32_ISCSI.checksum(&bytes).to_le_bytes());
        }
        Ok(bytes)
    }

    //Based on https://github.com/toncenter/tonweb/blob/c2d5d0fc23d2aec55a0412940ce6e580344a288c/src/boc/Cell.js#L198
    pub(super) fn into_ton_cells(self) -> Result<Vec<TonCell>, TonCoreError> {
        let cells_len = self.raw_cells.len();
        let mut cells: Vec<TonCell> = Vec::with_capacity(cells_len);

        for (cell_index, cell_raw) in self.raw_cells.into_iter().enumerate().rev() {
            let mut refs = RefStorage::with_capacity(cell_raw.refs_pos.len());
            for ref_index in &cell_raw.refs_pos {
                if *ref_index <= cell_index {
                    bail_ton_core_data!("Invalid BoC: ref to parent cell detected");
                }
                refs.push(cells[cells_len - 1 - ref_index].clone());
            }
            cells.push(cell_raw.into_ton_cell(refs));
        }

        let mut roots = Vec::with_capacity(self.roots_pos.len());
        for root_index in self.roots_pos {
            roots.push(cells[cells_len - 1 - root_index].clone());
        }
        Ok(roots)
    }

    pub(super) fn from_ton_cells(roots: &[TonCell]) -> Result<Self, TonCoreError> {
        let cell_by_hash = build_and_verify_index(roots)?;

        // Sort indexed cells by their index value.
        let mut cell_sorted: Vec<_> = cell_by_hash.values().collect();
        cell_sorted.sort_unstable_by(|a, b| a.index.cmp(&b.index));

        // Remove gaps in indices.
        cell_sorted
            .iter()
            .enumerate()
            .for_each(|(real_index, indexed_cell)| *indexed_cell.index.borrow_mut() = real_index);

        let raw_cells = cell_sorted
            .into_iter()
            .map(|indexed| raw_from_indexed(indexed.cell, &cell_by_hash))
            .collect::<Result<_, TonCoreError>>()?;

        let roots_pos = roots.iter().map(|x| get_position(x, &cell_by_hash)).collect::<Result<_, TonCoreError>>()?;

        Ok(RawBoC { raw_cells, roots_pos })
    }
}

#[derive(Debug, Clone)]
struct IndexedCell<'a> {
    cell: &'a TonCell,
    index: RefCell<usize>, // internal mutability required
}

fn build_and_verify_index(roots: &[TonCell]) -> Result<HashMap<TonHash, IndexedCell<'_>>, TonCoreError> {
    let mut cur_cells = Vec::from_iter(roots.iter());
    let mut new_hash_index = 0;
    let mut cells_by_hash = HashMap::new();

    // Process cells to build the initial index.
    while !cur_cells.is_empty() {
        let mut next_cells = Vec::with_capacity(cur_cells.len() * 4);
        for cell in cur_cells {
            let hash = cell.hash()?;

            if cells_by_hash.contains_key(hash) {
                continue; // Skip if already indexed.
            }

            let indexed_cell = IndexedCell {
                cell,
                index: RefCell::new(new_hash_index),
            };
            cells_by_hash.insert(hash.clone(), indexed_cell);

            new_hash_index += 1;
            next_cells.extend(cell.refs());
        }

        cur_cells = next_cells;
    }

    // Ensure indices are in the correct order based on cell references.
    let mut verify_order = true;
    while verify_order {
        verify_order = false;

        for index_cell in cells_by_hash.values() {
            for ref_cell in index_cell.cell.refs() {
                let ref_hash = ref_cell.hash()?;
                if let Some(indexed) = cells_by_hash.get(ref_hash) {
                    if indexed.index < index_cell.index {
                        *indexed.index.borrow_mut() = new_hash_index;
                        new_hash_index += 1;
                        verify_order = true; // Verify if an index was updated.
                    }
                }
            }
        }
    }

    Ok(cells_by_hash)
}

fn raw_from_indexed(cell: &TonCell, cells_by_hash: &HashMap<TonHash, IndexedCell>) -> Result<RawCell, TonCoreError> {
    let refs_positions = raw_cell_refs_indexes(cell, cells_by_hash)?;
    Ok(RawCell {
        cell_type: cell.cell_type(),
        data_storage: cell.cell_data.data_storage.clone(),
        start_bit: cell.borders.start_bit,
        end_bit: cell.borders.end_bit,
        refs_pos: refs_positions,
        level_mask: cell.level_mask(),
    })
}

fn raw_cell_refs_indexes(
    cell: &TonCell,
    cells_by_hash: &HashMap<TonHash, IndexedCell>,
) -> Result<SmallVec<[usize; 4]>, TonCoreError> {
    let mut vec = SmallVec::with_capacity(cell.refs().len());
    for ref_pos in 0..cell.refs().len() {
        let cell_ref = &cell.refs()[ref_pos];
        vec.push(get_position(cell_ref, cells_by_hash)?);
    }
    Ok(vec)
}

fn get_position(cell: &TonCell, call_by_hash: &HashMap<TonHash, IndexedCell>) -> Result<usize, TonCoreError> {
    let hash = cell.hash()?;
    call_by_hash
        .get(hash)
        .ok_or_else(|| TonCoreError::Custom(format!("cell with hash {hash:?} not found in available hashes")))
        .map(|indexed_cell| *indexed_cell.index.borrow().deref())
}
