use crate::cell::TonHash;
use crate::errors::TonCoreResult;
use crate::types::{TonAddress, TxLTHash};
use async_trait::async_trait;
use std::sync::Arc;

/// Loads contract state and the per-masterchain transaction deltas used to
/// invalidate state caches.
///
/// Implementations must support repeated calls for the same sequence number.
#[async_trait]
#[rustfmt::skip]
pub trait StateProvider: Send + Sync + 'static {
    /// Returns the latest masterchain sequence number whose transaction delta
    /// is ready to be loaded.
    async fn last_mc_seqno(&self) -> TonCoreResult<u32>;
    /// Loads the state at the exact transaction when `tx_id` is specified, or
    /// the latest state otherwise.
    ///
    /// For an exact transaction, the returned state's `last_tx_id` must equal
    /// `tx_id`.
    async fn load_state(&self, address: TonAddress, tx_id: Option<TxLTHash>) -> TonCoreResult<ContractState>;
    /// Returns one latest transaction per address changed between masterchain
    /// states `mc_seqno - 1` and `mc_seqno`.
    ///
    /// The result is the invalidation delta for this sequence number, not a
    /// snapshot of all known addresses. `mc_seqno` must be greater than zero,
    /// and each address must occur at most once. Implementations may wait until
    /// the block is available or return an error; the contract client retries
    /// every error. Repeated successful calls must return an equivalent delta.
    async fn load_latest_tx_per_address(&self, mc_seqno: u32) -> TonCoreResult<Vec<(TonAddress, TxLTHash)>>;
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ContractState {
    pub mc_seqno: Option<u32>,
    pub address: TonAddress,
    pub last_tx_id: TxLTHash,
    pub code_boc: Option<Arc<Vec<u8>>>,
    pub data_boc: Option<Arc<Vec<u8>>>,
    pub frozen_hash: Option<TonHash>,
    pub balance: i64,
}
