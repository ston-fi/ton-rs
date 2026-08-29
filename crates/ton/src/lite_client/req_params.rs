use std::time::Duration;

/// Retry and timeout settings for one lite-server request.
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub struct LiteReqParams {
    pub retries_count: u32,
    pub retry_waiting: Duration,
    pub query_timeout: Duration,
}

impl LiteReqParams {
    /// Creates request settings from millisecond durations.
    pub fn new(retries: u32, retry_waiting: u64, query_timeout: u64) -> Self {
        Self {
            retries_count: retries,
            retry_waiting: Duration::from_millis(retry_waiting),
            query_timeout: Duration::from_millis(query_timeout),
        }
    }
}

impl Default for LiteReqParams {
    fn default() -> Self {
        Self::new(10, 100, 5000)
    }
}
