use crate::errors::TonResult;
use crate::lite_client::connection::Connection;
use crate::lite_client::req_params::LiteReqParams;
use crate::lite_client::{Inner, LiteClient, WAIT_CONNECTION_MS};
use crate::net_config::TonNetConfig;
use auto_pool::config::{AutoPoolConfig, PickStrategy};
use auto_pool::pool::AutoPool;
use derive_setters::Setters;
use std::cmp::max;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::time::Duration;

#[derive(Setters, Debug, Clone)]
#[setters(prefix = "with_", strip_option)]
pub struct Builder {
    net_config: TonNetConfig,
    connections_per_node: u32,
    conn_timeout: Duration,
    default_req_params: LiteReqParams,
    last_seqno_polling_period: Duration,
    metrics_enabled: bool,
}

impl Builder {
    pub(super) fn new() -> TonResult<Self> {
        let builder = Self {
            net_config: TonNetConfig::new_default(true)?,
            connections_per_node: 1,
            conn_timeout: Duration::from_millis(500),
            default_req_params: LiteReqParams::default(),
            last_seqno_polling_period: Duration::from_millis(5000),
            metrics_enabled: true,
        };
        Ok(builder)
    }

    pub fn with_net_config_json(mut self, json: &str) -> TonResult<Self> {
        self.net_config = TonNetConfig::new(json)?;
        Ok(self)
    }

    pub fn with_net_config_path(mut self, path: &str) -> TonResult<Self> {
        self.net_config = TonNetConfig::from_path(path)?;
        Ok(self)
    }

    pub fn build(self) -> TonResult<LiteClient> {
        let conn_per_node = max(1, self.connections_per_node);
        log::info!(
            "Creating LiteClient with {} conns per node; nodes_cnt: {}, default_req_params: {:?}",
            conn_per_node,
            self.net_config.lite_endpoints.len(),
            self.default_req_params,
        );

        let mut connections = Vec::new();
        for _ in 0..conn_per_node {
            for endpoint in &self.net_config.lite_endpoints {
                let conn = Connection::new(endpoint.clone(), self.conn_timeout)?;
                connections.push(conn);
            }
        }
        let ap_config = AutoPoolConfig {
            wait_duration: Duration::MAX,
            lock_duration: Duration::from_millis(2),
            sleep_duration: Duration::from_millis(WAIT_CONNECTION_MS),
            pick_strategy: PickStrategy::RANDOM,
        };

        let connection_pool = AutoPool::new_with_config(ap_config, connections);
        let inner = Inner {
            default_req_params: self.default_req_params,
            conn_pool: connection_pool,
            global_req_id: AtomicU64::new(0),
        };
        Ok(LiteClient(Arc::new(inner)))
    }
}
