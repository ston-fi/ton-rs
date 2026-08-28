use crate::contracts::ContractClient;
use crate::contracts::contract_client::Inner;
use crate::contracts::contract_client::contract_client_cache::ContractClientCache;
use crate::errors::TonResult;
use derive_setters::Setters;
use std::sync::Arc;
use std::time::Duration;
use ton_core::traits::emulation_provider::EmulationProvider;
use ton_core::traits::state_provider::StateProvider;

#[derive(Setters)]
#[setters(prefix = "with_", strip_option)]
pub struct Builder {
    #[setters(skip)]
    pub(super) state_provider: Arc<dyn StateProvider>,
    #[setters(skip)]
    pub(super) emulation_provider: Arc<dyn EmulationProvider>,
    pub(super) tvm_emulation_timeout: Duration,
    pub(super) refresh_loop_idle_on_error: Duration,
    pub(super) contract_cache_capacity: u64,
    pub(super) contract_cache_ttl: Duration,
}

impl Builder {
    pub(super) fn new(
        state_provider: impl StateProvider,
        emulation_provider: impl EmulationProvider,
    ) -> TonResult<Self> {
        let builder = Self {
            state_provider: Arc::new(state_provider),
            emulation_provider: Arc::new(emulation_provider),
            tvm_emulation_timeout: Duration::from_secs(10),
            refresh_loop_idle_on_error: Duration::from_millis(100),
            contract_cache_capacity: 0,
            contract_cache_ttl: Duration::from_millis(0),
        };
        Ok(builder)
    }

    /// Builds the client, starting state-cache refresh when enabled.
    ///
    /// # Panics
    ///
    /// Panics if state caches are enabled outside a Tokio runtime.
    pub fn build(self) -> TonResult<ContractClient> {
        let cache = ContractClientCache::new(&self)?;
        let inner = Inner {
            emulation_provider: self.emulation_provider,
            emulation_timeout: self.tvm_emulation_timeout,
            cache,
        };
        Ok(ContractClient { inner: Arc::new(inner) })
    }

    /// Enables default contract-state caching and refresh; emulator caches are unchanged.
    pub fn with_default_caches(mut self) -> Self {
        self.contract_cache_capacity = 5_000;
        self.contract_cache_ttl = Duration::from_secs(300);
        self
    }
}
