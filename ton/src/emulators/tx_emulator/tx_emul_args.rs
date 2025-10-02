use crate::emulators::emul_bc_config::EmulBCConfig;
use std::fmt::{Display, Formatter};
use ton_lib_core::cell::TonHash;

#[derive(Debug, Clone)]
pub struct TXEmulOrdArgs {
    pub in_msg_boc: Vec<u8>,
    pub emul_args: TXEmulArgs,
}

pub struct TXEmulTickTockArgs {
    pub is_tock: bool,
    pub emul_args: TXEmulArgs,
}

#[derive(Debug, Clone)]
pub struct TXEmulArgs {
    pub shard_account_boc: Vec<u8>,
    pub bc_config: EmulBCConfig,
    pub rand_seed: TonHash,
    pub utime: u32,
    pub lt: u64,
    pub ignore_chksig: bool,
    pub prev_blocks_boc: Option<Vec<u8>>,
    pub libs_boc: Option<Vec<u8>>,
}

impl Display for TXEmulArgs {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let shard_acc_str = hex::encode(&self.shard_account_boc);

        let prev_blocks_str = match &self.prev_blocks_boc {
            None => "None",
            Some(boc) => &hex::encode(boc),
        };

        let libs_str = match &self.libs_boc {
            None => "None",
            Some(boc) => &hex::encode(boc),
        };

        f.write_fmt(format_args!(
            "shard_account_boc: {}, bc_config: {}, rand_seed: {}, utime: {}, lt: {}, ignore_chksig: {}, prev_blocks_boc: {}, libs_boc: {}",
            shard_acc_str, self.bc_config.to_string_lossy(), self.rand_seed, self.utime, self.lt, self.ignore_chksig, prev_blocks_str, libs_str
        ))
    }
}

impl Display for TXEmulOrdArgs {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("in_msg_boc : {}, emul_args: {}", hex::encode(&self.in_msg_boc), &self.emul_args))
    }
}

impl Display for TXEmulTickTockArgs {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("is_tock: {}, emul_args: {}", self.is_tock, &self.emul_args))
    }
}
