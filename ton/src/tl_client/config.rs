use crate::net_config::TonNetConfig;
use crate::tl_client::callback::TLCallbacksStore;
use crate::tl_client::tl::*;
use std::fmt::Debug;
use std::time::Duration;

#[derive(Clone)]
pub struct TLClientConfig {
    pub init_opts: TLOptions,
    pub connection_check: LiteNodeFilter,
    pub connections_count: usize,
    pub max_parallel_requests: usize, // max_parallel_requests / connections_count = parallel requests per connection
    pub retry_strategy: RetryStrategy,
    pub update_init_block: bool,
    pub update_init_block_timeout_sec: u64,
    pub tonlib_verbosity_level: u32,
    pub callbacks: TLCallbacksStore,
}

#[derive(Debug, PartialEq, Clone)]
pub enum LiteNodeFilter {
    Healthy, // connect to healthy node only
    Archive, // connect to archive node only
}

#[derive(Debug, Clone)]
pub struct RetryStrategy {
    pub retry_count: usize,
    pub retry_waiting: Duration,
}

impl TLClientConfig {
    pub fn new(net_config_json: String, archive_only: bool) -> TLClientConfig {
        let connection_check = match archive_only {
            true => LiteNodeFilter::Archive,
            false => LiteNodeFilter::Healthy,
        };
        TLClientConfig {
            init_opts: TLOptions {
                config: TLConfig {
                    net_config_json,
                    blockchain_name: None,
                    use_callbacks_for_network: false,
                    ignore_cache: false,
                },
                keystore_type: TLKeyStoreType::Directory {
                    directory: "/tmp/tonlibjson_keystore".to_string(),
                },
            },
            connection_check,
            connections_count: 10,
            max_parallel_requests: 127,
            retry_strategy: RetryStrategy {
                retry_count: 10,
                retry_waiting: Duration::from_millis(100),
            },
            update_init_block: true,
            update_init_block_timeout_sec: 10,
            tonlib_verbosity_level: 1,
            callbacks: TLCallbacksStore::default(),
        }
    }
    pub fn new_mainnet(archive_only: bool) -> TLClientConfig {
        TLClientConfig::new(TonNetConfig::get_json(true), archive_only)
    }
    pub fn new_testnet(archive_only: bool) -> TLClientConfig {
        TLClientConfig::new(TonNetConfig::get_json(false), archive_only)
    }
}

impl Debug for TLClientConfig {
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
