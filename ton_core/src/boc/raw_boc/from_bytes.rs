use super::{RawBoC, RawCell, GENERIC_BOC_MAGIC};
use crate::bail_ton_core_data;
use crate::cell::CellType;
use crate::cell::LevelMask;
use crate::errors::TonCoreError;
use bitstream_io::{BigEndian, ByteRead, ByteReader};
use std::io::Cursor;

impl RawBoC {
    // https://github.com/ton-blockchain/ton/blob/24dc184a2ea67f9c47042b4104bbb4d82289fac1/crypto/tl/boc.tlb#L25
    pub fn from_bytes(serial: &[u8]) -> Result<RawBoC, TonCoreError> {
        let cursor = Cursor::new(serial);
        let mut reader = ByteReader::endian(cursor, BigEndian);
        let magic = reader.read::<u32>()?;

        if magic != GENERIC_BOC_MAGIC {
            bail_ton_core_data!("Unexpected magic: {magic}");
        };

        let (has_idx, has_crc32c, _has_cache_bits, size) = {
            // has_idx:(## 1) has_crc32c:(## 1) has_cache_bits:(## 1) flags:(## 2) { flags = 0 }
            let header = reader.read::<u8>()?;
            let has_idx = (header & 0b1000_0000) != 0;
            let has_crc32c = (header & 0b0100_0000) != 0;
            let has_cache_bits = (header & 0b0010_0000) != 0;

            // size:(## 3) { size <= 4 }
            let size = header & 0b0000_0111;
            if size > 4 {
                bail_ton_core_data!("Invalid BoC header: size({size}) <= 4");
            }

            (has_idx, has_crc32c, has_cache_bits, size)
        };

        //   off_bytes:(## 8) { off_bytes <= 8 }
        let off_bytes = reader.read::<u8>()?;
        if off_bytes > 8 {
            bail_ton_core_data!("Invalid BoC header: off_bytes({off_bytes}) <= 8");
        }
        //cells:(##(size * 8))
        let cells_cnt = read_var_size(&mut reader, size)?;
        //   roots:(##(size * 8)) { roots >= 1 }
        let roots_cnt = read_var_size(&mut reader, size)?;
        if roots_cnt < 1 {
            bail_ton_core_data!("Invalid BoC header: roots({roots_cnt}) >= 1");
        }
        //   absent:(##(size * 8)) { roots + absent <= cells }
        let absent = read_var_size(&mut reader, size)?;
        if roots_cnt + absent > cells_cnt {
            bail_ton_core_data!("Invalid header: roots({roots_cnt}) + absent({absent}) <= cells({cells_cnt})");
        }
        //   tot_cells_size:(##(off_bytes * 8))
        let _tot_cells_size = read_var_size(&mut reader, off_bytes)?;
        //   root_list:(roots * ##(size * 8))
        let mut roots_position = vec![];
        for _ in 0..roots_cnt {
            roots_position.push(read_var_size(&mut reader, size)?)
        }
        //   index:has_idx?(cells * ##(off_bytes * 8))
        if has_idx {
            let _idx = reader.read_to_vec(cells_cnt * off_bytes as usize)?;
        }
        //   cell_data:(tot_cells_size * [ uint8 ])
        let mut cells = Vec::with_capacity(cells_cnt);

        for _ in 0..cells_cnt {
            let cell = read_cell(&mut reader, size)?;
            cells.push(cell);
        }
        //   crc32c:has_crc32c?uint32
        let _crc32c = if has_crc32c { reader.read::<u32>()? } else { 0 };

        Ok(RawBoC { cells, roots_position })
    }
}

fn read_cell(reader: &mut ByteReader<Cursor<&[u8]>, BigEndian>, size: u8) -> Result<RawCell, TonCoreError> {
    let d1 = reader.read::<u8>()?;
    let d2 = reader.read::<u8>()?;

    let ref_num = d1 & 0b111;
    let is_exotic = (d1 & 0b1000) != 0;
    let has_hashes = (d1 & 0b10000) != 0;
    let level_mask = LevelMask::new(d1 >> 5);
    let data_size = ((d2 >> 1) + (d2 & 1)).into();
    let full_bytes = (d2 & 0x01) == 0;

    if has_hashes {
        let hash_count = level_mask.hash_count();
        let skip_size = hash_count * (32 + 2);

        // TODO: check depth and hashes
        reader.skip(skip_size as u32)?;
    }

    let mut data = reader.read_to_vec(data_size)?;

    let data_len = data.len();
    let padding_len = if data_len > 0 && !full_bytes {
        // Fix last byte,
        // see https://github.com/toncenter/tonweb/blob/c2d5d0fc23d2aec55a0412940ce6e580344a288c/src/boc/BitString.js#L302
        let num_zeros = data[data_len - 1].trailing_zeros();
        if num_zeros >= 8 {
            return Err(TonCoreError::Custom(
                "Last byte of binary must not be zero if full_byte flag is not set".to_string(),
            ));
        }
        data[data_len - 1] &= !(1 << num_zeros);
        num_zeros + 1
    } else {
        0
    };
    let data_bits_len = data.len() * 8 - padding_len as usize;
    let mut refs_positions: Vec<usize> = Vec::new();
    for _ in 0..ref_num {
        refs_positions.push(read_var_size(reader, size)?);
    }

    let cell_type = match is_exotic {
        true => {
            if data.is_empty() {
                bail_ton_core_data!("Exotic cell must have at least 1 byte");
            }
            CellType::new_exotic(data[0])?
        }
        false => CellType::Ordinary,
    };

    let cell = RawCell {
        cell_type,
        data,
        data_bits_len,
        refs_positions,
        level_mask,
    };
    Ok(cell)
}

fn read_var_size(reader: &mut ByteReader<Cursor<&[u8]>, BigEndian>, n: u8) -> Result<usize, TonCoreError> {
    let bytes = reader.read_to_vec(n.into())?;

    let mut result = 0;
    for &byte in &bytes {
        result <<= 8;
        result |= usize::from(byte);
    }
    Ok(result)
}
