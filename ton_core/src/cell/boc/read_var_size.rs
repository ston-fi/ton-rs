use crate::cell::boc::raw_boc::BocBytesReader;
use crate::errors::TonCoreError;
use bitstream_io::ByteRead;

pub(super) fn read_var_size(reader: &mut BocBytesReader, bytes_len: u8) -> Result<usize, TonCoreError> {
    let bytes = reader.read_to_vec(bytes_len.into())?;

    let mut result = 0;
    for &byte in &bytes {
        result <<= 8;
        result |= usize::from(byte);
    }
    Ok(result)
}
