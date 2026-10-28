mod c7_prev_blocks_info;
mod tx_emul_args;
mod tx_emul_response;

pub use c7_prev_blocks_info::*;
pub use tx_emul_args::*;
pub use tx_emul_response::*;

use crate::emulators::emul_bc_config::EmulBCConfig;
use crate::emulators::emul_utils::{convert_emulator_response, make_base64_c_str, set_param_failed};
use crate::errors::{TonError, TonResult};
use std::ffi::CString;
use std::sync::Arc;
use ton_core::cell::TonHash;
use ton_core::constants::TON_ZERO_CONFIG_BOC_B64;
use tonlib_sys::*;

pub struct TXEmulator {
    emulator: *mut std::ffi::c_void,
    cur_bc_config_hash: u64,
    cur_random_seed: TonHash,
    cur_utime: u32,
    cur_lt: u64,
    cur_libs_hash: u64,
    cur_ignore_chksig: bool,
    cur_prev_blocks_info_hash: u64,
}

impl TXEmulator {
    pub fn new(log_level: u32, debug_enabled: bool) -> TonResult<Self> {
        let zero_config = Arc::new(CString::new(TON_ZERO_CONFIG_BOC_B64)?);
        let ptr = unsafe { transaction_emulator_create(zero_config.as_ptr(), log_level) };
        if ptr.is_null() {
            return Err(TonError::EmulatorCreationFailed);
        }
        let mut emulator = Self {
            emulator: ptr,
            cur_bc_config_hash: calc_hash(zero_config.as_bytes()),
            cur_random_seed: Default::default(),
            cur_utime: 0,
            cur_lt: 0,
            cur_libs_hash: calc_hash([]),
            cur_ignore_chksig: false,
            cur_prev_blocks_info_hash: 0,
        };
        emulator.set_debug_enabled(debug_enabled)?;
        Ok(emulator)
    }

    /// Shard-account schema: <https://github.com/ton-blockchain/ton/blob/cee4c674ea999fecc072968677a34a7545ac9c4d/crypto/block/block.tlb#L275>.
    /// You can't emulate tick-tock tx using this method
    pub fn emulate_ord(&mut self, args: &TXEmulOrdArgs) -> TonResult<TXEmulationResponse> {
        self.prepare_emulator(&args.emul_args)?;
        let state_c_str = make_base64_c_str(&args.emul_args.shard_account_boc)?;
        let in_msg_c_str = make_base64_c_str(&args.in_msg_boc)?;
        let response_ptr = unsafe {
            transaction_emulator_emulate_transaction(self.emulator, state_c_str.as_ptr(), in_msg_c_str.as_ptr())
        };
        let response_str = convert_emulator_response(response_ptr)?;
        TXEmulationResponse::from_json(response_str)
    }

    pub fn emulate_ticktock(&mut self, args: &TXEmulTickTockArgs) -> TonResult<TXEmulationResponse> {
        self.prepare_emulator(&args.emul_args)?;
        let state_c_str = make_base64_c_str(&args.emul_args.shard_account_boc)?;
        let response_ptr = unsafe {
            transaction_emulator_emulate_tick_tock_transaction(self.emulator, state_c_str.as_ptr(), args.is_tock)
        };
        let response_str = convert_emulator_response(response_ptr)?;
        TXEmulationResponse::from_json(response_str)
    }
    fn prepare_emulator(&mut self, args: &TXEmulArgs) -> TonResult<()> {
        self.actualize_config(&args.bc_config)?;
        self.actualize_rand_seed(&args.rand_seed)?;
        self.actualize_utime(args.utime)?;
        self.actualize_lt(args.lt)?;
        if let Some(libs) = &args.libs_boc {
            self.actualize_libs(libs.as_ref())?;
        }
        self.actualize_ignore_chksig(args.ignore_chksig)?;
        if let Some(prev_blocks) = &args.c7_prev_blocks_info_boc {
            self.actualize_prev_blocks_info(prev_blocks.as_ref())?;
        }
        Ok(())
    }

    fn actualize_config(&mut self, config: &EmulBCConfig) -> TonResult<()> {
        let config_hash = calc_hash(config.as_bytes());
        if self.cur_bc_config_hash == config_hash {
            return Ok(());
        }
        match unsafe { transaction_emulator_set_config(self.emulator, config.as_ptr()) } {
            true => self.cur_bc_config_hash = config_hash,
            false => return set_param_failed("config"),
        }
        Ok(())
    }

    fn actualize_rand_seed(&mut self, rand_seed: &TonHash) -> TonResult<()> {
        if self.cur_random_seed == *rand_seed {
            return Ok(());
        }
        match unsafe {
            transaction_emulator_set_rand_seed(self.emulator, CString::new(hex::encode(rand_seed))?.as_ptr())
        } {
            true => self.cur_random_seed = TonHash::from_slice_sized(rand_seed.as_slice_sized()),
            false => return set_param_failed("rand_seed"),
        }
        Ok(())
    }

    fn actualize_utime(&mut self, utime: u32) -> TonResult<()> {
        if self.cur_utime == utime {
            return Ok(());
        }
        match unsafe { transaction_emulator_set_unixtime(self.emulator, utime) } {
            true => self.cur_utime = utime,
            false => return set_param_failed("utime"),
        }
        Ok(())
    }

    fn actualize_lt(&mut self, lt: u64) -> TonResult<()> {
        if self.cur_lt == lt {
            return Ok(());
        }
        match unsafe { transaction_emulator_set_lt(self.emulator, lt) } {
            true => self.cur_lt = lt,
            false => return set_param_failed("lt"),
        }
        Ok(())
    }

    fn actualize_libs(&mut self, libs_boc: &[u8]) -> TonResult<()> {
        let libs_hash = calc_hash(libs_boc);
        if self.cur_libs_hash == libs_hash {
            return Ok(());
        }
        let libs = make_base64_c_str(libs_boc)?;
        match unsafe { transaction_emulator_set_libs(self.emulator, libs.as_ptr()) } {
            true => self.cur_libs_hash = libs_hash,
            false => return set_param_failed("libs"),
        }
        Ok(())
    }

    fn set_debug_enabled(&mut self, debug_enabled: bool) -> TonResult<()> {
        match unsafe { transaction_emulator_set_debug_enabled(self.emulator, debug_enabled) } {
            true => Ok(()),
            false => set_param_failed("debug_enabled"),
        }
    }

    fn actualize_prev_blocks_info(&mut self, prev_blocks_info: &[u8]) -> TonResult<()> {
        let prev_blocks_hash = calc_hash(prev_blocks_info);
        if self.cur_prev_blocks_info_hash == prev_blocks_hash {
            return Ok(());
        }
        let block_info = make_base64_c_str(prev_blocks_info)?;
        match unsafe { transaction_emulator_set_prev_blocks_info(self.emulator, block_info.as_ptr()) } {
            true => self.cur_prev_blocks_info_hash = prev_blocks_hash,
            false => return set_param_failed("prev_blocks_info"),
        }
        Ok(())
    }

    fn actualize_ignore_chksig(&mut self, ignore: bool) -> TonResult<()> {
        if self.cur_ignore_chksig == ignore {
            return Ok(());
        }
        match unsafe { transaction_emulator_set_ignore_chksig(self.emulator, ignore) } {
            true => self.cur_ignore_chksig = ignore,
            false => return set_param_failed("ignore_chksig"),
        }
        Ok(())
    }
}

impl Drop for TXEmulator {
    fn drop(&mut self) {
        unsafe {
            transaction_emulator_destroy(self.emulator);
        }
    }
}
unsafe impl Send for TXEmulator {}
unsafe impl Sync for TXEmulator {}

fn calc_hash<T: AsRef<[u8]> + std::hash::Hash>(data: T) -> u64 {
    use std::hash::{DefaultHasher, Hasher};
    let mut s = DefaultHasher::new();
    data.hash(&mut s);
    s.finish()
}

#[cfg(test)]
#[path = "../tests/emulators/test_tx_emulator.rs"]
mod tests;
