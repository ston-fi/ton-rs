use crate::contracts::ContractClient;
use crate::contracts::contract_client::Inner;
use crate::contracts::contract_client::contract_client_cache::ContractClientCache;
use crate::errors::TonResult;
use derive_setters::Setters;
use std::sync::Arc;
use std::time::Duration;
use ton_core::traits::emulator_provider::EmulatorProvider;
use ton_core::traits::state_provider::StateProvider;

#[derive(Setters)]
#[setters(prefix = "with_", strip_option)]
pub struct Builder {
    #[setters(skip)]
    pub(super) state_provider: Arc<dyn StateProvider>,
    #[setters(skip)]
    pub(super) emulator_provider: Arc<dyn EmulatorProvider>,
    pub(super) tvm_emulation_timeout: Duration,
    pub(super) refresh_loop_idle_on_error: Duration,
    pub(super) contract_cache_capacity: u64,
    pub(super) contract_cache_ttl: Duration,
}

impl Builder {
    /// Use `ContractClient::builder(state_provider, emulator_provider)` for creation.
    /// No cache by default
    /// Use `with_default_caches()` for meaningful defaults
    pub(super) fn new(state_provider: impl StateProvider, emulator_provider: impl EmulatorProvider) -> TonResult<Self> {
        let builder = Self {
            state_provider: Arc::new(state_provider),
            emulator_provider: Arc::new(emulator_provider),
            tvm_emulation_timeout: Duration::from_secs(1),
            refresh_loop_idle_on_error: Duration::from_millis(100),
            contract_cache_capacity: 0,
            contract_cache_ttl: Duration::from_millis(0),
        };
        Ok(builder)
    }

    /// Builds the contract client and starts state-cache refresh when enabled.
    ///
    /// The refresh task keeps a provider call alive until it completes. During
    /// initial sequence discovery, provider errors are retried until discovery
    /// succeeds, even if all client handles have been dropped.
    ///
    /// # Panics
    ///
    /// Panics when state caches are enabled and no Tokio runtime is active.
    pub fn build(self) -> TonResult<ContractClient> {
        let cache = ContractClientCache::new(&self)?;
        let inner = Inner {
            emulator_provider: self.emulator_provider,
            emulation_timeout: self.tvm_emulation_timeout,
            cache,
        };
        Ok(ContractClient { inner: Arc::new(inner) })
    }

    /// Enables the standard contract-state caches and background refresh task.
    ///
    /// Native emulator library caches are configured separately on
    /// `TLEmulatorProvider`.
    pub fn with_default_caches(mut self) -> Self {
        self.contract_cache_capacity = 5_000;
        self.contract_cache_ttl = Duration::from_secs(300);
        self
    }
}
