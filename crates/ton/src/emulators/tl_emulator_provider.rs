use crate::emulators::emul_bc_config::EmulBCConfig;
use crate::emulators::emulator_pool::{EmulatorPool, TVMGetMethodTask};
use crate::emulators::tvm_emulator::{TVMEmulatorC7, TVMState};
use crate::errors::{TonError, TonResult};
use crate::libs_dict::LibsDict;
use crate::tl_client::{TLClient, TLClientTrait, load_tl_state};
use async_trait::async_trait;
use futures_util::future::try_join_all;
use parking_lot::RwLock;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::OnceCell;
use ton_core::cell::{TonCell, TonCellUtils, TonHash};
use ton_core::errors::{TonCoreError, TonCoreResult};
use ton_core::traits::emulator_provider::{
    EmulatorContractState, EmulatorGetMethodRequest, EmulatorGetMethodSuccess, EmulatorProvider,
};
use ton_core::traits::tlb::TLB;

/// Native tonlib implementation of [`EmulatorProvider`].
#[derive(Clone)]
pub struct TLEmulatorProvider {
    client: TLClient,
    emulator_pool: EmulatorPool,
    bc_config: Arc<OnceCell<EmulBCConfig>>,
    cache: Arc<TLEmulatorProviderCache>,
    libs_cache_capacity: u64,
    libs_cache_ttl: Duration,
    libs_not_found_cache_capacity: u64,
    libs_not_found_cache_ttl: Duration,
    code_libs_cache_capacity: u64,
    code_libs_cache_idle: Duration,
    max_dyn_libs_per_contract: usize,
}

impl TLEmulatorProvider {
    /// Creates a native provider with library caches disabled.
    pub fn new(client: TLClient, emulator_pool: EmulatorPool) -> Self {
        let mut provider = Self {
            client,
            emulator_pool,
            bc_config: Arc::new(OnceCell::new()),
            cache: Arc::new(TLEmulatorProviderCache::default()),
            libs_cache_capacity: 0,
            libs_cache_ttl: Duration::ZERO,
            libs_not_found_cache_capacity: 0,
            libs_not_found_cache_ttl: Duration::ZERO,
            code_libs_cache_capacity: 0,
            code_libs_cache_idle: Duration::ZERO,
            max_dyn_libs_per_contract: 100,
        };
        provider.rebuild_cache();
        provider
    }

    /// Enables the standard library-cache settings.
    pub fn with_default_caches(mut self) -> Self {
        self.libs_cache_capacity = 1_000;
        self.libs_cache_ttl = Duration::from_secs(300);
        self.libs_not_found_cache_capacity = 5_000;
        self.libs_not_found_cache_ttl = Duration::from_secs(300);
        self.code_libs_cache_capacity = 5_000;
        self.code_libs_cache_idle = Duration::from_secs(600);
        self.rebuild_cache();
        self
    }

    /// Sets the resolved-library cache capacity.
    pub fn with_libs_cache_capacity(mut self, capacity: u64) -> Self {
        self.libs_cache_capacity = capacity;
        self.rebuild_cache();
        self
    }

    /// Sets the resolved-library cache time to live.
    pub fn with_libs_cache_ttl(mut self, ttl: Duration) -> Self {
        self.libs_cache_ttl = ttl;
        self.rebuild_cache();
        self
    }

    /// Sets the missing-library cache capacity.
    pub fn with_libs_not_found_cache_capacity(mut self, capacity: u64) -> Self {
        self.libs_not_found_cache_capacity = capacity;
        self.rebuild_cache();
        self
    }

    /// Sets the missing-library cache time to live.
    pub fn with_libs_not_found_cache_ttl(mut self, ttl: Duration) -> Self {
        self.libs_not_found_cache_ttl = ttl;
        self.rebuild_cache();
        self
    }

    /// Sets the code-to-dynamic-library cache capacity.
    pub fn with_code_libs_cache_capacity(mut self, capacity: u64) -> Self {
        self.code_libs_cache_capacity = capacity;
        self.rebuild_cache();
        self
    }

    /// Sets the code-to-dynamic-library cache idle timeout.
    pub fn with_code_libs_cache_idle(mut self, idle: Duration) -> Self {
        self.code_libs_cache_idle = idle;
        self.rebuild_cache();
        self
    }

    /// Sets the maximum number of missing libraries loaded for one request.
    pub fn with_max_dyn_libs_per_contract(mut self, max_dyn_libs_per_contract: usize) -> Self {
        self.max_dyn_libs_per_contract = max_dyn_libs_per_contract;
        self
    }

    fn rebuild_cache(&mut self) {
        self.cache = Arc::new(TLEmulatorProviderCache {
            libs_cache: init_cache(self.libs_cache_capacity, self.libs_cache_ttl),
            libs_cache_not_found: init_cache(self.libs_not_found_cache_capacity, self.libs_not_found_cache_ttl),
            code_extra_libs_cache: moka::sync::Cache::builder()
                .max_capacity(self.code_libs_cache_capacity)
                .time_to_idle(self.code_libs_cache_idle)
                .build(),
        });
    }

    async fn emulate_get_method_impl(
        &self,
        request: EmulatorGetMethodRequest,
        timeout: Option<Duration>,
    ) -> TonResult<EmulatorGetMethodSuccess> {
        let state = match request.contract_state {
            EmulatorContractState::Address { address, tx_id } => load_tl_state(&self.client, address, tx_id).await?,
            EmulatorContractState::Custom(state) => state,
            _ => {
                return Err(TonError::EmulatorUnexpectedResponse(
                    "unsupported EmulatorContractState variant".to_string(),
                ));
            }
        };

        let code_boc = state.code_boc.as_ref().ok_or_else(|| TonError::TonContractNotFull {
            address: state.address,
            tx_id: Some(state.last_tx_id.clone()),
            missing_field: "code".to_string(),
        })?;
        let code_cell = TonCell::from_boc(code_boc.to_owned())?;
        let data_cell = match &state.data_boc {
            Some(data_boc) => TonCell::from_boc(data_boc.clone())?,
            None => TonCell::empty().clone(),
        };
        let data_boc = state.data_boc.unwrap_or_else(|| Arc::new(Vec::new()));
        let code_hash = *code_cell.hash()?;
        let static_lib_ids = TonCellUtils::extract_lib_ids([&code_cell, &data_cell])?;
        let mut emulation_libs = self.get_or_load_libs(static_lib_ids).await?;
        emulation_libs.extend(self.get_or_load_code_dyn_libs(code_hash).await?);
        let mut libs_dict = LibsDict::from(emulation_libs);

        let config = self
            .bc_config
            .get_or_try_init(|| async {
                let config = self.client.get_config_boc_all(0).await?;
                EmulBCConfig::from_boc(&config)
            })
            .await?
            .clone();
        let unix_time = SystemTime::now().duration_since(UNIX_EPOCH).map_err(TonCoreError::from)?.as_secs() as u32;
        let mut task = TVMGetMethodTask {
            state: TVMState {
                code_boc: code_boc.clone(),
                data_boc,
                c7: TVMEmulatorC7 {
                    address: state.address,
                    unix_time,
                    balance: state.balance as u64,
                    rand_seed: TonHash::ZERO,
                    config,
                },
                libs_boc: None,
                debug_enabled: None,
                gas_limit: None,
            },
            method: request.method_id.into(),
            stack_boc: request.stack_boc,
        };
        if !libs_dict.is_empty() {
            task.state.libs_boc = Some(Arc::new(libs_dict.to_boc()?));
        }

        let mut iteration = 0;
        loop {
            let response = self.emulator_pool.emul_get_method(task.clone(), timeout).await?;
            let Some(missing_lib_hash) = response.missing_lib()? else {
                let success = response.into_success()?;
                let stack_boc = success.stack_boc()?;
                return Ok(EmulatorGetMethodSuccess::with_diagnostics(
                    success.vm_exit_code,
                    stack_boc,
                    success.vm_log,
                    success.gas_used,
                    success.raw_response,
                ));
            };

            iteration += 1;
            if iteration > self.max_dyn_libs_per_contract {
                return Err(TonError::EmulatorTooManyLibraries(self.max_dyn_libs_per_contract));
            }
            let Some(lib) = self.load_lib(missing_lib_hash).await? else {
                return Err(TonError::EmulatorMissingLibrary(missing_lib_hash));
            };
            self.cache.add_code_dyn_lib(code_hash, missing_lib_hash);
            libs_dict.insert(missing_lib_hash, lib.into());
            task.state.libs_boc = Some(Arc::new(libs_dict.to_boc()?));
        }
    }

    async fn get_or_load_code_dyn_libs(&self, code_hash: TonHash) -> TonResult<HashMap<TonHash, TonCell>> {
        let Some(lib_hashes) = self.cache.code_extra_libs_cache.get(&code_hash).map(|x| x.read().clone()) else {
            return Ok(HashMap::new());
        };
        self.get_or_load_libs(lib_hashes).await
    }

    async fn get_or_load_libs(&self, lib_ids: HashSet<TonHash>) -> TonResult<HashMap<TonHash, TonCell>> {
        let futs = lib_ids.into_iter().map(|lib_id| async move {
            let lib = self.load_lib(lib_id).await?;
            Ok::<_, TonError>(lib.map(|lib| (lib_id, lib)))
        });
        Ok(try_join_all(futs).await?.into_iter().flatten().collect())
    }

    async fn load_lib(&self, lib_id: TonHash) -> TonResult<Option<TonCell>> {
        if self.cache.libs_cache_not_found.contains_key(&lib_id) {
            return Ok(None);
        }
        if let Some(lib) = self.cache.libs_cache.get(&lib_id) {
            return Ok(Some(lib));
        }

        let mut libs = self.client.get_libs(vec![lib_id]).await?;
        let Some(lib) = libs.pop() else {
            self.cache.libs_cache_not_found.insert(lib_id, ());
            return Ok(None);
        };
        let lib_hash = TonHash::from_vec(lib.hash)?;
        let lib = TonCell::from_boc(lib.data)?;
        self.cache.libs_cache.insert(lib_hash, lib.clone());
        Ok((lib_hash == lib_id).then_some(lib))
    }
}

struct TLEmulatorProviderCache {
    libs_cache: moka::sync::Cache<TonHash, TonCell>,
    libs_cache_not_found: moka::sync::Cache<TonHash, ()>,
    code_extra_libs_cache: moka::sync::Cache<TonHash, Arc<RwLock<HashSet<TonHash>>>>,
}

impl Default for TLEmulatorProviderCache {
    fn default() -> Self {
        Self {
            libs_cache: init_cache(0, Duration::ZERO),
            libs_cache_not_found: init_cache(0, Duration::ZERO),
            code_extra_libs_cache: moka::sync::Cache::builder().max_capacity(0).time_to_idle(Duration::ZERO).build(),
        }
    }
}

impl TLEmulatorProviderCache {
    fn add_code_dyn_lib(&self, code_hash: TonHash, lib_id: TonHash) {
        self.code_extra_libs_cache.entry(code_hash).or_default().value().write().insert(lib_id);
    }
}

fn init_cache<K, V>(capacity: u64, ttl: Duration) -> moka::sync::Cache<K, V>
where
    K: Eq + std::hash::Hash + Send + Sync + 'static,
    V: Send + Sync + Clone + 'static,
{
    moka::sync::Cache::builder().max_capacity(capacity).time_to_live(ttl).build()
}

#[async_trait]
impl EmulatorProvider for TLEmulatorProvider {
    async fn emulate_get_method(
        &self,
        request: EmulatorGetMethodRequest,
        timeout: Option<Duration>,
    ) -> TonCoreResult<EmulatorGetMethodSuccess> {
        let result = match timeout {
            Some(timeout) => {
                match tokio::time::timeout(timeout, self.emulate_get_method_impl(request, Some(timeout))).await {
                    Ok(result) => result,
                    Err(_) => Err(TonError::EmulatorTimeout(timeout)),
                }
            }
            None => self.emulate_get_method_impl(request, None).await,
        };
        result.map_err(preserve_ton_error)
    }
}

fn preserve_ton_error(error: TonError) -> TonCoreError { TonCoreError::BoxedError(Box::new(error)) }
