use crate::errors::TonError;
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering::Relaxed};
use std::time::{Duration, Instant};
use ton_liteapi::tl::request::Request;
use ton_liteapi::tl::response::Response;

pub(super) struct LiteClientMetrics {
    pub(super) known_nodes_count: u32,
    pub(super) connections_per_node: u32,
    pub(super) wait_connection_ms_total: AtomicU64,
    pub(super) ongoing_requests: AtomicU32,
    pub(super) requests_ok: DashMap<String, AtomicU64>,
    pub(super) requests_failed: DashMap<String, AtomicU64>,
    pub(super) requests_count_total: DashMap<String, AtomicU64>, // req_type -> req_count
    pub(super) requests_duration_ms_total: DashMap<String, AtomicU64>, // req_type -> sum(duration)
    pub(super) retries_count_total: DashMap<String, AtomicU64>,  // req_type -> req_count
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiteClientMetricsSnapshot {
    pub known_nodes_count: u32,
    pub connections_per_node: u32,
    pub wait_connection_ms_total: u64,
    pub ongoing_requests: u32,
    pub requests_ok: HashMap<String, u64>,
    pub requests_failed: HashMap<String, u64>,
    pub requests_count_total: HashMap<String, u64>,
    pub requests_duration_ms_total: HashMap<String, u64>,
    pub retries_count_total: HashMap<String, u64>,
}

impl LiteClientMetrics {
    pub(super) fn new(nodes_count: u32, conn_count: u32) -> Result<Self, TonError> {
        let res = Self {
            known_nodes_count: nodes_count,
            connections_per_node: conn_count,
            wait_connection_ms_total: Default::default(),
            ongoing_requests: Default::default(),
            requests_ok: Default::default(),
            requests_failed: Default::default(),
            requests_count_total: Default::default(),
            requests_duration_ms_total: Default::default(),
            retries_count_total: Default::default(),
        };
        Ok(res)
    }

    pub(super) fn snapshot(&self) -> LiteClientMetricsSnapshot {
        LiteClientMetricsSnapshot {
            known_nodes_count: self.known_nodes_count,
            connections_per_node: self.connections_per_node,
            wait_connection_ms_total: self.wait_connection_ms_total.load(Relaxed),
            ongoing_requests: self.ongoing_requests.load(Relaxed),
            requests_ok: snapshot_dashmap(&self.requests_ok),
            requests_failed: snapshot_dashmap(&self.requests_failed),
            requests_count_total: snapshot_dashmap(&self.requests_count_total),
            requests_duration_ms_total: snapshot_dashmap(&self.requests_duration_ms_total),
            retries_count_total: snapshot_dashmap(&self.retries_count_total),
        }
    }
}

pub(super) struct MetricGuard<'a> {
    metrics: &'a LiteClientMetrics,
    req_str: String,
    started: Instant,
    ok: bool,
}

impl<'a> MetricGuard<'a> {
    pub(super) fn new(metrics: &'a LiteClientMetrics, req: &Request, is_retry: bool) -> Self {
        let req_str = req_to_str(req).to_string();

        metrics.ongoing_requests.fetch_add(1, Relaxed);
        increment_dashmap(&metrics.requests_count_total, &req_str, 1);
        if is_retry {
            increment_dashmap(&metrics.retries_count_total, &req_str, 1);
        }

        Self {
            metrics,
            req_str,
            started: Instant::now(),
            ok: false,
        }
    }

    pub(super) fn record_result(&mut self, result: &Result<Response, TonError>) {
        self.ok = matches!(result, Ok(response) if !matches!(response, Response::Error(_)));
    }

    pub(super) fn record_connection_wait(&self, duration: Duration) {
        self.metrics.wait_connection_ms_total.fetch_add(duration_ms_u64(duration), Relaxed);
    }
}

impl Drop for MetricGuard<'_> {
    fn drop(&mut self) {
        let count_map = if self.ok {
            &self.metrics.requests_ok
        } else {
            &self.metrics.requests_failed
        };
        increment_dashmap(count_map, &self.req_str, 1);
        increment_dashmap(
            &self.metrics.requests_duration_ms_total,
            &self.req_str,
            duration_ms_u64(self.started.elapsed()),
        );
        self.metrics.ongoing_requests.fetch_sub(1, Relaxed);
    }
}

fn increment_dashmap(map: &DashMap<String, AtomicU64>, key: &str, value: u64) {
    map.entry(key.to_string()).or_default().fetch_add(value, Relaxed);
}

fn snapshot_dashmap(map: &DashMap<String, AtomicU64>) -> HashMap<String, u64> {
    map.iter().map(|entry| (entry.key().clone(), entry.value().load(Relaxed))).collect()
}

fn req_to_str(req: &Request) -> &'static str {
    match req {
        Request::GetMasterchainInfo => "get_mc_info",
        Request::GetMasterchainInfoExt(_) => "get_mc_info_ext",
        Request::GetTime => "get_time",
        Request::GetVersion => "get_version",
        Request::GetBlock(_) => "get_block",
        Request::GetState(_) => "get_state",
        Request::GetBlockHeader(_) => "get_block_header",
        Request::SendMessage(_) => "send_msg",
        Request::GetAccountState(_) => "get_account_state",
        Request::GetAccountStatePrunned(_) => "get_account_state_prunned",
        Request::RunSmcMethod(_) => "run_smc_method",
        Request::GetShardInfo(_) => "get_shard_info",
        Request::GetAllShardsInfo(_) => "get_all_shards_info",
        Request::GetOneTransaction(_) => "get_one_tx",
        Request::GetTransactions(_) => "get_txs",
        Request::LookupBlock(_) => "lookup_block",
        Request::LookupBlockWithProof(_) => "lookup_block_with_proof",
        Request::ListBlockTransactions(_) => "list_block_txs",
        Request::ListBlockTransactionsExt(_) => "list_block_txs_ext",
        Request::GetBlockProof(_) => "get_block_proof",
        Request::GetConfigAll(_) => "get_config_all",
        Request::GetConfigParams(_) => "get_config_params",
        Request::GetValidatorStats(_) => "get_validator_stats",
        Request::GetLibraries(_) => "get_libs",
        Request::GetLibrariesWithProof(_) => "get_libs_with_proof",
        Request::GetOutMsgQueueSizes(_) => "get_out_msg_queue_sizes",
        Request::GetBlockOutMsgQueueSize(_) => "get_block_out_msg_queue_sizes",
        Request::GetShardBlockProof(_) => "get_shard_block_proof",
        Request::GetDispatchQueueInfo(_) => "get_dispatch_queue_info",
        Request::GetDispatchQueueMessages(_) => "get_dispatch_queue_msgs",
    }
}

fn duration_ms_u64(duration: Duration) -> u64 { duration.as_millis().min(u64::MAX as u128) as u64 }

#[cfg(test)]
mod tests {
    use super::*;
    use ton_liteapi::tl::common::String as TlString;
    use ton_liteapi::tl::response::{Error, Response};

    #[test]
    fn test_lite_client_metrics_counts_response_error_as_failed() {
        let metrics = LiteClientMetrics::new(1, 1).unwrap();
        let response = Ok(Response::Error(Error {
            code: -1,
            message: TlString::from("lite server error"),
        }));

        {
            let mut guard = MetricGuard::new(&metrics, &Request::GetMasterchainInfo, false);
            guard.record_result(&response);
        }

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.requests_failed.get("get_mc_info"), Some(&1));
        assert_eq!(snapshot.requests_ok.get("get_mc_info"), None);
        assert_eq!(snapshot.requests_count_total.get("get_mc_info"), Some(&1));
        assert_eq!(snapshot.ongoing_requests, 0);
    }
}
