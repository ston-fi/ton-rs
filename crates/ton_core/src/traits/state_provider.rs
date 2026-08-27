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
mod tests {
    use super::*;
    use serde_json::json;
    use std::str::FromStr;

    #[test]
    fn test_contract_state_json_roundtrip() -> anyhow::Result<()> {
        let address = TonAddress::from_str("EQCGScrZe1xbyWqWDvdI6mzP-GAcAWFv6ZXuaJOuSqemxku4")?;
        let last_hash = TonHash::from_str("16befdc4512ca3ffaa2919e1f0d7635588edcb9fa7d3990fe83e89275c291cc7")?;
        let frozen_hash = TonHash::from_slice(&[3; 32])?;
        let state = ContractState::new(address, TxLTHash::new(64_954_068_000_009, last_hash), 123)
            .with_mc_seqno(42)
            .with_code_boc(Arc::new(vec![1, 2]))
            .with_data_boc(Arc::new(vec![3, 4]))
            .with_frozen_hash(frozen_hash);

        let json = serde_json::to_value(&state)?;
        assert_eq!(
            json,
            json!({
                "mc_seqno": 42,
                "address": "EQCGScrZe1xbyWqWDvdI6mzP-GAcAWFv6ZXuaJOuSqemxku4",
                "last_tx_id": {
                    "lt": "64954068000009",
                    "hash": "Fr79xFEso/+qKRnh8NdjVYjty5+n05kP6D6JJ1wpHMc="
                },
                "code_boc": [1, 2],
                "data_boc": [3, 4],
                "frozen_hash": "0303030303030303030303030303030303030303030303030303030303030303",
                "balance": 123
            })
        );
        assert_eq!(serde_json::from_value::<ContractState>(json)?, state);
        Ok(())
    }
}
