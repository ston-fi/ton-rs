use std::time::Duration;

use crate::clients::net_config::TonNetConfig;
use crate::errors::TonError;

#[derive(Debug, Clone)]
pub struct LiteClientConfig {
    pub net_config: TonNetConfig,
    pub connections_per_node: u32,
    pub conn_timeout: Duration,
    pub default_req_params: LiteReqParams,
    pub last_seqno_polling_period: Duration,
    pub metrics_enabled: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct LiteReqParams {
    pub retries_count: u32,
    pub retry_waiting: Duration,
    pub query_timeout: Duration,
}

impl LiteReqParams {
    pub fn new(retries: u32, retry_waiting: u64, query_timeout: u64) -> Self {
        Self {
            retries_count: retries,
            retry_waiting: Duration::from_millis(retry_waiting),
            query_timeout: Duration::from_millis(query_timeout),
        }
    }
}

impl LiteClientConfig {
    pub fn new(net_config: &str) -> Result<Self, TonError> {
        Ok(Self {
            net_config: TonNetConfig::new(net_config)?,
            connections_per_node: 1,
            conn_timeout: Duration::from_millis(500),
            default_req_params: LiteReqParams::new(10, 100, 5000),
            last_seqno_polling_period: Duration::from_millis(5000),
            metrics_enabled: true,
        })
    }
}
