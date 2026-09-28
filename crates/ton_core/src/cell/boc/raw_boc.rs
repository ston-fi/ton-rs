use crate::bail_ton_core_data;
use crate::cell::boc::boc_reader::{BocReader, be_uint};
use crate::cell::boc::raw_cell::{RawCell, RefPosStorage};
use crate::cell::cell_meta::{CellMetaBuilder, LevelDepths, RefDepthsStorage};
use crate::cell::ton_cell::RefStorage;
use crate::cell::{LevelMask, TonCell, TonHash};
use crate::errors::TonCoreError;
use bitstream_io::BigEndian;
use bitstream_io::{BitWrite, BitWriter};
use crc::{Crc, Table};
use smallvec::SmallVec;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;

const GENERIC_BOC_MAGIC: u32 = 0xb5ee9c72;
// 16 lookup tables: several times faster than the default one on block-sized BoCs.
static CRC_32_ISCSI: Crc<u32, Table<16>> = Crc::<u32, Table<16>>::new(&crc::CRC_32_ISCSI);
const CRC32C_BYTES: usize = 4;
/// A serialized cell has at least its two descriptor bytes.
const MIN_CELL_BYTES: u64 = 2;

/// `cells` must be topologically sorted.
#[derive(PartialEq, Debug, Clone)]
pub(crate) struct RawBoC {
    pub(crate) raw_cells: Vec<RawCell>,
    pub(crate) roots_pos: RefPosStorage, // Usually one, sometimes two. Haven't seen more in practice.
}

impl RawBoC {
    /// Parses serialized BoC bytes, which may come from an untrusted source.
    ///
    /// Every size in the header must be backed by the input before anything is allocated,
    /// the input must be consumed exactly, and the index and CRC32C are verified when present.
    // https://github.com/ton-blockchain/ton/blob/24dc184a2ea67f9c47042b4104bbb4d82289fac1/crypto/tl/boc.tlb#L25
    pub(crate) fn from_bytes(data_storage: Arc<Vec<u8>>) -> Result<RawBoC, TonCoreError> {
        let data = data_storage.as_slice();
        let mut reader = BocReader::new(data);
        let magic = reader.read_uint(4)?;

        if magic != GENERIC_BOC_MAGIC as u64 {
            bail_ton_core_data!("Unexpected magic: {magic}");
        };

        // has_idx:(## 1) has_crc32c:(## 1) has_cache_bits:(## 1) flags:(## 2) { flags = 0 }
        let header = reader.read_u8()?;
        let has_idx = (header & 0b1000_0000) != 0;
        let has_crc32c = (header & 0b0100_0000) != 0;
        let has_cache_bits = (header & 0b0010_0000) != 0;
        if has_cache_bits && !has_idx {
            bail_ton_core_data!("Invalid BoC header: has_cache_bits requires has_idx");
        }

        // size:(## 3) { size <= 4 }
        let ref_pos_size = (header & 0b0000_0111) as usize;
        if !(1..=4).contains(&ref_pos_size) {
            bail_ton_core_data!("Invalid BoC header: ref_pos_size={ref_pos_size} (must be in 1..=4)");
        }

        //   off_bytes:(## 8) { off_bytes <= 8 }
        let off_bytes = reader.read_u8()? as usize;
        if !(1..=8).contains(&off_bytes) {
            bail_ton_core_data!("Invalid BoC header: off_bytes={off_bytes} (must be in 1..=8)");
        }
        // Counts have at most 4 bytes, so sums and products of them with small sizes fit in u64.
        //cells:(##(size * 8))
        let cells_cnt = reader.read_uint(ref_pos_size)?;
        //   roots:(##(size * 8)) { roots >= 1 }
        let roots_cnt = reader.read_uint(ref_pos_size)?;
        if roots_cnt < 1 {
            bail_ton_core_data!("Invalid BoC header: roots({roots_cnt}) >= 1");
        }
        //   absent:(##(size * 8)) { roots + absent <= cells }
        let absent = reader.read_uint(ref_pos_size)?;
        if roots_cnt + absent > cells_cnt {
            bail_ton_core_data!("Invalid header: roots({roots_cnt}) + absent({absent}) <= cells({cells_cnt})");
        }
        //   tot_cells_size:(##(off_bytes * 8))
        let tot_cells_size = reader.read_uint(off_bytes)?;
        if tot_cells_size < cells_cnt * MIN_CELL_BYTES {
            bail_ton_core_data!("Invalid BoC header: {cells_cnt} cells can't fit in tot_cells_size={tot_cells_size}");
        }

        let roots_size = roots_cnt * ref_pos_size as u64;
        let index_size = if has_idx { cells_cnt * off_bytes as u64 } else { 0 };
        let crc_size = if has_crc32c { CRC32C_BYTES as u64 } else { 0 };
        let expected_size = (roots_size + index_size + crc_size).checked_add(tot_cells_size);
        if expected_size != Some(reader.remaining() as u64) {
            bail_ton_core_data!(
                "Invalid BoC: header describes {} bytes after it, got {}",
                expected_size.map_or_else(|| "too many".to_string(), |size| size.to_string()),
                reader.remaining()
            );
        }
        // All sizes are backed by the input from here, so they fit in usize.
        let cells_cnt = cells_cnt as usize;

        //   crc32c:has_crc32c?uint32
        if has_crc32c {
            let (payload, crc) = data.split_at(data.len() - CRC32C_BYTES);
            let expected_crc = u32::from_le_bytes([crc[0], crc[1], crc[2], crc[3]]);
            if CRC_32_ISCSI.checksum(payload) != expected_crc {
                bail_ton_core_data!("Invalid BoC: CRC32C mismatch");
            }
        }

        //   root_list:(roots * ##(size * 8))
        let mut roots_pos = RefPosStorage::with_capacity(roots_cnt as usize);
        for _ in 0..roots_cnt {
            let root_pos = reader.read_uint(ref_pos_size)? as usize;
            if root_pos >= cells_cnt {
                bail_ton_core_data!("Invalid BoC: root index {root_pos} is out of {cells_cnt} cells");
            }
            roots_pos.push(root_pos);
        }
        //   index:has_idx?(cells * ##(off_bytes * 8))
        let index = reader.take(index_size as usize)?;
        //   cell_data:(tot_cells_size * [ uint8 ])
        let mut cells_reader = reader.split(tot_cells_size as usize)?;
        let cells_start = cells_reader.position();
        let mut cells = Vec::with_capacity(cells_cnt);
        let mut index_entries = index.chunks_exact(off_bytes);
        for _ in 0..cells_cnt {
            cells.push(RawCell::read(&mut cells_reader, ref_pos_size, &data_storage)?);
            // The TON node locates cells by the index, so it must agree with the sequential layout.
            if let Some(entry) = index_entries.next() {
                let entry = be_uint(entry);
                let cell_end = if has_cache_bits { entry >> 1 } else { entry };
                if cell_end != (cells_reader.position() - cells_start) as u64 {
                    bail_ton_core_data!(
                        "Invalid BoC: index entry for cell {} doesn't match cell data",
                        cells.len() - 1
                    );
                }
            }
        }
        if cells_reader.remaining() != 0 {
            bail_ton_core_data!("Invalid BoC: {} unused bytes after cells", cells_reader.remaining());
        }

        Ok(RawBoC {
            raw_cells: cells,
            roots_pos,
        })
    }

    //Based on https://github.com/toncenter/tonweb/blob/c2d5d0fc23d2aec55a0412940ce6e580344a288c/src/boc/Cell.js#L198
    pub(crate) fn to_bytes(&self, add_crc32: bool) -> Result<Vec<u8>, TonCoreError> {
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

    /// Builds cells of a BoC parsed from untrusted bytes, rejecting cells the TON node rejects:
    /// malformed exotic cells, level masks that differ from the computed ones, Merkle cells whose
    /// stored hashes or depths don't match their children, and cells deeper than TVM allows.
    pub(crate) fn into_verified_ton_cells(self) -> Result<Vec<TonCell>, TonCoreError> {
        // Depths per level, in the same order as the built cells.
        let mut depths: Vec<LevelDepths> = Vec::with_capacity(self.raw_cells.len());
        self.build_ton_cells(|cell, refs_pos, descriptor_level_mask| {
            let refs_depths: RefDepthsStorage = refs_pos.iter().map(|&pos| depths[pos]).collect();
            depths.push(CellMetaBuilder::new(cell).verify_deserialized(descriptor_level_mask, &refs_depths)?);
            Ok(())
        })
    }

    /// Builds cells of a BoC made by [`RawBoC::from_ton_cells`], which only holds already built cells.
    pub(crate) fn into_ton_cells(self) -> Result<Vec<TonCell>, TonCoreError> {
        self.build_ton_cells(|_, _, _| Ok(()))
    }

    /// Builds cells bottom-up, calling `on_cell` with every cell, the positions of its refs
    /// among the cells built before it, and its serialized level mask.
    //Based on https://github.com/toncenter/tonweb/blob/c2d5d0fc23d2aec55a0412940ce6e580344a288c/src/boc/Cell.js#L198
    fn build_ton_cells<F>(self, mut on_cell: F) -> Result<Vec<TonCell>, TonCoreError>
    where
        F: FnMut(&TonCell, &RefPosStorage, LevelMask) -> Result<(), TonCoreError>,
    {
        let cells_len = self.raw_cells.len();
        let mut cells: Vec<TonCell> = Vec::with_capacity(cells_len);
        // Cell `index` is stored at `cells_len - 1 - index`, since cells are built from the last one.
        let built_pos = |index: usize| cells_len - 1 - index;

        for (cell_index, cell_raw) in self.raw_cells.into_iter().enumerate().rev() {
            let mut refs = RefStorage::with_capacity(cell_raw.refs_pos.len());
            let mut refs_pos = RefPosStorage::with_capacity(cell_raw.refs_pos.len());
            for &ref_index in &cell_raw.refs_pos {
                if ref_index <= cell_index {
                    bail_ton_core_data!("Invalid BoC: ref to parent cell detected");
                }
                if ref_index >= cells_len {
                    bail_ton_core_data!("Invalid BoC: ref to cell {ref_index} is out of {cells_len} cells");
                }
                refs.push(cells[built_pos(ref_index)].clone());
                refs_pos.push(built_pos(ref_index));
            }
            let level_mask = cell_raw.level_mask;
            let cell = cell_raw.into_ton_cell(refs);
            on_cell(&cell, &refs_pos, level_mask)?;
            cells.push(cell);
        }

        let mut roots = Vec::with_capacity(self.roots_pos.len());
        for root_index in self.roots_pos {
            if root_index >= cells_len {
                bail_ton_core_data!("Invalid BoC: root index {root_index} is out of {cells_len} cells");
            }
            roots.push(cells[built_pos(root_index)].clone());
        }
        Ok(roots)
    }

    pub(crate) fn from_ton_cells(roots: &[TonCell], preserve_hash: bool) -> Result<Self, TonCoreError> {
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
            .map(|indexed| raw_from_indexed(indexed.cell, &cell_by_hash, preserve_hash))
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
            cells_by_hash.insert(*hash, indexed_cell);

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
                if let Some(indexed) = cells_by_hash.get(ref_hash)
                    && indexed.index < index_cell.index
                {
                    *indexed.index.borrow_mut() = new_hash_index;
                    new_hash_index += 1;
                    verify_order = true; // Verify if an index was updated.
                }
            }
        }
    }

    Ok(cells_by_hash)
}

fn raw_from_indexed(
    cell: &TonCell,
    cells_by_hash: &HashMap<TonHash, IndexedCell>,
    preserve_hash: bool,
) -> Result<RawCell, TonCoreError> {
    let refs_positions = raw_cell_refs_indexes(cell, cells_by_hash)?;
    Ok(RawCell {
        cell_type: cell.cell_type(),
        data_storage: cell.cell_data.data_storage.clone(),
        start_bit: cell.borders.start_bit,
        end_bit: cell.borders.end_bit,
        refs_pos: refs_positions,
        level_mask: cell.level_mask(),
        hashes_depths: if preserve_hash { cell.meta.hashes_depths.get().cloned() } else { None },
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
