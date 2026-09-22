//! TON address proofs, locally bound to the wallet address and exact request.
use sha2::{Digest, Sha256};
use ton::ton_core::types::TonAddress;
/// UTF-8 domain and arbitrary payload, each limited to 128 bytes and the APDU budget.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ProofRequest {
    pub domain: String,
    pub timestamp: u64,
    pub payload: Vec<u8>,
}
impl ProofRequest {
    pub fn new(domain: String, timestamp: u64, payload: Vec<u8>) -> Self {
        Self {
            domain,
            timestamp,
            payload,
        }
    }
}
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct AddressProof {
    pub signature: [u8; 64],
    pub hash: [u8; 32],
}
pub(crate) fn digest(address: &TonAddress, req: &ProofRequest) -> [u8; 32] {
    let mut inner = Sha256::new();
    inner.update(b"ton-proof-item-v2/");
    inner.update(address.workchain.to_be_bytes());
    inner.update(address.hash.as_slice());
    inner.update((req.domain.len() as u32).to_le_bytes());
    inner.update(req.domain.as_bytes());
    inner.update(req.timestamp.to_le_bytes());
    inner.update(&req.payload);
    let mut outer = Sha256::new();
    outer.update(b"\xff\xffton-connect");
    outer.update(inner.finalize());
    outer.finalize().into()
}
