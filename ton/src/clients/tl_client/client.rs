use crate::clients::tl_client::connection::TLConnection;
use crate::clients::tl_client::env::prepare_client_env;
use crate::clients::tl_client::RetryStrategy;
use crate::clients::tl_client::{config::TLClientConfig, tl::client::TLClientTrait};
use crate::errors::TonError;
use async_trait::async_trait;
use futures_util::future::try_join_all;
use rand::prelude::{IndexedRandom, StdRng};
use rand::SeedableRng;
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};
use tokio::sync::Semaphore;

// /// Simple client with many connections
#[derive(Clone)]
pub struct TLClient {
    inner: Arc<Inner>,
}

struct Inner {
    rnd: Mutex<StdRng>,
    connections: Vec<TLConnection>,
    config: TLClientConfig,
}

#[async_trait]
impl TLClientTrait for TLClient {
    fn get_connection(&self) -> &TLConnection {
        let mut rng_lock = self.inner.rnd.lock().unwrap();
        self.inner.connections.choose(&mut rng_lock.deref_mut()).unwrap()
    }

    fn get_retry_strategy(&self) -> &RetryStrategy { &self.inner.config.retry_strategy }
}

impl TLClient {
    pub async fn new(mut config: TLClientConfig) -> Result<TLClient, TonError> {
        prepare_client_env(&mut config).await?;

        let semaphore = Arc::new(Semaphore::new(config.max_parallel_requests));
        let conn_futs = (0..config.connections_count).map(|_| TLConnection::new(&config, semaphore.clone()));
        let connections = try_join_all(conn_futs).await?;
        log::info!("[TLClient] {} connections initialized", connections.len());
        let inner = Inner {
            rnd: Mutex::new(StdRng::from_rng(&mut rand::rng())),
            connections,
            config,
        };
        Ok(TLClient { inner: Arc::new(inner) })
    }
}
