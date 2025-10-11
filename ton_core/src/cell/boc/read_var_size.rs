use crate::cell::ton_cell::CellBytesReader;
use crate::errors::TonCoreError;
use bitstream_io::ByteRead;

const MAX_LEN_BYTES: usize = (usize::BITS / 8) as usize;

pub(super) fn read_var_size(reader: &mut CellBytesReader, bytes_len: u8) -> Result<usize, TonCoreError> {
    let mut bytes = [0u8; MAX_LEN_BYTES];
    let read_offset = MAX_LEN_BYTES - bytes_len as usize;
    reader.read_bytes(&mut bytes[read_offset..])?;

    let mut res = 0usize;
    for pos in 0..bytes_len as usize {
        res <<= 8;
        res |= bytes[read_offset + pos] as usize;
    }
    Ok(res)
}
