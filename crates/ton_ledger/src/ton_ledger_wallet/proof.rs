//! TON address proofs, locally bound to the wallet address and exact request.
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
