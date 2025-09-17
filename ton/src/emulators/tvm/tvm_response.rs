use crate::block_tlb::TVMStack;
use crate::emulators::emul_utils::require_field;
use crate::errors::TonError;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use serde::{Deserialize, Serialize};
use ton_lib_core::traits::tlb::TLB;

#[derive(Debug, Clone)]
pub struct TVMGetMethodSuccess {
    pub vm_exit_code: i32,
    pub vm_log: Option<String>,
    pub stack_boc_base64: String,
    pub gas_used: i32,
    pub raw_response: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TVMGetMethodResponse {
    pub success: bool,
    pub vm_exit_code: Option<i32>,
    pub vm_log: Option<String>,
    pub stack: Option<String>,
    pub gas_used: Option<String>,
    pub missing_library: Option<String>,
    pub error: Option<String>,
    #[serde(skip)]
    pub raw_response: String,
}

impl TVMGetMethodResponse {
    pub fn from_json(json: String) -> Result<Self, TonError> {
        let mut value: Self = serde_json::from_str(&json)?;
        value.raw_response = json;
        Ok(value)
    }

    pub fn into_success(self) -> Result<TVMGetMethodSuccess, TonError> {
        if !self.success {
            return Err(TonError::EmulatorEmulationError {
                vm_exit_code: self.vm_exit_code,
                response_raw: self.raw_response,
            });
        }
        let vm_exit_code = require_field(self.vm_exit_code, "vm_exit_code", &self.raw_response)?;
        if vm_exit_code != 0 && vm_exit_code != 1 {
            return Err(TonError::EmulatorEmulationError {
                vm_exit_code: self.vm_exit_code,
                response_raw: self.raw_response,
            });
        }

        let vm_log = self.vm_log;
        let stack_boc_base64 = require_field(self.stack, "stack", &self.raw_response)?;
        let gas_used = require_field(self.gas_used, "gas_used", &self.raw_response)?.parse::<i32>()?;

        Ok(TVMGetMethodSuccess {
            vm_log,
            vm_exit_code,
            stack_boc_base64,
            gas_used,
            raw_response: self.raw_response,
        })
    }
}

impl TVMGetMethodSuccess {
    pub fn stack_parsed(&self) -> Result<TVMStack, TonError> { Ok(TVMStack::from_boc_b64(&self.stack_boc_base64)?) }

    pub fn stack_boc(&self) -> Result<Vec<u8>, TonError> {
        Ok(BASE64_STANDARD.decode(self.stack_boc_base64.as_bytes())?)
    }

    pub fn exit_success(&self) -> bool { self.vm_exit_code == 0 || self.vm_exit_code == 1 }
}

#[derive(Debug)]
pub struct TVMSendMsgSuccess {
    pub new_code_boc_base64: Option<String>,
    pub new_data_boc_base64: Option<String>,
    pub accepted: bool,
    pub vm_exit_code: i32,
    pub vm_log: String,
    pub missing_library: Option<String>,
    pub gas_used: i32,
    pub actions_boc_base64: Option<String>,
    pub raw_response: String,
}

impl TVMSendMsgSuccess {
    pub fn exit_success(&self) -> bool { self.vm_exit_code == 0 || self.vm_exit_code == 1 }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TVMSendMsgResponse {
    pub success: bool,
    pub new_code_boc_base64: Option<String>,
    pub new_data_boc_base64: Option<String>,
    pub accepted: Option<bool>,
    pub vm_exit_code: Option<i32>,
    pub vm_log: Option<String>,
    pub missing_library: Option<String>,
    pub gas_used: Option<String>,
    pub actions: Option<String>,
    pub error: Option<String>,
    #[serde(skip)]
    pub raw_response: String,
}

impl TVMSendMsgResponse {
    pub fn from_json(json: String) -> Result<Self, TonError> {
        let mut value: Self = serde_json::from_str(&json)?;
        value.raw_response = json;
        Ok(value)
    }

    pub fn into_success(self) -> Result<TVMSendMsgSuccess, TonError> {
        if !self.success {
            return Err(TonError::EmulatorEmulationError {
                vm_exit_code: self.vm_exit_code,
                response_raw: self.raw_response,
            });
        }

        let accepted = require_field(self.accepted, "accepted", &self.raw_response)?;
        let vm_log = require_field(self.vm_log, "vm_log", &self.raw_response)?;
        let vm_exit_code = require_field(self.vm_exit_code, "vm_exit_code", &self.raw_response)?;
        let missing_library = self.missing_library;
        let gas_used = require_field(self.gas_used, "gas_used", &self.raw_response)?.parse::<i32>()?;
        let actions_boc_base64 = self.actions;
        Ok(TVMSendMsgSuccess {
            new_code_boc_base64: self.new_code_boc_base64,
            new_data_boc_base64: self.new_data_boc_base64,
            accepted,
            vm_exit_code,
            vm_log,
            missing_library,
            gas_used,
            actions_boc_base64,
            raw_response: self.raw_response,
        })
    }
}
