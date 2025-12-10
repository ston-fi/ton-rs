use std::time::Duration;

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

impl Default for LiteReqParams {
    fn default() -> Self { Self::new(10, 100, 5000) }
}
