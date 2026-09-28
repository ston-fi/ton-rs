use crate::bail_ton_core_data;
use crate::bits_utils::BitsUtils;
use crate::cell::CellMeta;
use crate::cell::cell_meta::HashesDepthsStorage;
use crate::cell::cell_meta::cell_type::CellType;
use crate::cell::cell_meta::level_mask::LevelMask;
use crate::cell::ton_cell::{CellBitWriter, TonCell};
use crate::cell::ton_hash::TonHash;
use crate::errors::TonCoreError;
use bitstream_io::{BigEndian, BitWrite, BitWriter};
use sha2::{Digest, Sha256};
use smallvec::SmallVec;
use std::borrow::Cow;

/// TVM refuses cells deeper than this (`vm::CellTraits::max_depth`).
pub(crate) const MAX_CELL_DEPTH: u16 = 1024;

/// Depths of a cell for levels 0..=3.
pub(crate) type LevelDepths = [u16; 4];
pub(crate) type RefDepthsStorage = SmallVec<[LevelDepths; TonCell::MAX_REFS_COUNT]>;

/// Type byte and level mask.
const PRUNED_HEADER_BYTES: usize = 2;
/// Type byte and library hash.
const LIBRARY_BYTES: usize = 1 + TonHash::BYTES_LEN;
/// Type byte, then a hash and a depth for the only ref.
const MERKLE_PROOF_BYTES: usize = 1 + TonHash::BYTES_LEN + CellMeta::DEPTH_BYTES;
/// Type byte, then hashes and depths for both refs.
const MERKLE_UPDATE_BYTES: usize = 1 + 2 * (TonHash::BYTES_LEN + CellMeta::DEPTH_BYTES);

pub struct CellMetaBuilder<'a> {
    cell_type: CellType,
    data: &'a [u8],
    start_bit: usize,
    is_byte_aligned: bool,
    data_len_bits: usize,
    refs: &'a [TonCell],
}

#[derive(Debug)]
struct Pruned {
    hash: TonHash,
    depth: u16,
}

impl<'a> CellMetaBuilder<'a> {
    pub fn new(cell: &'a TonCell) -> Self {
        let start_bit = cell.borders.start_bit;
        let data_bits_len = cell.borders.end_bit - start_bit;
        Self {
            cell_type: cell.cell_type(),
            data: &cell.cell_data.data_storage,
            start_bit,
            is_byte_aligned: start_bit.is_multiple_of(8),
            data_len_bits: data_bits_len,
            refs: cell.refs(),
        }
    }

    pub fn validate(&self) -> Result<(), TonCoreError> {
        match self.cell_type {
            CellType::Ordinary => self.validate_ordinary(), // guaranteed by builder
            CellType::PrunedBranch => self.validate_pruned(),
            CellType::LibraryRef => self.validate_library(),
            CellType::MerkleProof => self.validate_merkle_proof(),
            CellType::MerkleUpdate => self.validate_merkle_update(),
        }
    }

    pub fn calc_level_mask(&self) -> LevelMask {
        match self.cell_type {
            CellType::Ordinary => self.calc_level_mask_ordinary(),
            CellType::PrunedBranch => self.calc_level_mask_pruned(),
            CellType::LibraryRef => LevelMask::new(0),
            CellType::MerkleProof => self.refs[0].level_mask() >> 1,
            CellType::MerkleUpdate => self.calc_level_mask_merkle_update(),
        }
    }

    fn validate_ordinary(&self) -> Result<(), TonCoreError> {
        if self.data_len_bits > TonCell::MAX_DATA_LEN_BITS {
            bail_ton_core_data!("Ordinary cell data bits length is too big");
        }
        Ok(())
    }

    fn validate_pruned(&self) -> Result<(), TonCoreError> {
        if !self.refs.is_empty() {
            bail_ton_core_data!("Pruned cell can't have refs");
        }
        if self.data_len_bits < PRUNED_HEADER_BYTES * 8 {
            bail_ton_core_data!("Pruned Branch require at least 16 bits data");
        }

        let level_mask = self.calc_level_mask_pruned();
        if !(1..=LevelMask::MAX_LEVEL.mask()).contains(&level_mask.level()) {
            bail_ton_core_data!("Pruned Branch cell level must be in range [1, 3] (got mask {level_mask})");
        }

        let expected_size = pruned_len_bytes(level_mask) * 8;
        if self.data_len_bits != expected_size {
            bail_ton_core_data!("PrunedBranch must have exactly {expected_size} bits, got {}", self.data_len_bits);
        }

        Ok(())
    }

    fn validate_library(&self) -> Result<(), TonCoreError> {
        const LIB_CELL_BITS_LEN: usize = LIBRARY_BYTES * 8;

        if self.data_len_bits != LIB_CELL_BITS_LEN {
            bail_ton_core_data!("Lib cell must have exactly {LIB_CELL_BITS_LEN} bits, got {}", self.data_len_bits);
        }
        if !self.refs.is_empty() {
            bail_ton_core_data!("Lib cell can't have refs");
        }

        Ok(())
    }

    fn validate_merkle_proof(&self) -> Result<(), TonCoreError> {
        const MERKLE_PROOF_BITS_LEN: usize = MERKLE_PROOF_BYTES * 8;

        if self.data_len_bits != MERKLE_PROOF_BITS_LEN {
            bail_ton_core_data!(
                "MerkleProof must have exactly {MERKLE_PROOF_BITS_LEN} bits, got {}",
                self.data_len_bits
            );
        }
        if self.refs.len() != 1 {
            bail_ton_core_data!("Merkle Proof cell must have exactly 1 ref");
        }
        Ok(())
    }

    fn validate_merkle_update(&self) -> Result<(), TonCoreError> {
        const MERKLE_UPDATE_BITS_LEN: usize = MERKLE_UPDATE_BYTES * 8;

        if self.data_len_bits != MERKLE_UPDATE_BITS_LEN {
            bail_ton_core_data!(
                "MerkleUpdate must have exactly {MERKLE_UPDATE_BITS_LEN} bits, got {}",
                self.data_len_bits
            );
        }
        if self.refs.len() != 2 {
            bail_ton_core_data!("Merkle Update cell must have exactly 2 refs");
        }
        Ok(())
    }

    /// Checks a cell deserialized from untrusted bytes the way the TON node does,
    /// given the level mask from its descriptor and the depths of its refs.
    /// Returns the depths of the cell.
    pub(crate) fn verify_deserialized(
        &self,
        descriptor_level_mask: LevelMask,
        refs_depths: &[LevelDepths],
    ) -> Result<LevelDepths, TonCoreError> {
        self.validate()?;

        let level_mask = self.calc_level_mask();
        if level_mask != descriptor_level_mask {
            bail_ton_core_data!(
                "{:?} cell level mask {descriptor_level_mask} doesn't match computed {level_mask}",
                self.cell_type
            );
        }

        let depths = self.calc_level_depths(level_mask, refs_depths)?;
        if let Some(depth) = depths.iter().find(|&&depth| depth > MAX_CELL_DEPTH) {
            bail_ton_core_data!("Cell depth {depth} exceeds {MAX_CELL_DEPTH}");
        }

        if matches!(self.cell_type, CellType::MerkleProof | CellType::MerkleUpdate) {
            self.verify_merkle_refs(refs_depths)?;
        }
        Ok(depths)
    }

    /// Computes depths for all levels without hashing, the same way `calc_hashes_and_depths` does.
    fn calc_level_depths(
        &self,
        level_mask: LevelMask,
        refs_depths: &[LevelDepths],
    ) -> Result<LevelDepths, TonCoreError> {
        let mut depths = LevelDepths::default();
        if self.cell_type == CellType::PrunedBranch {
            let pruned = self.calc_pruned_hash_depth(level_mask)?;
            for (level, depth) in (0u8..).zip(depths.iter_mut()) {
                // Levels below the cell's own keep the stored depths, its own level has depth 0.
                if let Some(stored) = pruned.get(level_mask.apply(level).hash_index()) {
                    *depth = stored.depth;
                }
            }
            return Ok(depths);
        }

        let extra_level = matches!(self.cell_type, CellType::MerkleProof | CellType::MerkleUpdate) as usize;
        for (level, depth) in depths.iter_mut().enumerate() {
            let ref_level = (level + extra_level).min(LevelMask::MAX_LEVEL.mask() as usize);
            for ref_depths in refs_depths {
                *depth = (*depth).max(ref_depths[ref_level].saturating_add(1));
            }
        }
        Ok(depths)
    }

    /// The TON node requires a Merkle cell to store the level 0 hash and depth of each ref.
    fn verify_merkle_refs(&self, refs_depths: &[LevelDepths]) -> Result<(), TonCoreError> {
        let data = self.data_bytes();
        let hashes_len = self.refs.len() * TonHash::BYTES_LEN;
        let depths_len = self.refs.len() * CellMeta::DEPTH_BYTES;
        let Some(stored) = data.get(1..1 + hashes_len + depths_len) else {
            bail_ton_core_data!("{:?} cell is too short for {} refs", self.cell_type, self.refs.len());
        };
        let (hashes, depths) = stored.split_at(hashes_len);

        let refs = self.refs.iter().zip(refs_depths);
        for ((cell_ref, ref_depths), (hash, depth)) in
            refs.zip(hashes.chunks_exact(TonHash::BYTES_LEN).zip(depths.chunks_exact(CellMeta::DEPTH_BYTES)))
        {
            if cell_ref.hash_for_level(LevelMask::new(0))?.as_slice() != hash {
                bail_ton_core_data!("{:?} cell stored hash doesn't match its ref", self.cell_type);
            }
            let stored_depth = u16::from_be_bytes([depth[0], depth[1]]);
            if ref_depths[0] != stored_depth {
                bail_ton_core_data!(
                    "{:?} cell stored depth {stored_depth} doesn't match ref depth {}",
                    self.cell_type,
                    ref_depths[0]
                );
            }
        }
        Ok(())
    }

    fn calc_level_mask_ordinary(&self) -> LevelMask {
        let mut mask = LevelMask::new(0);
        for cell_ref in self.refs {
            mask |= cell_ref.level_mask();
        }
        mask
    }

    fn calc_level_mask_pruned(&self) -> LevelMask {
        match self.data_len_bits >= PRUNED_HEADER_BYTES * 8 {
            true => LevelMask::new(self.data_bytes()[1]),
            false => LevelMask::new(0),
        }
    }

    fn calc_level_mask_merkle_update(&self) -> LevelMask {
        let refs_lm = self.refs[0].level_mask() | self.refs[1].level_mask();
        refs_lm >> 1
    }

    /// Cell data as whole bytes, borrowed when the cell starts and ends on byte boundaries.
    fn data_bytes(&self) -> Cow<'a, [u8]> {
        let len = self.data_len_bits.div_ceil(8);
        let start = self.start_bit / 8;
        if self.is_byte_aligned
            && self.data_len_bits.is_multiple_of(8)
            && let Some(bytes) = self.data.get(start..start + len)
        {
            return Cow::Borrowed(bytes);
        }
        let mut data = vec![0; len];
        BitsUtils::read_with_offset(self.data, &mut data, self.start_bit, self.data_len_bits);
        Cow::Owned(data)
    }

    /// This function replicates unknown logic of resolving cell data
    /// <https://github.com/ton-blockchain/ton/blob/24dc184a2ea67f9c47042b4104bbb4d82289fac1/crypto/vm/cells/DataCell.cpp#L214>
    pub fn calc_hashes_and_depths(&self, level_mask: LevelMask) -> Result<HashesDepthsStorage, TonCoreError> {
        let hash_count = match self.cell_type {
            CellType::PrunedBranch => 1,
            _ => level_mask.hash_count(),
        };

        let total_hash_count = level_mask.hash_count();
        let hash_i_offset = total_hash_count - hash_count;

        let mut hashes = Vec::<TonHash>::with_capacity(hash_count);
        let mut depths = Vec::with_capacity(hash_count);

        // Iterate through significant levels
        let sign_levels = (0..=level_mask.level()).filter(|&i| level_mask.is_significant(i));
        for (hash_pos, level_pos) in sign_levels.enumerate() {
            if hash_pos < hash_i_offset {
                continue;
            }

            let mut data = vec![0; self.data_len_bits.div_ceil(8)];
            BitsUtils::read_with_offset(self.data, &mut data, self.start_bit, self.data_len_bits);
            // Get current data

            let (cur_data, cur_bit_len) = if hash_pos == hash_i_offset {
                (data.as_slice(), self.data_len_bits)
            } else {
                let prev_hash = &hashes[hash_pos - hash_i_offset - 1];
                (prev_hash.as_slice(), 256)
            };

            // Calculate Depth
            let depth = if self.refs.is_empty() {
                0
            } else {
                let mut max_ref_depth = 0;
                for cell_ref in self.refs {
                    let ref_depth = self.get_ref_depth(cell_ref, level_pos)?;
                    max_ref_depth = max_ref_depth.max(ref_depth);
                }
                match max_ref_depth.checked_add(1) {
                    Some(depth) => depth,
                    None => bail_ton_core_data!("Cell depth overflow"),
                }
            };

            // Calculate Hash
            let repr = self.get_repr_for_data(cur_data, cur_bit_len, level_mask, level_pos)?;
            let hash = TonHash::from_slice(&Sha256::new_with_prefix(repr).finalize())?;
            hashes.push(hash);
            depths.push(depth);
        }

        self.resolve_hashes_and_depths(&hashes, &depths, level_mask)
    }

    fn get_repr_for_data(
        &self,
        cur_data: &[u8],
        cur_data_bits_len: usize,
        level_mask: LevelMask,
        level: u8,
    ) -> Result<Vec<u8>, TonCoreError> {
        // descriptors + data + (hash + depth) * refs_count
        let buffer_len = 2 + cur_data.len() + (32 + 2) * self.refs.len();

        let mut writer = BitWriter::endian(Vec::with_capacity(buffer_len), BigEndian);
        let d1 = self.get_refs_descriptor(level_mask.apply(level));
        let d2 = get_bits_descriptor(self.data_len_bits);

        // Write descriptors
        writer.write_var(8, d1)?;
        writer.write_var(8, d2)?;
        // Write main data
        write_data(&mut writer, cur_data, cur_data_bits_len)?;
        // Write ref data
        self.write_ref_depths(&mut writer, level)?;
        self.write_ref_hashes(&mut writer, level)?;

        if !writer.byte_aligned() {
            bail_ton_core_data!("Stream for cell repr is not byte-aligned");
        }
        Ok(writer.into_writer())
    }

    /// Calculates d1 descriptor for cell
    /// See <https://docs.ton.org/tvm.pdf>, section 3.1.4.
    fn get_refs_descriptor<L: Into<u8>>(&self, level_mask: L) -> u8 {
        let cell_type_var = self.cell_type.is_exotic() as u8;
        self.refs.len() as u8 + 8 * cell_type_var + level_mask.into() * 32
    }

    fn write_ref_hashes(&self, writer: &mut CellBitWriter, level: u8) -> Result<(), TonCoreError> {
        for cell_ref in self.refs {
            let ref_hash = self.get_ref_hash(cell_ref, level)?;
            writer.write_bytes(ref_hash.as_slice())?;
        }

        Ok(())
    }

    fn write_ref_depths(&self, writer: &mut CellBitWriter, level: u8) -> Result<(), TonCoreError> {
        for cell_ref in self.refs {
            let ref_depth = self.get_ref_depth(cell_ref, level)?;
            writer.write_var(8, ref_depth / 256)?;
            writer.write_var(8, ref_depth % 256)?;
        }
        Ok(())
    }

    fn resolve_hashes_and_depths(
        &self,
        hashes: &[TonHash],
        depths: &[u16],
        level_mask: LevelMask,
    ) -> Result<HashesDepthsStorage, TonCoreError> {
        let mut resolved_hashes = SmallVec::from([TonHash::ZERO; 4]);
        let mut resolved_depths = SmallVec::from([0; 4]);
        let pruned = match self.cell_type {
            CellType::PrunedBranch => self.calc_pruned_hash_depth(level_mask)?,
            _ => Vec::new(),
        };

        for i in 0..4 {
            let hash_index = level_mask.apply(i).hash_index();

            let (hash, depth) = match self.cell_type {
                CellType::PrunedBranch => match pruned.get(hash_index) {
                    Some(stored) => (stored.hash, stored.depth),
                    // The pruned branch's own level
                    None => (hashes[0], depths[0]),
                },
                _ => (hashes[hash_index], depths[hash_index]),
            };

            resolved_hashes[i as usize] = hash;
            resolved_depths[i as usize] = depth;
        }

        Ok((resolved_hashes, resolved_depths))
    }

    fn get_ref_depth(&self, cell_ref: &TonCell, level: u8) -> Result<u16, TonCoreError> {
        let extra_level = matches!(self.cell_type, CellType::MerkleProof | CellType::MerkleUpdate) as usize;
        let lm = (level as usize + extra_level).min(3) as u8;
        cell_ref.depth_for_level(LevelMask::new(lm))
    }

    fn get_ref_hash(&self, cell_ref: &'a TonCell, level: u8) -> Result<&'a TonHash, TonCoreError> {
        let extra_level = matches!(self.cell_type, CellType::MerkleProof | CellType::MerkleUpdate) as usize;
        let lm = (level as usize + extra_level).min(3) as u8;
        cell_ref.hash_for_level(LevelMask::new(lm))
    }

    /// Reads the hashes and depths a pruned branch stores for the levels below its own:
    /// one per set bit of the level mask, all hashes first, then all depths.
    fn calc_pruned_hash_depth(&self, level_mask: LevelMask) -> Result<Vec<Pruned>, TonCoreError> {
        let count = level_mask.hash_index();
        let data = self.data_bytes();
        let Some(stored) = data.get(PRUNED_HEADER_BYTES..pruned_len_bytes(level_mask)) else {
            bail_ton_core_data!("Pruned Branch is too short for level mask {level_mask}");
        };
        let (hashes, depths) = stored.split_at(count * TonHash::BYTES_LEN);

        let mut result = Vec::with_capacity(count);
        for (hash, depth) in hashes.chunks_exact(TonHash::BYTES_LEN).zip(depths.chunks_exact(CellMeta::DEPTH_BYTES)) {
            result.push(Pruned {
                hash: TonHash::from_slice(hash)?,
                depth: u16::from_be_bytes([depth[0], depth[1]]),
            });
        }
        Ok(result)
    }
}

/// Pruned branch length: header, then a hash and a depth per set bit of the level mask.
fn pruned_len_bytes(level_mask: LevelMask) -> usize {
    PRUNED_HEADER_BYTES + level_mask.hash_index() * (TonHash::BYTES_LEN + CellMeta::DEPTH_BYTES)
}

/// Calculates d2 descriptor for cell
/// See <https://docs.ton.org/tvm.pdf>, section 3.1.4.
fn get_bits_descriptor(data_bits_len: usize) -> u8 {
    (data_bits_len / 8 + data_bits_len.div_ceil(8)) as u8
}

fn write_data(writer: &mut CellBitWriter, data: &[u8], bit_len: usize) -> Result<(), TonCoreError> {
    let data_len = data.len();
    let rest_bits = bit_len % 8;
    let full_bytes = rest_bits == 0;

    if !full_bytes {
        writer.write_bytes(&data[..data_len - 1])?;
        let last_byte = data[data_len - 1];
        let last_bits = last_byte | (1 << (8 - rest_bits - 1));
        writer.write_var(8, last_bits)?;
    } else {
        writer.write_bytes(data)?;
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::cell::ton_cell::{CellBorders, CellData, RefStorage};
    use std::sync::Arc;

    fn empty_cell_ref() -> TonCell {
        TonCell::empty().to_owned()
    }

    #[test]
    fn test_refs_descriptor_d1() {
        let meta_builder = CellMetaBuilder::new(TonCell::empty());
        assert_eq!(meta_builder.get_refs_descriptor(0), 0);
        assert_eq!(meta_builder.get_refs_descriptor(3), 96);

        let cell_2 = TonCell {
            cell_type: CellType::Ordinary,
            cell_data: Arc::new(CellData {
                data_storage: Arc::new(vec![]),
                refs: RefStorage::from_iter([empty_cell_ref()]),
            }),
            meta: Arc::new(CellMeta::default()),
            borders: CellBorders {
                start_bit: 0,
                end_bit: 0,
                start_ref: 0,
                end_ref: 1,
            },
        };
        let meta_builder = CellMetaBuilder::new(&cell_2);
        assert_eq!(meta_builder.get_refs_descriptor(3), 97);

        let cell_3 = TonCell {
            cell_type: CellType::Ordinary,
            cell_data: Arc::new(CellData {
                data_storage: Arc::new(vec![]),
                refs: RefStorage::from_iter([empty_cell_ref(), empty_cell_ref()]),
            }),
            meta: Arc::new(CellMeta::default()),
            borders: CellBorders {
                start_bit: 0,
                end_bit: 0,
                start_ref: 0,
                end_ref: 2,
            },
        };
        let meta_builder = CellMetaBuilder::new(&cell_3);
        assert_eq!(meta_builder.get_refs_descriptor(3), 98);
    }

    #[test]
    fn test_bits_descriptor_d2() {
        assert_eq!(get_bits_descriptor(0), 0);
        assert_eq!(get_bits_descriptor(1023), 255);
    }

    #[test]
    fn test_hashes_and_depths() -> anyhow::Result<()> {
        let meta_builder = CellMetaBuilder::new(TonCell::empty());
        let level_mask = LevelMask::new(0);
        let (hashes, depths) = meta_builder.calc_hashes_and_depths(level_mask)?;

        for i in 0..4 {
            assert_eq!(hashes[i], TonCell::EMPTY_CELL_HASH);
            assert_eq!(depths[i], 0);
        }
        Ok(())
    }
}
