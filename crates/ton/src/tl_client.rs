mod builder;
mod callback;
mod connection;
mod tl_state_provider;

pub mod tl;
mod tl_client_trait;

pub use callback::*;
pub use connection::*;
pub use tl_client_trait::*;
pub use tl_state_provider::*;

use crate::errors::TonResult;
use crate::tl_client::builder::Builder;
use async_trait::async_trait;
use rand::prelude::{IndexedRandom, StdRng};
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};
use std::time::Duration;

// /// Simple contract_client with many connections
#[derive(Clone)]
pub struct TLClient {
    inner: Arc<Inner>,
}

impl TLClient {
    pub fn builder() -> TonResult<Builder> { Builder::new() }
}

#[async_trait]
impl TLClientTrait for TLClient {
    fn get_connection(&self) -> &TLConnection {
        let mut rng_lock = self.inner.rnd.lock().unwrap();
        self.inner.connections.choose(&mut rng_lock.deref_mut()).unwrap()
    }

    fn get_retry_strategy(&self) -> &RetryStrategy { &self.inner.retry_strategy }
}

struct Inner {
    rnd: Mutex<StdRng>,
    connections: Vec<TLConnection>,
    retry_strategy: RetryStrategy,
}

/// Selects which healthy lite-server connections may be used.
///
/// Serde represents the variants as `"Healthy"` and `"Archive"`.
#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub enum LiteNodeFilter {
    Healthy, // connect to any healthy node
    Archive, // connect to archive node only
}

/// Retry settings for tonlib requests.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct RetryStrategy {
    pub retry_count: usize,
    pub retry_waiting: Duration,
}

impl RetryStrategy {
    /// Creates a retry strategy.
    pub const fn new(retry_count: usize, retry_waiting: Duration) -> Self {
        Self {
            retry_count,
            retry_waiting,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::LiteNodeFilter;

    #[test]
    fn test_lite_node_filter_serde_contract() -> anyhow::Result<()> {
        let cases = [
            (LiteNodeFilter::Healthy, "Healthy"),
            (LiteNodeFilter::Archive, "Archive"),
        ];

        for (filter, serialized) in cases {
            assert_eq!(serde_json::to_string(&filter)?, format!("\"{serialized}\""));
            assert_eq!(serde_json::from_str::<LiteNodeFilter>(&format!("\"{serialized}\""))?, filter);
        }

        Ok(())
    }
}
