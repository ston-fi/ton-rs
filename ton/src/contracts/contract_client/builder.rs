use crate::contracts::ContractClient;
use crate::contracts::contract_client::Inner;
use crate::contracts::contract_client::contract_client_cache::ContractClientCache;
use crate::emulators::tvm_emulator::{TVMEmulatorPool, TVMEmulatorWrapper};
use crate::errors::{TonError, TonResult};
use derive_setters::Setters;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tokio::sync::OnceCell;
use ton_core::traits::contract_provider::TonProvider;

#[derive(Setters)]
#[setters(prefix = "with_", strip_option)]
pub struct Builder {
    #[setters(skip)]
    pub(super) provider: Arc<dyn TonProvider>,
    pub(super) tvm_emulator_pool_size: usize,
    pub(super) tvm_emulator_pool: Option<TVMEmulatorPool>,
    pub(super) tvm_emulation_timeout: Duration,
    pub(super) refresh_loop_idle_on_error: Duration,
    pub(super) contract_cache_capacity: u64,
    pub(super) contract_cache_ttl: Duration,
    pub(super) libs_cache_capacity: u64,
    pub(super) libs_cache_ttl: Duration,
    pub(super) libs_not_found_cache_capacity: u64,
    pub(super) libs_not_found_cache_ttl: Duration,
    pub(super) code_libs_cache_capacity: u64,
    pub(super) code_libs_cache_idle: Duration,
    // how many times emulate_get_method will try load new missing_libraries
    pub(super) max_dyn_libs_per_contract: usize,
}

impl Builder {
    /// Use ContractClient::builder() for creation
    /// No cache by default
    /// Use `with_default_caches()` for meaningful defaults
    pub(super) fn new(provider: impl TonProvider) -> TonResult<Self> {
        let builder = Self {
            provider: Arc::new(provider),
            tvm_emulator_pool_size: thread::available_parallelism().map_err(TonError::system)?.get(),
            tvm_emulator_pool: None,
            tvm_emulation_timeout: Duration::from_millis(10),
            refresh_loop_idle_on_error: Duration::from_millis(100),
            contract_cache_capacity: 0,
            contract_cache_ttl: Duration::from_millis(0),
            libs_cache_capacity: 0,
            libs_cache_ttl: Duration::from_secs(0),
            libs_not_found_cache_capacity: 0,
            libs_not_found_cache_ttl: Duration::from_secs(0),
            code_libs_cache_capacity: 0,
            code_libs_cache_idle: Duration::from_secs(0),
            max_dyn_libs_per_contract: 100,
        };
        Ok(builder)
    }

    pub fn build(self) -> TonResult<ContractClient> {
        let cache = ContractClientCache::new(&self)?;
        let emulator_pool = match self.tvm_emulator_pool {
            Some(pool) => pool,
            None => TVMEmulatorPool::builder(vec![TVMEmulatorWrapper; self.tvm_emulator_pool_size])?.build()?,
        };
        let inner = Inner {
            provider: self.provider,
            emulation_timeout: self.tvm_emulation_timeout,
            emulator_pool,
            cache,
            bc_config: OnceCell::new(),
            max_dyn_libs_per_contract: self.max_dyn_libs_per_contract,
        };
        Ok(ContractClient { inner: Arc::new(inner) })
    }

    /// Some meaningful defaults
    pub fn with_default_caches(mut self) -> Self {
        self.contract_cache_capacity = 5_000;
        self.contract_cache_ttl = Duration::from_secs(300);
        self.libs_cache_capacity = 1_000;
        self.libs_cache_ttl = Duration::from_secs(300);
        self.libs_not_found_cache_capacity = 5_000; // keeps only TonHash
        self.libs_not_found_cache_ttl = Duration::from_secs(300);
        self.code_libs_cache_capacity = 5_000;
        self.code_libs_cache_idle = Duration::from_secs(600);
        self
    }
}
