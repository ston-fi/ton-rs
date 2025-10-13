use crate::contracts::contract_client::Inner;
use crate::contracts::contract_client_cache::ContractClientCache;
use crate::contracts::ContractClient;
use crate::errors::TonResult;
use derive_setters::Setters;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::OnceCell;
use ton_lib_core::traits::contract_provider::TonProvider;

#[derive(Setters)]
#[setters(prefix = "with_", strip_option)]
pub struct Builder {
    #[setters(skip)]
    pub(super) provider: Arc<dyn TonProvider>,
    pub(super) refresh_loop_idle_on_error: Duration,
    pub(super) cache_capacity: u64,
    pub(super) cache_ttl: Duration,
}

impl Builder {
    /// No cache by default
    pub(super) fn new(provider: impl TonProvider) -> Self {
        Self {
            provider: Arc::new(provider),
            refresh_loop_idle_on_error: Duration::from_millis(100),
            cache_capacity: 0,
            cache_ttl: Duration::from_millis(0),
        }
    }

    pub fn build(self) -> TonResult<ContractClient> {
        let cache = ContractClientCache::new(&self)?;
        let inner = Inner {
            provider: self.provider,
            cache,
            bc_config: OnceCell::new(),
        };
        Ok(ContractClient { inner: Arc::new(inner) })
    }
}
