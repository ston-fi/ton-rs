use crate::bail_ton;
use crate::block_tlb::BlockIdExt;
use crate::errors::{TonError, TonResult};
use crate::lite_client::LiteClient;
use crate::net_config::TonNetConfig;
use crate::tl_client::tl::{TLConfig, TLKeyStoreType, TLOptions};
use crate::tl_client::{Inner, LiteNodeFilter, RetryStrategy, TLCallbacksStore, TLClient, TLConnection};
use derive_setters::Setters;
use futures_util::future::{join_all, try_join_all};
use rand::SeedableRng;
use rand::prelude::StdRng;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::Semaphore;

#[derive(Setters, Debug)]
#[setters(prefix = "with_", strip_option)]
pub struct Builder {
    pub(super) mainnet: bool,
    pub(super) init_opts: TLOptions,
    pub(super) connection_check: LiteNodeFilter,
    pub(super) connections_count: usize,
    pub(super) max_parallel_requests: usize, // max_parallel_requests / connections_count = parallel requests per connection
    pub(super) retry_strategy: RetryStrategy,
    pub(super) update_init_block: bool,
    pub(super) update_init_block_timeout_sec: u64,
    pub(super) sleep_on_connection_error_ms: Duration,
    pub(super) tonlib_verbosity_level: u32,
    pub(super) callbacks: TLCallbacksStore,
}

impl Builder {
    /// May fail to read net config
    pub(super) fn new() -> TonResult<Self> {
        let builder = Self {
            mainnet: true,
            init_opts: TLOptions {
                config: TLConfig {
                    net_config_json: "".to_string(),
                    blockchain_name: None,
                    use_callbacks_for_network: false,
                    ignore_cache: false,
                },
                keystore_type: TLKeyStoreType::InMemory,
            },
            connection_check: LiteNodeFilter::Healthy,
            connections_count: 5,
            max_parallel_requests: 10,
            retry_strategy: RetryStrategy {
                retry_count: 5,
                retry_waiting: Duration::from_millis(200),
            },
            update_init_block: true,
            update_init_block_timeout_sec: 10,
            sleep_on_connection_error_ms: Duration::from_millis(100),
            tonlib_verbosity_level: 1,
            callbacks: Default::default(),
        };
        Ok(builder)
    }

    pub async fn build(mut self) -> TonResult<TLClient> {
        if self.connections_count == 0 {
            bail_ton!("connections_count must be > 0");
        }

        if self.init_opts.config.net_config_json.is_empty() {
            self.init_opts.config.net_config_json = TonNetConfig::new_default(self.mainnet)?.to_json()?;
        }

        if self.update_init_block {
            if let Some(net_config) = update_net_config(&self).await? {
                self.init_opts.config.net_config_json = net_config.to_json()?;
            }
        }
        if let TLKeyStoreType::Directory { directory } = &self.init_opts.keystore_type {
            std::fs::create_dir_all(directory).map_err(TonError::system)?
        }

        let semaphore = Arc::new(Semaphore::new(self.max_parallel_requests));
        let conn_futs = (0..self.connections_count).map(|_| TLConnection::new(&self, semaphore.clone()));
        let connections = match try_join_all(conn_futs).await {
            Ok(conns) => {
                log::info!("[TLClient] {} connections initialized", conns.len());
                conns
            }
            Err(err) => bail_ton!("[TLClient] Failed to initialize TLConnection: {:?}", err),
        };

        let inner = Inner {
            rnd: Mutex::new(StdRng::from_rng(&mut rand::rng())),
            connections,
            retry_strategy: self.retry_strategy,
        };
        Ok(TLClient { inner: Arc::new(inner) })
    }

    pub fn with_net_config(mut self, net_config: &TonNetConfig) -> TonResult<Self> {
        self.init_opts.config.net_config_json = net_config.to_json()?;
        Ok(self)
    }

    pub fn with_keystore_type(mut self, keystore_type: TLKeyStoreType) -> Self {
        self.init_opts.keystore_type = keystore_type;
        self
    }

    pub fn with_tl_config(mut self, tl_config: TLConfig) -> Self {
        self.init_opts.config = tl_config;
        self
    }
}

async fn update_net_config(builder: &Builder) -> TonResult<Option<TonNetConfig>> {
    log::info!("Updating init_block...");
    let net_config = TonNetConfig::new(&builder.init_opts.config.net_config_json)?;
    let cur_init_seqno = net_config.get_init_block_seqno();

    let lite_client = LiteClient::builder()?.with_net_config(net_config.clone()).build()?;
    let lite_client_ref = &lite_client;

    let mut futs = vec![];
    for _ in net_config.lite_endpoints.iter() {
        let future = async {
            let mc_info = lite_client_ref.get_mc_info().await?;
            let block = lite_client_ref.get_block(mc_info.last, None).await?;
            lite_client_ref.lookup_mc_block(block.data.info.prev_key_block_seqno).await
        };
        futs.push(future);
    }
    let exec_timeout = Duration::from_secs(builder.update_init_block_timeout_sec.saturating_sub(1));
    let key_block_ids = tokio::time::timeout(exec_timeout, join_all(futs)).await?;
    let mut max_block: Option<BlockIdExt> = None;
    for block_id_res in &key_block_ids {
        match block_id_res {
            Ok(block_id) => {
                if max_block.is_none() || max_block.as_ref().unwrap().seqno < block_id.seqno {
                    max_block = Some(block_id.clone());
                }
            }
            Err(err) => log::warn!("Failed to get recent_init_block from node: {err:?}"),
        }
    }

    if let Some(block) = max_block {
        log::info!("Got new init_block for TonNetConfig: {} -> {}", cur_init_seqno, block.seqno);
        let mut net_conf = net_config.clone();
        net_conf.set_init_block(&block);
        return Ok(Some(net_conf));
    }
    Ok(None)
}
