//! Legacy Ledger data encoding and signature preimages.
use super::encoding;
use crate::{
    error::{TonLedgerError, TonLedgerResult},
    ton_ledger_wallet::data::LedgerDataRequest,
};
use ton::ton_core::{cell::TonCell, traits::tlb::TLB};
pub(crate) struct EncodedData {
    pub(crate) apdu: Vec<u8>,
    pub(crate) preimage: Vec<u8>,
    pub(crate) schema: u32,
    pub(crate) hash: [u8; 32],
}
pub(crate) fn encode(req: &LedgerDataRequest, timestamp: u64) -> TonLedgerResult<EncodedData> {
    let mut b = TonCell::builder();
    let mut d = Vec::new();
    let schema: u32 = match req {
        LedgerDataRequest::Plaintext(text) => {
            encoding::printable(text.as_bytes(), 120)?;
            b.write_bits(text.as_bytes(), text.len() * 8)?;
            d.extend(text.as_bytes());
            0x754bf91b
        },
        LedgerDataRequest::AppData {
            address,
            domain,
            data,
            extension,
        } => {
            if address.is_none() && domain.is_none() {
                return Err(TonLedgerError::Invalid("app data requires address or domain"));
            }
            d.push(u8::from(address.is_some()));
            b.write_bit(address.is_some())?;
            if let Some(a) = address {
                encoding::address(&mut d, a)?;
                a.to_msg_address_int().write(&mut b)?;
            }
            d.push(u8::from(domain.is_some()));
            b.write_bit(domain.is_some())?;
            if let Some(domain) = domain {
                encoding::printable(domain.as_bytes(), 126)?;
                d.push(domain.len() as u8);
                d.extend(domain.as_bytes());
                let mut db = TonCell::builder();
                for label in domain.rsplit('.') {
                    db.write_bits(label.as_bytes(), label.len() * 8)?;
                    db.write_num(&0u8, 8)?;
                }
                b.write_ref(db.build()?)?;
            }
            encoding::cell_ref(&mut d, data)?;
            b.write_ref(data.clone())?;
            d.push(u8::from(extension.is_some()));
            b.write_bit(extension.is_some())?;
            if let Some(ext) = extension {
                encoding::cell_ref(&mut d, ext)?;
                b.write_ref(ext.clone())?;
            }
            0x54b58535
        },
    };
    let hash: [u8; 32] =
        b.build()?.cell_hash()?.as_slice().try_into().map_err(|_| TonLedgerError::Invalid("hash size"))?;
    let mut prefix = schema.to_be_bytes().to_vec();
    prefix.extend(timestamp.to_be_bytes());
    let mut apdu = prefix.clone();
    apdu.extend(d);
    prefix.extend(hash);
    Ok(EncodedData {
        apdu,
        preimage: prefix,
        schema,
        hash,
    })
}
