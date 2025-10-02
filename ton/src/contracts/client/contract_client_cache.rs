use crate::contracts::client::cache_stats::CacheStats;
use crate::contracts::ContractClientConfig;
use crate::errors::TonError;
use futures_util::future::join_all;
use moka::future::Cache;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::{Arc, Weak};
use std::time::Duration;
use ton_lib_core::errors::TonCoreError;
use ton_lib_core::traits::contract_provider::{TonContractState, TonProvider};
use ton_lib_core::types::{TonAddress, TxLTHash};

pub(super) struct ContractClientCache {
    provider: Arc<dyn TonProvider>,
    latest_tx_cache: Cache<TonAddress, TxLTHash>,
    state_latest_cache: Cache<TonAddress, Arc<TonContractState>>,
    state_by_tx_cache: Cache<TxLTHash, Arc<TonContractState>>,
    cache_stats: CacheStats,
}

impl ContractClientCache {
    pub(super) fn new(config: ContractClientConfig, provider: Arc<dyn TonProvider>) -> Result<Arc<Self>, TonError> {
        let (capacity, ttl) = (config.cache_capacity, config.cache_ttl);
        let client_cache = Arc::new(Self {
            provider: provider.clone(),
            latest_tx_cache: init_cache(capacity, ttl),
            state_latest_cache: init_cache(capacity, ttl),
            state_by_tx_cache: init_cache(capacity, ttl),
            cache_stats: CacheStats::default(),
        });
        let weak = Arc::downgrade(&client_cache);
        tokio::spawn(recent_tx_loop(weak, config.refresh_loop_idle_on_error));
        Ok(client_cache)
    }

    pub(super) async fn get_or_load_contract(
        &self,
        address: &TonAddress,
        tx_id: Option<&TxLTHash>,
    ) -> Result<Arc<TonContractState>, TonError> {
        if let Some(tx_id) = tx_id {
            self.cache_stats.state_by_tx_req.fetch_add(1, Relaxed);
            return Ok(self
                .state_by_tx_cache
                .try_get_with_by_ref(tx_id, self.load_contract(address, Some(tx_id.clone())))
                .await?);
        }

        self.cache_stats.state_latest_req.fetch_add(1, Relaxed);
        let state = if let Some(id) = self.latest_tx_cache.get(address).await {
            self.state_latest_cache.try_get_with_by_ref(address, self.load_contract(address, Some(id))).await?
        } else {
            self.state_latest_cache.try_get_with_by_ref(address, self.load_contract(address, None)).await?
        };
        Ok(state)
    }

    pub(super) fn cache_stats(&self) -> HashMap<String, usize> {
        let latest_entry_count = self.state_latest_cache.entry_count() as usize;
        let by_tx_entry_count = self.state_by_tx_cache.entry_count() as usize;
        self.cache_stats.export(latest_entry_count, by_tx_entry_count)
    }

    async fn load_contract(
        &self,
        address: &TonAddress,
        tx_id: Option<TxLTHash>,
    ) -> Result<Arc<TonContractState>, TonCoreError> {
        match &tx_id {
            Some(_) => self.cache_stats.state_by_tx_miss.fetch_add(1, Relaxed),
            None => self.cache_stats.state_latest_miss.fetch_add(1, Relaxed),
        };
        let state = self.provider.load_state(address.clone(), tx_id).await?;
        Ok(Arc::new(state))
    }
}

async fn recent_tx_loop(weak_cache: Weak<ContractClientCache>, idle_on_error: Duration) {
    log::info!("[recent_tx_loop] initializing...");
    let mut cur_mc_seqno = if let Some(inner) = weak_cache.upgrade() {
        loop {
            match inner.provider.last_mc_seqno().await {
                Ok(seqno) => break seqno,
                Err(err) => {
                    log::warn!("[recent_tx_loop] fail to get last mc seqno: {err}");
                    tokio::time::sleep(idle_on_error).await;
                    continue;
                }
            }
        }
    } else {
        log::warn!("[recent_tx_loop] inner is already dropped, exiting loop");
        return;
    };
    log::info!("[recent_tx_loop] started with last_mc_seqno: {cur_mc_seqno}");

    loop {
        let client_cache = match weak_cache.upgrade() {
            Some(inner) => inner,
            None => {
                log::warn!("[recent_tx_loop] inner is dropped");
                break;
            }
        };
        let client_cache_ref = &client_cache;

        let latest_tx_per_addr = match client_cache_ref.provider.load_latest_tx_per_address(cur_mc_seqno).await {
            Ok(latest_tx) => latest_tx,
            Err(err) => {
                log::warn!("[recent_tx_loop] fail to loading latest txs: {err}");
                tokio::time::sleep(idle_on_error).await;
                continue;
            }
        };

        let update_cache_futs = latest_tx_per_addr.into_iter().map(|(address, tx_id)| async move {
            client_cache_ref.latest_tx_cache.insert(address.clone(), tx_id).await;
            client_cache_ref.state_latest_cache.invalidate(&address).await;
        });
        join_all(update_cache_futs).await;
        cur_mc_seqno += 1;
    }
    log::info!("[recent_tx_loop] completed");
}

fn init_cache<K, V>(capacity: u64, ttl: Duration) -> Cache<K, V>
where
    K: Eq + Hash + Send + Sync + 'static,
    V: Send + Sync + Clone + 'static,
{
    Cache::builder().max_capacity(capacity).time_to_live(ttl).build()
}
