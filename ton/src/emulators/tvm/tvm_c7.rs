use crate::emulators::emul_bc_config::EmulBCConfig;
use crate::errors::TonError;
use std::time::{SystemTime, UNIX_EPOCH};
use ton_lib_core::cell::TonHash;
use ton_lib_core::types::TonAddress;

#[derive(Clone, Debug)]
pub struct TVMEmulatorC7 {
    pub address: TonAddress,
    pub unix_time: u32,
    pub balance: u64,
    pub rand_seed: TonHash,
    pub config: EmulBCConfig,
}

impl TVMEmulatorC7 {
    pub fn new(address: TonAddress, config: EmulBCConfig) -> Result<Self, TonError> {
        Ok(Self {
            address,
            unix_time: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u32,
            balance: 0,
            rand_seed: TonHash::ZERO,
            config,
        })
    }
}
