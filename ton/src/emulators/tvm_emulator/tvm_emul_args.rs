use crate::emulators::tvm_emulator::{TVMEmulatorC7, TVMGetMethodID};
use std::sync::Arc;

pub struct TVMState {
    pub code_boc: Arc<Vec<u8>>,
    pub data_boc: Arc<Vec<u8>>,
    pub c7: TVMEmulatorC7,
    pub libs_boc: Option<Arc<Vec<u8>>>,
    pub debug_enabled: Option<bool>,
    pub gas_limit: Option<u64>,
}
