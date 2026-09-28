use crate::bail_ton_core_data;
use crate::bits_utils::BitsUtils;
use crate::cell::boc::boc_reader::BocReader;
use crate::cell::ton_cell::{CellBitWriter, CellData, RefStorage};
use crate::cell::{CellBorders, CellMeta, CellType, HashesDepthsStorage, LevelMask, TonCell, TonHash};
use crate::errors::TonCoreError;
use bitstream_io::BitWrite;
use once_cell;
use once_cell::sync::OnceCell;
use smallvec::SmallVec;
use std::sync::Arc;

/// References are stored as indices in BagOfCells.
#[derive(PartialEq, Debug, Clone)]
pub(crate) struct RawCell {
    pub(crate) cell_type: CellType,
    pub(crate) data_storage: Arc<Vec<u8>>,
    pub(crate) start_bit: usize,
    pub(crate) end_bit: usize,
    pub(crate) refs_pos: RefPosStorage,
    pub(crate) level_mask: LevelMask,
    pub(crate) hashes_depths: Option<HashesDepthsStorage>,
}

pub(crate) type RefPosStorage = SmallVec<[usize; TonCell::MAX_REFS_COUNT]>;

impl RawCell {
    pub(crate) fn data_len_bits(&self) -> usize {
        self.end_bit - self.start_bit
    }
    pub(crate) fn data_len_bytes(&self) -> usize {
        self.data_len_bits().div_ceil(8)
    }
    pub(crate) fn size_in_boc_bytes(&self, ref_size_bytes: u32) -> u32 {
        2 + self.data_len_bytes() as u32 + self.refs_pos.len() as u32 * ref_size_bytes
    }

    pub(crate) fn write_to(&self, writer: &mut CellBitWriter, ref_size_bytes: u32) -> std::io::Result<()> {
        let level = self.level_mask;
        let is_exotic = self.cell_type.is_exotic() as u32;
        let num_refs = self.refs_pos.len() as u32;
        let data_len_bits = self.data_len_bits();
        let data_len_bytes = self.data_len_bytes();

        let d1 = num_refs + is_exotic * 8 + level.mask() as u32 * 32;

        let is_bytes_aligned = data_len_bits.is_multiple_of(8);
        // data_len_bytes <= 128 by spec (128*2 <= 256), but d2 must be u8 (0-255) by spec as well ¯\_(ツ)_/¯
        let d2 = (data_len_bytes * 2 - if is_bytes_aligned { 0 } else { 1 }) as u8; // subtract 1 if the last byte is not full

        writer.write_bytes(&[d1 as u8, d2])?;

        let full_bytes = self.data_len_bits() / 8;
        let mut data = vec![0; full_bytes + 1]; // TODO use something better then Vec
        BitsUtils::read_with_offset(&self.data_storage, &mut data, self.start_bit, self.data_len_bits());
        writer.write_bytes(&data[0..full_bytes])?;
        if !is_bytes_aligned {
            // https://github.com/ton-blockchain/ton/blob/05bea13375448a401d8e07c6132b7f709f5e3a32/crypto/vm/cells/DataCell.cpp#L362
            let rest_bits_len = self.data_len_bits() % 8;
            let mut last_byte = data[full_bytes];
            last_byte >>= 7 - rest_bits_len;
            last_byte |= 1;
            last_byte <<= 7 - rest_bits_len;
            writer.write_var(8, last_byte)?;
        }

        for refs_pos in &self.refs_pos {
            writer.write_var(8 * ref_size_bytes, *refs_pos as u32)?;
        }

        Ok(())
    }

    /// Reads one serialized cell. Reference indices are checked when the cell tree is built.
    pub(super) fn read(
        reader: &mut BocReader,
        ref_pos_size_bytes: usize,
        data_storage: &Arc<Vec<u8>>,
    ) -> Result<Self, TonCoreError> {
        let d1 = reader.read_u8()?;
        let d2 = reader.read_u8()?;

        let refs_count = (d1 & 0b111) as usize;
        let is_exotic = (d1 & 0b1000) != 0;
        let has_hashes = (d1 & 0b10000) != 0;
        let level_mask = LevelMask::new(d1 >> 5);
        let full_bytes = (d2 & 0x01) == 0;
        let data_len_bytes = ((d2 >> 1) + (d2 & 1)) as usize;

        // 5 and 6 are invalid, 7 marks an absent cell: neither can be represented as TonCell.
        if refs_count > TonCell::MAX_REFS_COUNT {
            bail_ton_core_data!("Invalid cell: {refs_count} refs, at most {} allowed", TonCell::MAX_REFS_COUNT);
        }

        // Stored hashes and depths are not trusted: they are recomputed from the cell tree on demand.
        if has_hashes {
            reader.take(level_mask.hash_count() * (TonHash::BYTES_LEN + CellMeta::DEPTH_BYTES))?;
        }

        let start_bit = reader.position() * 8;
        let data = reader.take(data_len_bytes)?;

        let data_len_bits = match data.last() {
            // The last byte ends with a completion tag: a 1 bit followed by zeros.
            // A zero byte has no tag and 0x80 holds no data bits, so both are rejected like in the TON node.
            Some(&last_byte) if !full_bytes => {
                if last_byte & 0x7f == 0 {
                    bail_ton_core_data!("Invalid cell: bad completion tag in the last data byte {last_byte:#04x}");
                }
                data_len_bytes * 8 - last_byte.trailing_zeros() as usize - 1
            },
            _ => data_len_bytes * 8,
        };

        let cell_type = match (is_exotic, data.first()) {
            (false, _) => CellType::Ordinary,
            // Every exotic cell layout is a whole number of bytes.
            (true, _) if !full_bytes => bail_ton_core_data!("Exotic cell data must be byte-aligned"),
            (true, Some(&type_byte)) => CellType::new_exotic(type_byte)?,
            (true, None) => bail_ton_core_data!("Exotic cell must have at least 1 byte"),
        };

        let mut refs_pos = RefPosStorage::with_capacity(refs_count);
        for _ in 0..refs_count {
            refs_pos.push(reader.read_uint(ref_pos_size_bytes)? as usize);
        }

        Ok(RawCell {
            cell_type,
            data_storage: data_storage.clone(),
            start_bit,
            end_bit: start_bit + data_len_bits,
            refs_pos,
            level_mask,
            hashes_depths: None,
        })
    }

    pub(crate) fn into_ton_cell(self, refs: RefStorage) -> TonCell {
        let end_ref = refs.len() as u8;
        let hashes_depths = match self.hashes_depths {
            Some(value) => OnceCell::with_value(value),
            None => OnceCell::default(),
        };
        TonCell {
            cell_type: self.cell_type,
            cell_data: Arc::new(CellData {
                data_storage: self.data_storage,
                refs,
            }),
            borders: CellBorders {
                start_bit: self.start_bit,
                end_bit: self.end_bit,
                start_ref: 0,
                end_ref,
            },
            meta: Arc::new(CellMeta {
                level_mask: self.level_mask.into(),
                hashes_depths,
            }),
        }
    }
}
