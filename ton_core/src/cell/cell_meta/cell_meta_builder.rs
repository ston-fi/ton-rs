use crate::bail_ton_core_data;
use crate::bits_utils::BitsUtils;
use crate::cell::cell_meta::cell_type::CellType;
use crate::cell::cell_meta::level_mask::LevelMask;
use crate::cell::cell_meta::HashesDepthsStorage;
use crate::cell::ton_cell::{CellBitWriter, TonCell};
use crate::cell::ton_hash::TonHash;
use crate::cell::CellMeta;
use crate::errors::TonCoreError;
use bitstream_io::{BigEndian, BitWrite, BitWriter, ByteRead, ByteReader};
use sha2::{Digest, Sha256};
use smallvec::SmallVec;
use std::io::Cursor;

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
            is_byte_aligned: start_bit % 8 == 0,
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
        if self.data_len_bits < 16 {
            bail_ton_core_data!("Pruned Branch require at least 16 bits data");
        }

        if self.is_config_proof() {
            return Ok(());
        }

        let level_mask = self.calc_level_mask_pruned();

        if level_mask <= LevelMask::MIN_LEVEL || level_mask > LevelMask::MAX_LEVEL {
            bail_ton_core_data!("Pruned Branch cell level must in range [1, 3] (got {level_mask})");
        }

        let expected_size = (2 + level_mask.apply(level_mask.level() - 1u8).hash_count()
            * (TonHash::BYTES_LEN + CellMeta::DEPTH_BYTES))
            * 8;

        if self.data_len_bits != expected_size {
            bail_ton_core_data!("PrunedBranch must have exactly {expected_size} bits, got {}", self.data_len_bits);
        }

        Ok(())
    }

    fn validate_library(&self) -> Result<(), TonCoreError> {
        const LIB_CELL_BITS_LEN: usize = (1 + TonHash::BYTES_LEN) * 8;

        if self.data_len_bits != LIB_CELL_BITS_LEN {
            bail_ton_core_data!("Lib cell must have exactly {LIB_CELL_BITS_LEN} bits, got {}", self.data_len_bits);
        }

        Ok(())
    }

    fn validate_merkle_proof(&self) -> Result<(), TonCoreError> {
        // type + hash + depth
        const MERKLE_PROOF_BITS_LEN: usize = (1 + TonHash::BYTES_LEN + CellMeta::DEPTH_BYTES) * 8;

        if self.data_len_bits != MERKLE_PROOF_BITS_LEN {
            bail_ton_core_data!(
                "MerkleProof must have exactly {MERKLE_PROOF_BITS_LEN} bits, got {}",
                self.data_len_bits
            );
        }

        if self.refs.len() != 1 {
            bail_ton_core_data!("Merkle Proof cell must have exactly 1 ref");
        }
        if self.is_byte_aligned {
            let data = &self.data[self.start_bit / 8..];
            validate_merkle_proof_slice(data)?;
        } else {
            let mut data = vec![0; self.data_len_bits.div_ceil(8)];
            BitsUtils::read_with_offset(self.data, &mut data, self.start_bit, self.data_len_bits);
            validate_merkle_proof_slice(&data)?;
        };
        Ok(())
    }

    fn validate_merkle_update(&self) -> Result<(), TonCoreError> {
        // type + hash + hash + depth + depth
        // const MERKLE_UPDATE_BITS_LEN: usize = 8 + (2 * (256 + 16));
        log::trace!("validate_merkle_update is not implemented yet"); // TODO
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
        let mut data = vec![0; self.data_len_bits.div_ceil(8)];
        BitsUtils::read_with_offset(self.data, &mut data, self.start_bit, self.data_len_bits);

        match self.is_config_proof() {
            true => LevelMask::new(1),
            false => LevelMask::new(data[1]),
        }
    }

    fn calc_level_mask_merkle_update(&self) -> LevelMask {
        let refs_lm = self.refs[0].level_mask() | self.refs[1].level_mask();
        refs_lm >> 1
    }

    fn is_config_proof(&self) -> bool {
        const CONFIG_PROOF_DATA_LEN_BITS: usize = 200;
        self.cell_type == CellType::PrunedBranch && self.data_len_bits == CONFIG_PROOF_DATA_LEN_BITS
    }

    /// This function replicates unknown logic of resolving cell data
    /// https://github.com/ton-blockchain/ton/blob/24dc184a2ea67f9c47042b4104bbb4d82289fac1/crypto/vm/cells/DataCell.cpp#L214
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
                max_ref_depth + 1
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
    /// See https://docs.ton.org/tvm.pdf 3.1.4 for details
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

        for i in 0..4 {
            let hash_index = level_mask.apply(i).hash_index();

            let (hash, depth) = match self.cell_type {
                CellType::PrunedBranch => {
                    let this_hash_index = level_mask.hash_index();
                    if hash_index != this_hash_index {
                        let pruned = self.calc_pruned_hash_depth(level_mask)?;
                        (pruned[hash_index].hash.clone(), pruned[hash_index].depth)
                    } else {
                        (hashes[0].clone(), depths[0])
                    }
                }
                _ => (hashes[hash_index].clone(), depths[hash_index]),
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

    fn calc_pruned_hash_depth(&self, level_mask: LevelMask) -> Result<Vec<Pruned>, TonCoreError> {
        let current_index = if self.is_config_proof() { 1 } else { 2 };

        // TODO find a way to avoid allocation
        let mut data = vec![0; self.data_len_bits.div_ceil(8)];
        BitsUtils::read_with_offset(self.data, &mut data, self.start_bit, self.data_len_bits);

        let cursor = Cursor::new(&data[current_index..]);
        let mut reader = ByteReader::endian(cursor, BigEndian);

        let level = level_mask.level() as usize;
        let hashes = (0..level).map(|_| reader.read::<[u8; TonHash::BYTES_LEN]>()).collect::<Result<Vec<_>, _>>()?;
        let depths = (0..level).map(|_| reader.read::<u16>()).collect::<Result<Vec<_>, _>>()?;
        let result = hashes
            .into_iter()
            .zip(depths)
            .map(|(hash, depth)| Pruned {
                hash: hash.into(),
                depth,
            })
            .collect();

        Ok(result)
    }
}

/// Calculates d2 descriptor for cell
/// See https://docs.ton.org/tvm.pdf 3.1.4 for details
fn get_bits_descriptor(data_bits_len: usize) -> u8 { (data_bits_len / 8 + data_bits_len.div_ceil(8)) as u8 }

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

fn validate_merkle_proof_slice(data: &[u8]) -> Result<(), TonCoreError> {
    let mut data_slice = &data[1..];
    let _proof_hash = match TonHash::from_slice(&data_slice[..TonHash::BYTES_LEN]) {
        Ok(hash) => hash,
        Err(err) => bail_ton_core_data!("Can't parse proof hash from cell data: {err}"),
    };

    data_slice = &data_slice[TonHash::BYTES_LEN..];
    let _proof_depth = u16::from_be_bytes(data_slice[..CellMeta::DEPTH_BYTES].try_into().unwrap());
    log::trace!("validate_merkle_proof is not implemented yet!"); // TODO
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::cell::ton_cell::{CellBorders, CellData, RefStorage};
    use std::sync::Arc;

    fn empty_cell_ref() -> TonCell { TonCell::empty().to_owned() }

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
