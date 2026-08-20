use crate::cell::TonHash;
use crate::errors::TonCoreResult;
use crate::types::{TonAddress, TxLTHash};
use async_trait::async_trait;
use std::sync::Arc;

/// Supplies contract state and cache-invalidation deltas.
#[async_trait]
#[rustfmt::skip]
pub trait StateProvider: Send + Sync + 'static {
    async fn last_mc_seqno(&self) -> TonCoreResult<u32>;

    /// Loads the latest state or the state at the exact `tx_id`.
    /// Exact loads must return that ID in `ContractState::last_tx_id`.
    async fn load_state(&self, address: TonAddress, tx_id: Option<TxLTHash>) -> TonCoreResult<ContractState>;

    /// Loads the cache-invalidation delta from `mc_seqno - 1` to `mc_seqno`.
    ///
    /// `mc_seqno` must be nonzero. Each address occurs at most once, and
    /// repeated calls for the same sequence number return an equivalent delta.
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
