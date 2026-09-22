//! Legacy Ledger data signatures. This is not TON Connect signData.
use crate::{
    error::{TonLedgerError, TonLedgerResult},
    protocol,
};
use ton::ton_core::{cell::TonCell, traits::tlb::TLB, types::TonAddress};
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum LedgerDataRequest {
    Plaintext(String),
    /// The device displays the address/domain and hashes of data and extension.
    AppData {
        address: Option<TonAddress>,
        domain: Option<String>,
        data: TonCell,
        extension: Option<TonCell>,
    },
}
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct SignedData {
    pub signature: [u8; 64],
    pub cell_hash: [u8; 32],
    pub schema: u32,
    pub timestamp: u64,
}
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
            protocol::printable(text.as_bytes(), 120)?;
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
                protocol::address(&mut d, a)?;
                a.to_msg_address_int().write(&mut b)?;
            }
            d.push(u8::from(domain.is_some()));
            b.write_bit(domain.is_some())?;
            if let Some(domain) = domain {
                protocol::printable(domain.as_bytes(), 126)?;
                d.push(domain.len() as u8);
                d.extend(domain.as_bytes());
                let mut db = TonCell::builder();
                for label in domain.rsplit('.') {
                    db.write_bits(label.as_bytes(), label.len() * 8)?;
                    db.write_num(&0u8, 8)?;
                }
                b.write_ref(db.build()?)?;
            }
            protocol::cell_ref(&mut d, data)?;
            b.write_ref(data.clone())?;
            d.push(u8::from(extension.is_some()));
            b.write_bit(extension.is_some())?;
            if let Some(ext) = extension {
                protocol::cell_ref(&mut d, ext)?;
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
