/*use crate::lite::client::connection::Descriptor;
use prometheus::core::Collector;
use prometheus::{HistogramOpts, HistogramVec, IntCounterVec, IntGauge, Opts};
use std::fmt;
use std::time::Duration;
use ton_liteapi::tl_api::request::Request;
use tongrid_core::metrics::DURATION_BUCKETS_1MS_20S;
use crate::errors::TonLibError;

#[derive(Clone)]
pub struct LiteClientMetrics {
    lite_client_max_connections_count: IntGauge,
    pub lite_client_wait_connection_ms: HistogramVec,
    pub lite_client_requests: IntCounterVec,
    pub lite_client_requests_duration_ms: HistogramVec,
    pub lite_client_max_seen_mc_seqno: IntGauge,
    pub lite_client_free_connections: IntGauge,
}

impl LiteClientMetrics {
    pub fn new(conn_count: usize) -> Result<Self, TonLibError> {
        let labels = vec!["status", "endpoint_type", "endpoint_ip", "method", "retry_num"];

        let res = Self {
            lite_client_max_connections_count: IntGauge::new("lite_max_connections_count", "Max available connections")?,
            lite_client_wait_connection_ms: HistogramVec::new(
                HistogramOpts::new("lite_wait_connection_ms", "Time to wait for a connection")
                    .buckets(DURATION_BUCKETS_1MS_20S.clone()),
                &[],
            )?,

            lite_client_requests: IntCounterVec::new(
                Opts::new("lite_requests", "Amount of lite requests processed"),
                &labels,
            )?,
            lite_client_requests_duration_ms: HistogramVec::new(
                HistogramOpts::new("lite_requests_duration_ms", "Time since external call to response")
                    .buckets(DURATION_BUCKETS_1MS_20S.clone()),
                &labels,
            )?,
            lite_client_max_seen_mc_seqno: IntGauge::new("lite_max_seen_mc_seqno", "Maximal seqno seen")?,
            lite_client_free_connections: IntGauge::new("lite_free_connections", "Number of available connections")?,
        };

        res.lite_client_max_connections_count.set(conn_count as i64);

        Ok(res)
    }

    pub fn update(
        &self,
        req: &Request,
        retry_num: u32,
        status: Status,
        duration: Duration,
        descriptor: &Descriptor,
        connection_wait_time: Duration,
    ) {
        let method = req_to_label_values(req);
        let label_values = &[
            &status.to_string(),
            &descriptor.endpoint_type.to_string(),
            &descriptor.endpoint_ip.to_string(),
            method,
            &retry_num.to_string(),
        ];

        self.lite_client_requests_duration_ms
            .with_label_values(label_values)
            .observe(duration.as_millis() as f64);

        self.lite_client_free_connections.inc();

        self.update_lite_requests_counter(req, retry_num, status, descriptor);

        self.update_wait_conn_duration(connection_wait_time);
    }

    fn update_wait_conn_duration(&self, duration: Duration) {
        self.lite_client_wait_connection_ms
            .with_label_values(&[])
            .observe(duration.as_millis() as f64);
    }

    pub fn dec_available_connections(&self) {
        self.lite_client_free_connections.dec()
    }

    fn update_lite_requests_counter(&self, req: &Request, retry_num: u32, status: Status, descriptor: &Descriptor) {
        let method = req_to_label_values(req);

        let vals = &[
            &status.to_string(),
            &descriptor.endpoint_type.to_string(),
            &descriptor.endpoint_ip.to_string(),
            method,
            &retry_num.to_string(),
        ];
        self.lite_client_requests.with_label_values(vals).inc();
    }

    pub fn provide_metrics(&self) -> Vec<Box<dyn Collector>> {
        vec![
            Box::new(self.lite_client_max_connections_count.clone()),
            Box::new(self.lite_client_wait_connection_ms.clone()),
            Box::new(self.lite_client_requests.clone()),
            Box::new(self.lite_client_requests_duration_ms.clone()),
            Box::new(self.lite_client_max_seen_mc_seqno.clone()),
            Box::new(self.lite_client_free_connections.clone()),
        ]
    }
}

fn req_to_label_values(req: &Request) -> &'static str {
    match req {
        Request::GetMasterchainInfo => "get_masterchain_info",
        Request::GetMasterchainInfoExt(_) => "get_masterchain_info_ext",
        Request::GetTime => "get_time",
        Request::GetVersion => "get_version",
        Request::GetBlock(_) => "get_block",
        Request::GetState(_) => "get_state",
        Request::GetBlockHeader(_) => "get_block_header",
        Request::SendMessage(_) => "send_message",
        Request::GetAccountState(_) => "get_account_state",
        Request::RunSmcMethod(_) => "run_smc_method",
        Request::GetShardInfo(_) => "get_shard_info",
        Request::GetAllShardsInfo(_) => "get_all_shards_info",
        Request::GetOneTransaction(_) => "get_one_transaction",
        Request::GetTransactions(_) => "get_transactions",
        Request::LookupBlock(_) => "lookup_block",
        Request::ListBlockTransactions(_) => "list_block_transactions",
        Request::GetBlockProof(_) => "get_block_proof",
        Request::GetConfigAll(_) => "get_config_all",
        Request::GetConfigParams(_) => "get_config_params",
        Request::GetValidatorStats(_) => "get_validator_stats",
        Request::GetLibraries(_) => "get_libraries",
        Request::GetLibrariesWithProof(_) => "get_libraries_with_proof",
    }
}

pub enum Status {
    Ok,
    Error,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Status::Ok => "Ok",
            Status::Error => "Error",
        };
        write!(f, "{}", s)
    }
}
*/
