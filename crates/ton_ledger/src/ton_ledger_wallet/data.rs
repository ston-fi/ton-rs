//! Legacy Ledger data requests and signatures, not TON Connect signData.
use ton::ton_core::{cell::TonCell, types::TonAddress};
#[derive(Debug, Clone)]
#[non_exhaustive]
/// Legacy data schema to approve; encoded and validated before device I/O.
pub enum LedgerDataRequest {
    /// At most 120 printable ASCII bytes.
    Plaintext(String),
    /// The device displays the address/domain and hashes of data and extension.
    AppData {
        /// Optional standard workchain 0/-1 address; address or domain is required.
        address: Option<TonAddress>,
        /// Optional printable ASCII domain, at most 126 bytes.
        domain: Option<String>,
        /// Data cell whose hash and depth are supplied to the device.
        data: TonCell,
        /// Optional extension cell, displayed by hash.
        extension: Option<TonCell>,
    },
}
#[derive(Debug, Clone)]
#[non_exhaustive]
/// Verified legacy signature and the fields needed to reconstruct its preimage.
pub struct SignedData {
    /// Ed25519 signature over schema BE32, timestamp BE64 and cell hash.
    pub signature: [u8; 64],
    /// Hash of the schema-specific data cell.
    pub cell_hash: [u8; 32],
    /// Firmware schema tag identifying plaintext or app data.
    pub schema: u32,
    /// Caller-supplied Unix timestamp in seconds.
    pub timestamp: u64,
}
