//! Custom transport extension point.
use crate::error::TransportError;
use async_trait::async_trait;
use std::time::Duration;
/// Exclusively owns a device connection. Never interleave commands through another
/// handle. Return a complete response including its two status bytes. No automatic
/// retries or reconnection: a changed device must establish its identity again.
#[async_trait]
pub trait Transport: Send {
    /// Exchanges a complete short APDU within one total timeout budget.
    /// Dropping the future may leave a device prompt active; it does not cancel it.
    async fn exchange(&mut self, command: &[u8], timeout: Duration) -> Result<Vec<u8>, TransportError>;
}
