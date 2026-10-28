use crate::cell::TonHash;
use crate::errors::TonCoreResult;
use crate::types::{TonAddress, TxLTHash};
use async_trait::async_trait;
use derive_setters::Setters;
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

/// Contract state returned by a [`StateProvider`].
/// Implements Serde when the `serde` feature is enabled.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Setters)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[setters(prefix = "with_", strip_option)]
#[non_exhaustive]
pub struct ContractState {
    /// Masterchain sequence number associated with the snapshot.
    pub mc_seqno: Option<u32>,
    /// Contract address.
    pub address: TonAddress,
    /// Last transaction represented by the snapshot.
    #[cfg_attr(feature = "serde", serde(with = "crate::serde::serde_tx_lt_hash_json"))]
    pub last_tx_id: TxLTHash,
    /// Contract code serialized as BOC.
    pub code_boc: Option<Arc<Vec<u8>>>,
    /// Contract data serialized as BOC.
    pub data_boc: Option<Arc<Vec<u8>>>,
    /// Frozen-state hash, when the account is frozen.
    pub frozen_hash: Option<TonHash>,
    /// Account balance in nanotons.
    pub balance: i64,
}

impl ContractState {
    /// Creates a snapshot with no masterchain, code, data, or frozen-state metadata.
    pub fn new(address: TonAddress, last_tx_id: TxLTHash, balance: i64) -> Self {
        Self {
            mc_seqno: None,
            address,
            last_tx_id,
            code_boc: None,
            data_boc: None,
            frozen_hash: None,
            balance,
        }
    }
}

#[cfg(all(test, feature = "serde"))]
#[path = "../tests/traits/test_state_provider.rs"]
mod tests;
