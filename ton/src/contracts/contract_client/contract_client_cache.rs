use crate::contracts::contract_client::builder::Builder;
use crate::contracts::contract_client::cache_stats::CacheStats;
use crate::errors::{TonError, TonResult};
use futures_util::future::{join_all, try_join_all};
use moka::future::Cache;
use num_traits::Zero;
use parking_lot::RwLock;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::{Arc, Weak};
use std::time::Duration;
use ton_core::cell::{TonCell, TonHash};
use ton_core::traits::contract_provider::{TonContractState, TonProvider};
use ton_core::traits::tlb::TLB;
use ton_core::types::{TonAddress, TxLTHash};

pub(super) struct ContractClientCache {
    provider: Arc<dyn TonProvider>,
    latest_tx_cache: Cache<TonAddress, TxLTHash>,
    state_latest_cache: Cache<TonAddress, Arc<TonContractState>>,
    state_by_tx_cache: Cache<TxLTHash, Arc<TonContractState>>,
    libs_cache: moka::sync::Cache<TonHash, TonCell>,
    libs_cache_not_found: moka::sync::Cache<TonHash, ()>,
    code_extra_libs_cache: moka::sync::Cache<TonHash, Arc<RwLock<HashSet<TonHash>>>>, // code_hash -> set of lib_hashes
    cache_stats: CacheStats,
}

impl ContractClientCache {
    pub(super) fn new(builder: &Builder) -> Result<Arc<Self>, TonError> {
        let (contract_cache_capacity, contract_cache_ttl) =
            (builder.contract_cache_capacity, builder.contract_cache_ttl);
        let client_cache = Arc::new(Self {
            provider: builder.provider.clone(),
            latest_tx_cache: init_cache(contract_cache_capacity, contract_cache_ttl),
            state_latest_cache: init_cache(contract_cache_capacity, contract_cache_ttl),
            state_by_tx_cache: init_cache(contract_cache_capacity, contract_cache_ttl),
            libs_cache: init_sync_cache(builder.libs_cache_capacity, builder.libs_cache_ttl),
            libs_cache_not_found: init_sync_cache(
                builder.libs_not_found_cache_capacity,
                builder.libs_not_found_cache_ttl,
            ),
            code_extra_libs_cache: moka::sync::Cache::builder()
                .max_capacity(builder.code_libs_cache_capacity)
                .time_to_idle(builder.code_libs_cache_idle)
                .build(),
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
    ) -> TonResult<Arc<TonContractState>> {
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

    pub(super) fn add_code_dyn_lib(&self, code_hash: TonHash, lib_id: TonHash) {
        self.code_extra_libs_cache.entry(code_hash).or_default().value().write().insert(lib_id);
    }

    /// This method just skip unavailable libraries
    pub(super) async fn get_or_load_code_dyn_libs(&self, code_hash: TonHash) -> TonResult<HashMap<TonHash, TonCell>> {
        let Some(lib_hashes) = self.code_extra_libs_cache.get(&code_hash).map(|x| x.read().clone()) else {
            return Ok(HashMap::new());
        };
        self.get_or_load_libs(lib_hashes).await
    }

    pub(super) async fn get_or_load_libs(&self, lib_ids: HashSet<TonHash>) -> TonResult<HashMap<TonHash, TonCell>> {
        let futs = lib_ids.into_iter().map(|lib_id| async move {
            let lib = self.get_or_load_lib(lib_id.clone()).await?;
            Ok::<_, TonError>(lib.map(|x| (lib_id, x)))
        });
        let libs = try_join_all(futs).await?.into_iter().flatten().collect();
        Ok(libs)
    }

    pub(super) async fn get_or_load_lib(&self, lib_id: TonHash) -> TonResult<Option<TonCell>> {
        if self.libs_cache_not_found.contains_key(&lib_id) {
            return Ok(None);
        }
        if let Some(lib) = self.libs_cache.get(&lib_id) {
            return Ok(Some(lib.clone()));
        };

        if let Some(lib) = self.load_lib(lib_id.clone()).await? {
            self.libs_cache.insert(lib_id, lib.clone());
            return Ok(Some(lib.clone()));
        }
        self.libs_cache_not_found.insert(lib_id, ());
        Ok(None)
    }

    pub(super) fn cache_stats(&self) -> HashMap<String, usize> {
        let latest_entry_count = self.state_latest_cache.entry_count() as usize;
        let by_tx_entry_count = self.state_by_tx_cache.entry_count() as usize;
        self.cache_stats.export(latest_entry_count, by_tx_entry_count)
    }

    async fn load_contract(&self, address: &TonAddress, tx_id: Option<TxLTHash>) -> TonResult<Arc<TonContractState>> {
        match &tx_id {
            Some(_) => self.cache_stats.state_by_tx_miss.fetch_add(1, Relaxed),
            None => self.cache_stats.state_latest_miss.fetch_add(1, Relaxed),
        };
        let state = self.provider.load_state(address.clone(), tx_id).await?;
        Ok(Arc::new(state))
    }

    // TODO think about providing mc_seqno
    async fn load_lib(&self, lib_id: TonHash) -> TonResult<Option<TonCell>> {
        let mut response = self.provider.load_libs(vec![lib_id.clone()], None).await?;
        let Some(entry) = response.pop() else {
            return Ok(None);
        };
        Ok(Some(TonCell::from_boc(entry.1)?))
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

fn init_sync_cache<K, V>(capacity: u64, ttl: Duration) -> moka::sync::Cache<K, V>
where
    K: Eq + Hash + Send + Sync + 'static,
    V: Send + Sync + Clone + 'static,
{
    moka::sync::Cache::builder().max_capacity(capacity).time_to_live(ttl).build()
}
