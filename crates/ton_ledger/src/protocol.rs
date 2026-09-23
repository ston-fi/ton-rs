use crate::error::{TonLedgerError, TonLedgerResult};
use ton::ton_core::traits::tlb::TLB;
use ton::ton_core::{
    cell::TonCell,
    types::{
        TonAddress,
        tlb_core::{MsgAddress, MsgAddressInt},
    },
};

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
pub(crate) fn coins(out: &mut Vec<u8>, value: u128) -> TonLedgerResult<()> {
    let len = (128 - value.leading_zeros()).div_ceil(8) as usize;
    if len > 15 {
        return Err(TonLedgerError::Invalid("coins exceed VarUInteger16"));
    }
    out.push(len as u8);
    out.extend(&value.to_be_bytes()[16 - len..]);
    Ok(())
}
pub(crate) fn address(out: &mut Vec<u8>, addr: &TonAddress) -> TonLedgerResult<()> {
    if !matches!(addr.workchain, 0 | -1) {
        return Err(TonLedgerError::Invalid("unsupported workchain"));
    }
    out.push(addr.workchain as u8);
    out.extend(addr.hash.as_slice());
    Ok(())
}
pub(crate) fn standard_address(addr: &MsgAddress) -> TonLedgerResult<TonAddress> {
    match addr {
        MsgAddress::Int(MsgAddressInt::Std(a)) if a.anycast.is_none() => {
            let addr = TonAddress::from_msg_address(addr.clone())?;
            if !matches!(addr.workchain, 0 | -1) {
                return Err(TonLedgerError::Invalid("unsupported workchain"));
            }
            Ok(addr)
        },
        _ => Err(TonLedgerError::Invalid("only standard non-anycast addresses are supported")),
    }
}
pub(crate) fn cell_ref(out: &mut Vec<u8>, cell: &TonCell) -> TonLedgerResult<()> {
    out.extend(cell.depth()?.to_be_bytes());
    out.extend(cell.cell_hash()?.as_slice());
    Ok(())
}
pub(crate) fn printable(bytes: &[u8], max: usize) -> TonLedgerResult<()> {
    if bytes.len() > max || bytes.iter().any(|b| !(0x20..0x7f).contains(b)) {
        return Err(TonLedgerError::Invalid("text exceeds printable ASCII limit"));
    }
    Ok(())
}
pub(crate) fn uint48(out: &mut Vec<u8>, value: u64) -> TonLedgerResult<()> {
    if value >= 1 << 48 {
        return Err(TonLedgerError::Invalid("integer exceeds 48 bits"));
    }
    out.extend(&value.to_be_bytes()[2..]);
    Ok(())
}
