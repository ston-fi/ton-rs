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
