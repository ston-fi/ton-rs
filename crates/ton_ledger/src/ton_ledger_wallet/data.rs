//! Legacy Ledger data requests and signatures, not TON Connect signData.
use ton::ton_core::{cell::TonCell, types::TonAddress};
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
