use crate::errors::TonCoreError;
use bitstream_io::{BigEndian, ByteRead, ByteReader};
use std::io::Cursor;

pub(super) fn read_var_size(
    reader: &mut ByteReader<Cursor<&[u8]>, BigEndian>,
    bytes_len: u8,
) -> Result<usize, TonCoreError> {
    let bytes = reader.read_to_vec(bytes_len.into())?;

    let mut result = 0;
    for &byte in &bytes {
        result <<= 8;
        result |= usize::from(byte);
    }
    Ok(result)
}
