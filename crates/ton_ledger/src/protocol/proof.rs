//! TON proof digest bound to the wallet address and request.
use crate::ton_ledger_wallet::proof::ProofRequest;
use sha2::{Digest, Sha256};
use ton::ton_core::types::TonAddress;
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
