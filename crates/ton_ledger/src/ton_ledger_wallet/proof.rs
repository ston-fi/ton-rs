//! TON address proofs, locally bound to the wallet address and exact request.
/// UTF-8 domain and arbitrary payload, each limited to 128 bytes and the APDU budget.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ProofRequest {
    /// UTF-8 domain bound into the proof, at most 128 bytes.
    pub domain: String,
    /// Caller-supplied Unix timestamp in seconds.
    pub timestamp: u64,
    /// Verifier challenge, at most 128 bytes and subject to the combined APDU budget.
    pub payload: Vec<u8>,
}
impl ProofRequest {
    /// Creates a request; byte limits are checked when requesting the proof.
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
/// Locally verified TON proof returned by the wallet.
pub struct AddressProof {
    /// Ed25519 signature over `hash`.
    pub signature: [u8; 64],
    /// TON proof digest bound to the wallet address and exact request.
    pub hash: [u8; 32],
}
