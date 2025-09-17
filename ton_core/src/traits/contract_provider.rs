use crate::cell::TonHash;
use crate::errors::TonCoreError;
use crate::types::{TonAddress, TxLTHash};
use async_trait::async_trait;

#[async_trait]
#[rustfmt::skip]
pub trait TonProvider: Send + Sync + 'static {
    async fn last_mc_seqno(&self) -> Result<u32, TonCoreError>;
    /// if tx_id is None, returns latest state
    async fn load_state(&self, address: TonAddress, tx_id: Option<TxLTHash>) -> Result<TonContractState, TonCoreError>;
    /// load latest blockchain config if mc_seqno is None
    async fn load_bc_config(&self, mc_seqno: Option<u32>) -> Result<Vec<u8>, TonCoreError>;
    
    async fn load_libs(&self, lib_ids: Vec<TonHash>, mc_seqno: Option<u32>) -> Result<Vec<(TonHash, Vec<u8>)>, TonCoreError>;
    
    async fn load_latest_tx_per_address(&self, mc_seqno: u32) -> Result<Vec<(TonAddress, TxLTHash)>, TonCoreError>;
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TonContractState {
    pub mc_seqno: Option<u32>,
    pub address: TonAddress,
    pub last_tx_id: TxLTHash,
    pub code_boc: Option<Vec<u8>>,
    pub data_boc: Option<Vec<u8>>,
    pub frozen_hash: Option<TonHash>,
    pub balance: i64,
}
