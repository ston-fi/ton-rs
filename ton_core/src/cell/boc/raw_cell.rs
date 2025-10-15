use crate::bail_ton_core_data;
use crate::bits_utils::BitsUtils;
use crate::cell::boc::read_var_size::read_var_size;
use crate::cell::ton_cell::{CellBitWriter, CellBytesReader, CellData, RefStorage};
use crate::cell::{CellBorders, CellMeta, CellType, LevelMask, TonCell};
use crate::errors::TonCoreError;
use bitstream_io::{BitWrite, ByteRead};
use smallvec::SmallVec;
use std::sync::Arc;

/// References are stored as indices in BagOfCells.
#[derive(PartialEq, Debug, Clone)]
pub(super) struct RawCell {
    pub(super) cell_type: CellType,
    pub(super) data_storage: Arc<Vec<u8>>,
    pub(super) start_bit: usize,
    pub(super) end_bit: usize,
    pub(super) refs_pos: RefPosStorage,
    pub(super) level_mask: LevelMask,
}

pub(super) type RefPosStorage = SmallVec<[usize; TonCell::MAX_REFS_COUNT]>;

impl RawCell {
    pub(super) fn data_len_bits(&self) -> usize { self.end_bit - self.start_bit }
    pub(super) fn data_len_bytes(&self) -> usize { self.data_len_bits().div_ceil(8) }
    pub(super) fn size_in_boc_bytes(&self, ref_size_bytes: u32) -> u32 {
        2 + self.data_len_bytes() as u32 + self.refs_pos.len() as u32 * ref_size_bytes
    }

    pub(super) fn write_to(&self, writer: &mut CellBitWriter, ref_size_bytes: u32) -> std::io::Result<()> {
        let level = self.level_mask;
        let is_exotic = self.cell_type.is_exotic() as u32;
        let num_refs = self.refs_pos.len() as u32;
        let data_len_bits = self.data_len_bits();
        let data_len_bytes = self.data_len_bytes();

        let d1 = num_refs + is_exotic * 8 + level.mask() as u32 * 32;

        let is_bytes_aligned = (data_len_bits % 8) == 0;
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

    pub(super) fn read(
        reader: &mut CellBytesReader,
        ref_pos_size_bytes: u8,
        data_storage: Arc<Vec<u8>>,
    ) -> Result<Self, TonCoreError> {
        let mut descriptors = [0u8; 2];
        reader.read_bytes(&mut descriptors)?;
        let (d1, d2) = (descriptors[0], descriptors[1]);

        let refs_count = d1 & 0b111;
        let is_exotic = (d1 & 0b1000) != 0;
        let has_hashes = (d1 & 0b10000) != 0;
        let level_mask = LevelMask::new(d1 >> 5);
        let full_bytes = (d2 & 0x01) == 0;
        let data_len_bytes = ((d2 >> 1) + (d2 & 1)) as usize;

        // TODO: check or save depths and hashes if provided?
        if has_hashes {
            let hash_count = level_mask.hash_count();
            let skip_size = hash_count * (32 + 2);
            reader.skip(skip_size as u32)?;
        }

        let start_bit = reader.reader().position() as usize * 8;

        let cell_type = match is_exotic {
            true if data_len_bytes == 0 => bail_ton_core_data!("Exotic cell must have at least 1 byte"),
            true => CellType::new_exotic(reader.read::<u8>()?)?,
            false => CellType::Ordinary,
        };

        // we need to read last byte to get padding info,
        // if it's exotic, we already took 1 byte.
        let mut data_len_bytes_left = data_len_bytes;
        if is_exotic {
            data_len_bytes_left -= 1;
        }

        let padding_len_bits = if data_len_bytes_left > 0 && !full_bytes {
            reader.skip(data_len_bytes_left as u32 - 1)?; // can skip 0
            let last_byte = reader.read::<u8>()?;
            let num_zeros = last_byte.trailing_zeros();
            if num_zeros >= 8 {
                bail_ton_core_data!("Last byte can't be zero if full_byte flag is not set");
            }
            num_zeros + 1
        } else {
            // not interesting in last byte, skipp all the rest
            reader.skip(data_len_bytes_left as u32)?; // can skip 0
            0
        };

        let data_len_bits = data_len_bytes * 8 - padding_len_bits as usize;
        let end_bit = start_bit + data_len_bits;

        let mut refs_pos = RefPosStorage::with_capacity(refs_count as usize);
        for _ in 0..refs_count {
            refs_pos.push(read_var_size(reader, ref_pos_size_bytes)?);
        }

        let cell = RawCell {
            cell_type,
            data_storage,
            start_bit,
            end_bit,
            refs_pos,
            level_mask,
        };
        Ok(cell)
    }

    pub(super) fn into_ton_cell(self, refs: RefStorage) -> TonCell {
        let end_ref = refs.len() as u8;
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
                hashes_depths: Default::default(),
            }),
        }
    }
}
