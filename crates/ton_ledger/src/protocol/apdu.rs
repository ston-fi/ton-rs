//! Short APDU commands and response status decoding.
use crate::error::{TonLedgerError, TonLedgerResult};
pub(crate) fn command(ins: u8, p1: u8, p2: u8, data: &[u8]) -> TonLedgerResult<Vec<u8>> {
    let len = u8::try_from(data.len()).map_err(|_| TonLedgerError::Invalid("APDU exceeds 255 bytes"))?;
    let mut out = vec![0xe0, ins, p1, p2, len];
    out.extend(data);
    Ok(out)
}
pub(crate) fn response(mut bytes: Vec<u8>) -> TonLedgerResult<Vec<u8>> {
    if bytes.len() < 2 {
        return Err(TonLedgerError::Response("missing status"));
    }
    let n = bytes.len();
    let sw = u16::from_be_bytes([bytes[n - 2], bytes[n - 1]]);
    bytes.truncate(n - 2);
    match sw {
        0x9000 => Ok(bytes),
        0x6985 => Err(TonLedgerError::UserDenied),
        0x6d00 => Err(TonLedgerError::UnsupportedCommand),
        0xbd00 => Err(TonLedgerError::BlindSigningDisabled),
        _ => Err(TonLedgerError::Status(sw)),
    }
}
