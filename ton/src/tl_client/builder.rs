use crate::bail_ton;
use crate::block_tlb::{BlockIdExt, BlockInfo};
use crate::errors::{TonError, TonResult};
use crate::lite_client::{LiteClient, LiteClientConfig};
use crate::net_config::TonNetConfig;
use crate::tl_client::tl::{TLConfig, TLKeyStoreType, TLOptions};
use crate::tl_client::{Inner, LiteNodeFilter, RetryStrategy, TLCallbacksStore, TLClient, TLConnection};
use derive_setters::Setters;
use futures_util::future::{join_all, try_join_all};
use rand::prelude::StdRng;
use rand::SeedableRng;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::Semaphore;
use ton_core::cell::TonCell;
use ton_core::errors::TonCoreError;
use ton_core::traits::tlb::TLB;
use ton_liteapi::tl::response::BlockData;

#[derive(Setters)]
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
            std::fs::create_dir_all(directory)?
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

impl Debug for Builder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TLClientConfig")
            .field("init_opts", &self.init_opts)
            .field("connection_check", &self.connection_check)
            .field("connections_count", &self.connections_count)
            .field("max_parallel_requests", &self.max_parallel_requests)
            .field("retry_strategy", &self.retry_strategy)
            .field("update_init_block", &self.update_init_block)
            .field("update_init_block_timeout_sec", &self.update_init_block_timeout_sec)
            .field("tonlib_verbosity_level", &self.tonlib_verbosity_level)
            .field("callbacks_cnt", &self.callbacks.callbacks.len())
            .finish()
    }
}

async fn update_net_config(builder: &Builder) -> TonResult<Option<TonNetConfig>> {
    log::info!("Updating init_block...");
    let net_config = TonNetConfig::new(&builder.init_opts.config.net_config_json)?;
    let cur_init_seqno = net_config.get_init_block_seqno();

    let lite_config = LiteClientConfig::new(net_config)?;
    let lite_client = LiteClient::new(lite_config.clone())?;
    let lite_client_ref = &lite_client;

    let mut futs = Vec::with_capacity(lite_config.net_config.lite_endpoints.len());
    for _ in lite_config.net_config.lite_endpoints.iter() {
        let future = async {
            let mc_info = lite_client_ref.get_mc_info().await?;
            let block = lite_client_ref.get_block(mc_info.last, None).await?;
            let seqno = parse_key_block_seqno(block)?;
            lite_client_ref.lookup_mc_block(seqno).await
        };
        futs.push(future);
    }
    let exec_timeout = Duration::from_secs(builder.update_init_block_timeout_sec.saturating_sub(1));
    let results = tokio::time::timeout(exec_timeout, join_all(futs)).await?;
    let mut max_block: Option<BlockIdExt> = None;
    for res in &results {
        match res {
            Ok(block) => {
                if max_block.is_none() || max_block.as_ref().unwrap().seqno < block.seqno {
                    max_block = Some(block.clone());
                }
            }
            Err(err) => log::warn!("Failed to get recent_init_block from node: {err:?}"),
        }
    }

    if let Some(block) = max_block {
        log::info!("Got new init_block for TonNetConfig: {} -> {}", cur_init_seqno, block.seqno);
        let mut net_conf = lite_config.net_config.clone();
        net_conf.set_init_block(&block);
        return Ok(Some(net_conf));
    }
    Ok(None)
}

fn parse_key_block_seqno(block: BlockData) -> Result<u32, TonError> {
    let block_cell = TonCell::from_boc(block.data)?;
    if block_cell.refs().is_empty() {
        return Err(TonError::Custom("No refs in block cell".to_string()));
        // TODO make proper block parser
    }
    let mut parser = block_cell.refs()[0].parser();
    let tag: usize = parser.read_num(32)?;
    if tag != BlockInfo::PREFIX.value {
        return Err(TonCoreError::TLBWrongPrefix {
            exp: BlockInfo::PREFIX.value,
            given: tag,
            bits_exp: BlockInfo::PREFIX.bits_len,
            bits_left: parser.data_bits_left()? + 32,
        }
        .into());
    }
    // version(32), merge_info(8), flags(8), seqno(32), vert_seqno(32), shard(104), utime(32), start/end lt(128),
    // validator_list_hash(32), catchain_seqno(32), min_ref_mc_seqno(32)
    parser.read_bits(32 + 8 + 8 + 32 + 32 + 104 + 32 + 128 + 32 + 32 + 32)?;
    let key_block_seqno = parser.read_num(32)?;
    Ok(key_block_seqno)
}
