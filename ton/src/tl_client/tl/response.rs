use crate::block_tlb::BlockIdExt;
use crate::errors::TonError;
use crate::tl_client::tl::ser_de::serde_block_id_ext;
use crate::tl_client::tl::tl_types::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::ffi::CStr;
use std::os::raw::c_char;
use strum::IntoStaticStr;

#[derive(IntoStaticStr, Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(tag = "@type", rename_all = "camelCase")]
pub enum TLResponse {
    // tonlib_api.tl_api, line 20
    Error {
        code: i32,
        message: String,
    },
    // tonlib_api.tl_api, line 21
    Ok(()),
    // tonlib_api.tl_api, line 30
    #[serde(rename = "options.info")]
    TLOptionsInfo(TLOptionsInfo),
    // tonlib_api.tl_api, line 51
    #[serde(rename = "ton.blockIdExt")]
    TLBlockIdExt(#[serde(with = "serde_block_id_ext")] BlockIdExt),
    // tonlib_api.tl_api, line 53
    #[serde(rename = "raw.fullAccountState")]
    TLRawFullAccountState(TLRawFullAccountState),
    // tonlib_api.tl_api, line 56
    #[serde(rename = "raw.transactions")]
    TLRawTxs(TLRawTxs),
    // tonlib_api.tl_api, line 58
    #[serde(rename = "raw.extMessageInfo")]
    TLRawExtMessageInfo(TLRawExtMessageInfo),
    // tonlib_api.tl_api, line 90
    #[serde(rename = "fullAccountState")]
    TLFullAccountState(Box<TLFullAccountState>),
    // tonlib_api.tl_api, line 167
    #[serde(rename = "tvm_types.cell")]
    TLTvmCell(()), // Unsupported
    // tonlib_api.tl_api, line 179
    #[serde(rename = "smc.info")]
    TLSmcInfo(TLSmcInfo),
    // tonlib_api.tl_api, line 184
    #[serde(rename = "smc.runResult")]
    TLSmcRunResult(()), // Unsupported
    // tonlib_api.tl_api, line 187
    #[serde(rename = "smc.libraryResult")]
    TLSmcLibraryResult(TLSmcLibraryResult),
    // tonlib_api.tl_api, line 191
    #[serde(rename = "smc.libraryResultExt")]
    TLSmcLibraryResultExt(TLSmcLibraryResultExt),
    // tonlib_api.tl_api, line 194
    #[serde(rename = "updateSyncState")]
    TLUpdateSyncState(TLUpdateSyncState),
    // tonlib_api.tl_api, line 203
    #[serde(rename = "liteServer.info")]
    TLLiteServerInfo(TLLiteServerInfo),
    // tonlib_api.tl_api, line 216
    #[serde(rename = "logVerbosityLevel")]
    TLLogVerbosityLevel(TLLogVerbosityLevel),
    // tonlib_api.tl_api, line 219
    #[serde(rename = "blocks.masterchainInfo")]
    TLBlocksMCInfo(TLBlocksMCInfo),
    // tonlib_api.tl_api, line 220
    #[serde(rename = "blocks.shards")]
    TLBlocksShards(TLBlocksShards),
    // tonlib_api.tl_api, line 223
    #[serde(rename = "blocks.transactions")]
    TLBlocksTxs(TLBlocksTxs),
    // tonlib_api.tl_api, line 224
    #[serde(rename = "blocks.transactionsExt")]
    TLBlocksTransactionsExt(TLBlocksTransactionsExt),
    // tonlib_api.tl_api, line 225
    #[serde(rename = "blocks.header")]
    TLBlocksHeader(TLBlocksHeader),
    // tonlib_api.tl_api, line 243
    #[serde(rename = "configInfo")]
    TLConfigInfo(TLConfigInfo),
}

impl TLResponse {
    /// # Safety
    ///
    /// Safe to call if there is a string underline
    pub unsafe fn from_c_str_json(c_str: *const c_char) -> Result<(TLResponse, Option<String>), TonError> {
        if c_str.is_null() {
            return Err(TonError::TLWrongUsage("null pointer passed to TLResponse".to_string()));
        }
        // No need to free c_str. Tonlib cares about it itself.
        let c_str = unsafe { CStr::from_ptr(c_str) };
        let json_str = c_str.to_str()?.to_string();
        let value: Value = serde_json::from_str(&json_str)?;

        let extra: Option<String> =
            value.as_object().and_then(|m| m.get("@extra")).and_then(|v| v.as_str()).map(|s| s.to_string());

        let response = serde_json::from_value(value)?;
        Ok((response, extra))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::ffi::CString;

    #[test]
    fn test_tl_response_from_c_str_json() -> anyhow::Result<()> {
        let cstr =
            CString::new("{\"@extra\":\"some_extra\",\"@type\":\"logVerbosityLevel\",\"verbosity_level\":100500}")?;
        let (rsp, extra) = unsafe { TLResponse::from_c_str_json(cstr.as_ptr())? };
        std::mem::forget(cstr); // pointer is free during parsing
        let expected = Some(String::from("some_extra"));
        assert_eq!(extra, expected);
        assert!(matches!(rsp, TLResponse::TLLogVerbosityLevel(_)));
        Ok(())
    }
}
