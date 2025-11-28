use crate::contracts::ContractClient;
use crate::contracts::contract_client::Inner;
use crate::contracts::contract_client::contract_client_cache::ContractClientCache;
use crate::errors::TonResult;
use derive_setters::Setters;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::OnceCell;
use ton_core::traits::contract_provider::TonProvider;

#[derive(Setters)]
#[setters(prefix = "with_", strip_option)]
pub struct Builder {
    #[setters(skip)]
    pub(super) provider: Arc<dyn TonProvider>,
    pub(super) refresh_loop_idle_on_error: Duration,
    pub(super) contract_cache_capacity: u64,
    pub(super) contract_cache_ttl: Duration,
    pub(super) libs_cache_capacity: u64,
    pub(super) libs_cache_ttl: Duration,
    pub(super) libs_not_found_cache_capacity: u64,
    pub(super) libs_not_found_cache_ttl: Duration,
    pub(super) code_libs_cache_capacity: u64,
    pub(super) code_libs_cache_idle: Duration,
    pub(super) max_libs_per_contract: usize,
}

impl Builder {
    /// Use ContractClient::builder() for creation
    /// No cache by default
    /// Use `with_default_caches()` for meaningful defaults
    pub(super) fn new(provider: impl TonProvider) -> Self {
        Self {
            provider: Arc::new(provider),
            refresh_loop_idle_on_error: Duration::from_millis(100),
            contract_cache_capacity: 0,
            contract_cache_ttl: Duration::from_millis(0),
            libs_cache_capacity: 0,
            libs_cache_ttl: Duration::from_secs(0),
            libs_not_found_cache_capacity: 0,
            libs_not_found_cache_ttl: Duration::from_secs(0),
            code_libs_cache_capacity: 0,
            code_libs_cache_idle: Duration::from_secs(0),
            max_libs_per_contract: 100,
        }
    }

    pub fn build(self) -> TonResult<ContractClient> {
        let cache = ContractClientCache::new(&self)?;
        let inner = Inner {
            provider: self.provider,
            cache,
            bc_config: OnceCell::new(),
            max_libs_per_contract: self.max_libs_per_contract,
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
