use crate::contracts::contract_client::builder::Builder;
use crate::contracts::contract_client::cache_stats::CacheStats;
use crate::errors::{TonError, TonResult};
use futures_util::future::join_all;
use moka::future::Cache;
use num_traits::Zero;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering::{AcqRel, Acquire, Relaxed, Release};
use std::sync::{Arc, Weak};
use std::time::Duration;
use ton_core::traits::state_provider::{ContractState, StateProvider};
use ton_core::types::{TonAddress, TxLTHash};

pub(super) struct ContractClientCache {
    state_provider: Arc<dyn StateProvider>,
    latest_tx_cache: Cache<TonAddress, TxLTHash>,
    state_latest_cache: Cache<TonAddress, Arc<ContractState>>,
    state_by_tx_cache: Cache<TxLTHash, Arc<ContractState>>,
    refresh_epoch: AtomicU64,
    cache_stats: CacheStats,
}

impl ContractClientCache {
    pub(super) fn new(builder: &Builder) -> Result<Arc<Self>, TonError> {
        let (contract_cache_capacity, contract_cache_ttl) =
            (builder.contract_cache_capacity, builder.contract_cache_ttl);
        let client_cache = Arc::new(Self {
            state_provider: builder.state_provider.clone(),
            latest_tx_cache: init_cache(contract_cache_capacity, contract_cache_ttl),
            state_latest_cache: init_cache(contract_cache_capacity, contract_cache_ttl),
            state_by_tx_cache: init_cache(contract_cache_capacity, contract_cache_ttl),
            refresh_epoch: AtomicU64::new(0),
            cache_stats: CacheStats::default(),
        });
        let weak = Arc::downgrade(&client_cache);
        if contract_cache_capacity.is_zero() {
            log::warn!("[ContractClientCache] contract_cache_capacity == 0, recent_tx_loop won't be started");
        } else {
            tokio::spawn(recent_tx_loop(weak, builder.refresh_loop_idle_on_error));
        }
        Ok(client_cache)
    }

    pub(super) async fn get_or_load_contract(
        &self,
        address: &TonAddress,
        tx_id: Option<&TxLTHash>,
    ) -> TonResult<Arc<ContractState>> {
        if let Some(tx_id) = tx_id {
            self.cache_stats.state_by_tx_req.fetch_add(1, Relaxed);
            return Ok(self
                .state_by_tx_cache
                .try_get_with_by_ref(tx_id, self.load_contract(address, Some(tx_id.clone())))
                .await?);
        }

        self.cache_stats.state_latest_req.fetch_add(1, Relaxed);
        loop {
            let refresh_epoch = self.refresh_epoch.load(Acquire);
            if refresh_epoch & 1 != 0 {
                tokio::task::yield_now().await;
                continue;
            }
            let state = if let Some(id) = self.latest_tx_cache.get(address).await {
                self.state_latest_cache.try_get_with_by_ref(address, self.load_contract(address, Some(id))).await?
            } else {
                self.state_latest_cache.try_get_with_by_ref(address, self.load_contract(address, None)).await?
            };

            // A load started before refresh invalidated this address can finish
            // afterwards. Discard that result instead of returning stale state.
            let transaction_changed =
                self.latest_tx_cache.get(address).await.is_some_and(|latest_tx_id| latest_tx_id != state.last_tx_id);
            let current_refresh_epoch = self.refresh_epoch.load(Acquire);
            let refresh_overlapped = refresh_epoch != current_refresh_epoch || current_refresh_epoch & 1 != 0;
            if refresh_overlapped || transaction_changed {
                self.state_latest_cache.invalidate(address).await;
                continue;
            }
            return Ok(state);
        }
    }

    pub(super) fn cache_stats(&self) -> HashMap<String, usize> {
        let latest_entry_count = self.state_latest_cache.entry_count() as usize;
        let by_tx_entry_count = self.state_by_tx_cache.entry_count() as usize;
        self.cache_stats.export(latest_entry_count, by_tx_entry_count)
    }

    async fn load_contract(&self, address: &TonAddress, tx_id: Option<TxLTHash>) -> TonResult<Arc<ContractState>> {
        match &tx_id {
            Some(_) => self.cache_stats.state_by_tx_miss.fetch_add(1, Relaxed),
            None => self.cache_stats.state_latest_miss.fetch_add(1, Relaxed),
        };
        let state = self.state_provider.load_state(*address, tx_id).await?;
        Ok(Arc::new(state))
    }
}

async fn recent_tx_loop(weak_cache: Weak<ContractClientCache>, idle_on_error: Duration) {
    log::info!("[recent_tx_loop] initializing...");
    let mut cur_mc_seqno = if let Some(inner) = weak_cache.upgrade() {
        loop {
            match inner.state_provider.last_mc_seqno().await {
                Ok(seqno) => break seqno,
                Err(err) => {
                    log::warn!("[recent_tx_loop] fail to get last mc seqno: {err}");
                    tokio::time::sleep(idle_on_error).await;
                    continue;
                },
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
            },
        };
        let client_cache_ref = &client_cache;

        let latest_tx_per_addr = match client_cache_ref.state_provider.load_latest_tx_per_address(cur_mc_seqno).await {
            Ok(latest_tx) => latest_tx,
            Err(err) => {
                log::warn!("[recent_tx_loop] fail to loading latest txs: {err}");
                tokio::time::sleep(idle_on_error).await;
                continue;
            },
        };
        log::debug!(
            "[recent_tx_loop] mc_seqno {}: loaded {} txs (last per address)",
            cur_mc_seqno,
            latest_tx_per_addr.len()
        );

        client_cache_ref.refresh_epoch.fetch_add(1, AcqRel);
        let update_cache_futs = latest_tx_per_addr.into_iter().map(|(address, tx_id)| async move {
            client_cache_ref.latest_tx_cache.insert(address, tx_id).await;
            client_cache_ref.state_latest_cache.invalidate(&address).await;
        });
        join_all(update_cache_futs).await;
        client_cache_ref.refresh_epoch.fetch_add(1, Release);
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
