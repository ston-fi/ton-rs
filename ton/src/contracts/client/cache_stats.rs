use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;

#[derive(Default)]
pub(crate) struct CacheStats {
    pub state_latest_req: AtomicUsize,
    pub state_latest_miss: AtomicUsize,
    pub state_by_tx_req: AtomicUsize,
    pub state_by_tx_miss: AtomicUsize,
}

impl CacheStats {
    pub(crate) fn export(&self, latest_entry_count: usize, by_tx_entry_count: usize) -> HashMap<String, usize> {
        HashMap::from([
            ("state_latest_req".to_string(), self.state_latest_req.load(Relaxed)),
            ("state_latest_miss".to_string(), self.state_latest_miss.load(Relaxed)),
            ("state_latest_entry_count".to_string(), latest_entry_count),
            ("state_by_tx_req".to_string(), self.state_by_tx_req.load(Relaxed)),
            ("state_by_tx_miss".to_string(), self.state_by_tx_miss.load(Relaxed)),
            ("state_by_tx_entry_count".to_string(), by_tx_entry_count),
        ])
    }
}
