mod tvm_c7;
mod tvm_method_id;
mod tvm_response;

pub use tvm_c7::*;
pub use tvm_method_id::*;
pub use tvm_response::*;

use crate::emulators::emul_utils::*;
use crate::errors::{TonError, TonResult};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use std::ffi::CString;
use std::sync::Arc;
use tonlib_sys::*;

#[derive(Clone, Debug)]
pub struct TVMState {
    pub code_boc: Arc<Vec<u8>>,
    pub data_boc: Arc<Vec<u8>>,
    pub c7: TVMEmulatorC7,
    pub libs_boc: Option<Arc<Vec<u8>>>,
    pub debug_enabled: Option<bool>,
    pub gas_limit: Option<u64>,
}

#[derive(Debug)]
pub struct TVMEmulator {
    ptr: *mut std::os::raw::c_void,
}

const DEFAULT_TVM_LOG_VERBOSITY: u32 = 1;

impl TVMEmulator {
    pub fn from_state(state: &TVMState) -> TonResult<Self> {
        let mut emulator = TVMEmulator::new(&state.code_boc, &state.data_boc, &state.c7)?;
        if let Some(gas_limit) = state.gas_limit {
            emulator.set_gas_limit(gas_limit)?;
        }
        if let Some(debug_enabled) = state.debug_enabled {
            emulator.set_debug_enabled(debug_enabled)?;
        }
        if let Some(libs_boc) = state.libs_boc.as_deref() {
            emulator.set_libs(libs_boc)?;
        }
        Ok(emulator)
    }

    pub fn new(code_boc: &[u8], data_boc: &[u8], c7: &TVMEmulatorC7) -> TonResult<Self> {
        let code = CString::new(STANDARD.encode(code_boc.as_ref()))?;
        let data = CString::new(STANDARD.encode(data_boc.as_ref()))?;
        let ptr = unsafe { tvm_emulator_create(code.as_ptr(), data.as_ptr(), DEFAULT_TVM_LOG_VERBOSITY) };
        if ptr.is_null() {
            return Err(TonError::EmulatorCreationFailed);
        }
        let mut emulator = TVMEmulator { ptr };
        emulator.set_c7(c7)?;
        Ok(emulator)
    }

    pub fn set_c7(&mut self, c7: &TVMEmulatorC7) -> TonResult<()> {
        let address = CString::new(c7.address.to_hex().as_bytes())?;
        let seed = CString::new(c7.rand_seed.to_hex().as_bytes())?;
        let success = unsafe {
            tvm_emulator_set_c7(self.ptr, address.as_ptr(), c7.unix_time, c7.balance, seed.as_ptr(), c7.config.as_ptr())
        };
        match success {
            true => Ok(()),
            false => set_param_failed("c7"),
        }
    }

    pub fn set_debug_enabled(&mut self, enabled: bool) -> TonResult<()> {
        let success = unsafe { tvm_emulator_set_debug_enabled(self.ptr, enabled as i32) };
        match success {
            true => Ok(()),
            false => set_param_failed("debug_enabled"),
        }
    }

    pub fn set_gas_limit(&mut self, limit: u64) -> TonResult<()> {
        let success = unsafe { tvm_emulator_set_gas_limit(self.ptr, limit) };
        match success {
            true => Ok(()),
            false => set_param_failed("gas_limit"),
        }
    }

    pub fn set_libs(&mut self, libs_boc: &[u8]) -> TonResult<()> {
        let libs = CString::new(STANDARD.encode(libs_boc))?;
        let success = unsafe { tvm_emulator_set_libraries(self.ptr, libs.as_ptr()) };
        match success {
            true => Ok(()),
            false => set_param_failed("libs"),
        }
    }

    pub fn emul_get_method<T>(&mut self, method: T, stack_boc: &[u8]) -> TonResult<TVMGetMethodResponse>
    where
        T: Into<TVMGetMethodID>,
    {
        let tvm_method = method.into();
        log::trace!("[TVMEmulator][run_get_method]: method: {tvm_method}, stack: {stack_boc:?}");
        let stack = make_base64_c_str(stack_boc)?;
        let response_ptr = unsafe { tvm_emulator_run_get_method(self.ptr, tvm_method.to_id(), stack.as_ptr()) };
        let json_str = convert_emulator_response(response_ptr)?;
        log::trace!("[TVMEmulator][run_get_method]: method: {tvm_method}, stack_boc: {stack_boc:?}, rsp: {json_str}");
        TVMGetMethodResponse::from_json(json_str)
    }

    pub fn emul_send_int_msg(&mut self, msg_boc: &[u8], amount: u64) -> TonResult<TVMSendMsgResponse> {
        log::trace!("[TVMEmulator][send_int_msg]: msg_boc: {msg_boc:?}, amount: {amount}");
        let msg = CString::new(STANDARD.encode(msg_boc))?;

        let c_str = unsafe { tvm_emulator_send_internal_message(self.ptr, msg.as_ptr(), amount) };
        let json_str = convert_emulator_response(c_str)?;
        log::trace!("[TVMEmulator][send_int_msg]: msg_boc: {msg_boc:?}, amount: {amount}, rsp: {json_str}");
        TVMSendMsgResponse::from_json(json_str)
    }

    pub fn emul_send_ext_msg(&mut self, msg_boc: &[u8]) -> TonResult<TVMSendMsgResponse> {
        log::trace!("[TVMEmulator][send_ext_msg]: msg_boc: {msg_boc:?}");
        let msg = make_base64_c_str(msg_boc)?;
        let response_ptr = unsafe { tvm_emulator_send_external_message(self.ptr, msg.as_ptr()) };
        let json_str = convert_emulator_response(response_ptr)?;
        log::trace!("[TVMEmulator][send_ext_msg]: msg_boc: {msg_boc:?}, rsp: {json_str}");
        TVMSendMsgResponse::from_json(json_str)
    }
}

impl Drop for TVMEmulator {
    fn drop(&mut self) {
        unsafe { tvm_emulator_destroy(self.ptr) };
    }
}
unsafe impl Send for TVMEmulator {}
unsafe impl Sync for TVMEmulator {}

#[cfg(test)]
#[path = "../tests/emulators/test_tvm_emulator.rs"]
mod tests;
