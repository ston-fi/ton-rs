use crate::block_tlb::BlockIdExt;
use crate::clients::tl_client::tl::client::TLClientTrait;
use crate::clients::tl_client::{TLClient, TLConnection};
use crate::errors::TonError;
use async_recursion::async_recursion;
use async_trait::async_trait;
use futures_util::future::try_join_all;
use moka::future::Cache;
use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;
use ton_lib_core::cell::TonHash;
use ton_lib_core::errors::TonCoreError;
use ton_lib_core::traits::contract_provider::{TonContractState, TonProvider};
use ton_lib_core::types::{TonAddress, TxLTHash};

static BLOCK_IDS_CACHE_SIZE: u64 = 100;

pub struct TLProvider {
    client: TLClient,
    mc_block_cache: Cache<u32, BlockIdExt>, // mc_seqno -> block_id
    block_shards_cache: Cache<u32, Arc<HashSet<BlockIdExt>>>, // mc_seqno -> shards, must keep it separately from unseen_cache for proper checks
    unseen_cache: Cache<u32, Arc<HashSet<BlockIdExt>>>,
}

impl TLProvider {
    pub fn new(client: TLClient) -> Self {
        Self {
            client,
            mc_block_cache: Cache::new(BLOCK_IDS_CACHE_SIZE),
            block_shards_cache: Cache::new(BLOCK_IDS_CACHE_SIZE),
            unseen_cache: Cache::new(BLOCK_IDS_CACHE_SIZE),
        }
    }
}

#[async_trait]
impl TonProvider for TLProvider {
    async fn last_mc_seqno(&self) -> Result<u32, TonCoreError> { Ok(self.client.get_mc_info().await?.last.seqno) }

    async fn load_state(&self, address: TonAddress, tx_id: Option<TxLTHash>) -> Result<TonContractState, TonCoreError> {
        let raw_state = match tx_id {
            Some(id) => self.client.get_account_state_raw_by_tx(address.clone(), id).await,
            None => self.client.get_account_state_raw(address.clone()).await,
        }?;

        let code_boc = Some(raw_state.code).filter(|x| !x.is_empty());
        let data_boc = Some(raw_state.data).filter(|x| !x.is_empty());
        let frozen_hash = match raw_state.frozen_hash.is_empty() {
            true => None,
            false => Some(TonHash::from_vec(raw_state.frozen_hash)?),
        };
        Ok(TonContractState {
            mc_seqno: None,
            address,
            last_tx_id: raw_state.last_tx_id,
            code_boc,
            data_boc,
            frozen_hash,
            balance: raw_state.balance,
        })
    }

    async fn load_bc_config(&self, _mc_seqno: Option<u32>) -> Result<Vec<u8>, TonCoreError> {
        Ok(self.client.get_config_boc_all(0).await?)
    }

    async fn load_libs(
        &self,
        lib_ids: Vec<TonHash>,
        _mc_seqno: Option<u32>,
    ) -> Result<Vec<(TonHash, Vec<u8>)>, TonCoreError> {
        let libs_raw = self.client.get_libs(lib_ids).await?;
        let mut libs = Vec::with_capacity(libs_raw.len());
        for lib in libs_raw {
            libs.push((TonHash::from_vec(lib.hash)?, lib.data));
        }
        Ok(libs)
    }

    async fn load_latest_tx_per_address(&self, mc_seqno: u32) -> Result<Vec<(TonAddress, TxLTHash)>, TonCoreError> {
        let conn = self.find_connection(mc_seqno).await?;
        let prev_mc_block = self.get_or_load_master(conn, mc_seqno - 1).await?;
        let prev_shards = self.get_or_load_shards(conn, &prev_mc_block).await?;

        let cur_mc_block = self.get_or_load_master(conn, mc_seqno).await?;
        let cur_shards = self.get_or_load_shards(conn, &cur_mc_block).await?;

        let unseen_shards = self.get_or_load_unseen(conn, mc_seqno, &prev_shards, cur_shards.deref().clone()).await?;
        let txs_futs = unseen_shards.deref().iter().chain([&cur_mc_block]).map(|block_id| async {
            let res = self
                .client
                .get_block_txs(block_id)
                .await?
                .into_iter()
                .map(|x| {
                    (TxLTHash::new(x.lt, x.tx_hash), TonAddress::new(block_id.shard_ident.workchain, x.address_hash))
                })
                .collect::<Vec<_>>();
            Ok::<_, TonCoreError>(res)
        });
        let block_txs = try_join_all(txs_futs).await?;

        let mut latest_by_address = HashMap::<TonAddress, TxLTHash>::new();
        for txs in block_txs {
            for (tx_id, address) in txs {
                match latest_by_address.get_mut(&address) {
                    Some(cur_id) => {
                        if cur_id.lt < tx_id.lt {
                            *cur_id = tx_id;
                        };
                    }
                    None => {
                        latest_by_address.insert(address, tx_id);
                    }
                }
            }
        }
        Ok(latest_by_address.into_iter().collect())
    }
}

impl TLProvider {
    async fn find_connection(&self, mc_seqno: u32) -> Result<&TLConnection, TonError> {
        loop {
            let conn = self.client.get_connection();
            let mc_info = conn.get_mc_info().await?;
            if mc_info.last.seqno >= mc_seqno {
                return Ok(conn);
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    async fn get_or_load_master(&self, conn: &TLConnection, mc_seqno: u32) -> Result<BlockIdExt, TonError> {
        Ok(self.mc_block_cache.try_get_with(mc_seqno, async move { Ok(conn.lookup_mc_block(mc_seqno).await?) }).await?)
    }

    async fn get_or_load_shards(
        &self,
        conn: &TLConnection,
        mc_block: &BlockIdExt,
    ) -> Result<Arc<HashSet<BlockIdExt>>, TonError> {
        Ok(self
            .block_shards_cache
            .try_get_with(mc_block.seqno, async move {
                let shards = conn.get_block_shards(mc_block.clone()).await?.shards;
                Ok(Arc::new(shards.into_iter().collect()))
            })
            .await?)
    }

    async fn get_or_load_unseen(
        &self,
        conn: &TLConnection,
        mc_seqno: u32,
        prev_shards: &HashSet<BlockIdExt>,
        cur_shards: HashSet<BlockIdExt>,
    ) -> Result<Arc<HashSet<BlockIdExt>>, TonError> {
        Ok(self
            .unseen_cache
            .try_get_with(mc_seqno, async move {
                let unseen_shards = self.get_unseen_shards(conn, mc_seqno, prev_shards, cur_shards).await?;
                Ok(Arc::new(unseen_shards))
            })
            .await?)
    }

    #[async_recursion]
    async fn get_unseen_shards(
        &self,
        conn: &TLConnection,
        mc_seqno: u32,
        prev_shards: &HashSet<BlockIdExt>,
        cur_shards: HashSet<BlockIdExt>,
    ) -> Result<HashSet<BlockIdExt>, TonError> {
        let get_prev_ids_futs = cur_shards.into_iter().map(|block_id| async {
            if prev_shards.contains(&block_id) || block_id.seqno == 0 {
                return Ok::<_, TonError>(Default::default());
            }

            let prev_ids = self.get_prev_blocks_with_retry(conn, mc_seqno, &block_id).await?;
            let mut unseen_prev_ids = self.get_unseen_shards(conn, mc_seqno, prev_shards, prev_ids).await?;
            unseen_prev_ids.insert(block_id);
            Ok(unseen_prev_ids)
        });

        let blocks = try_join_all(get_prev_ids_futs).await?.into_iter().flatten().collect();
        Ok(blocks)
    }

    async fn get_prev_blocks_with_retry(
        &self,
        conn: &TLConnection,
        mc_seqno: u32,
        block_id: &BlockIdExt,
    ) -> Result<HashSet<BlockIdExt>, TonError> {
        if let Ok(header) = conn.get_block_header(block_id.clone()).await {
            return Ok(HashSet::from_iter(header.prev_blocks.unwrap_or_default().into_iter()));
        }
        let mut last_error = None;
        for _ in 0..3 {
            let new_conn = self.find_connection(mc_seqno).await?;
            match new_conn.get_block_header(block_id.clone()).await {
                Ok(header) => return Ok(HashSet::from_iter(header.prev_blocks.unwrap_or_default().into_iter())),
                Err(err) => last_error = Some(err),
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        Err(last_error.unwrap())
    }
}
