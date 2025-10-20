mod builder;
mod callback;
mod connection;

pub mod tl;
mod tl_client_trait;

pub use callback::*;
pub use connection::*;
pub use tl_client_trait::*;

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

#[derive(Debug, PartialEq, Clone)]
pub enum LiteNodeFilter {
    Healthy, // connect to any healthy node
    Archive, // connect to archive node only
}

#[derive(Debug, Clone)]
pub struct RetryStrategy {
    pub retry_count: usize,
    pub retry_waiting: Duration,
}
