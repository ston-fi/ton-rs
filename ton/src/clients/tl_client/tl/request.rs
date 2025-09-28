use crate::clients::tl_client::tl::ser_de::serde_block_id_ext;
use crate::clients::tl_client::tl::types::{
    TLAccountAddress, TLAccountTxId, TLBlockId, TLOptions, TLSmcLibraryQueryExt,
};
use crate::clients::tl_client::tl::Base64Standard;
use crate::ton_lib_core::cell::serde_ton_hash_vec_base64;
use crate::ton_lib_core::types::serde_tx_lt_hash_json;

use crate::block_tlb::BlockIdExt;
use crate::errors::TonError;
use serde::{Deserialize, Serialize};
use std::ffi::CString;
use strum::IntoStaticStr;
use ton_lib_core::cell::TonHash;
use ton_lib_core::types::TxLTHash;

#[derive(IntoStaticStr, Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(tag = "@type", rename_all = "camelCase")]
pub enum TLRequest {
    // tonlib_api.tl, line 216
    LiteServerInfo {
        now: i64,
        version: i32,
        capabilities: i64,
    },

    // tonlib_api.tl, line 238
    Init {
        options: TLOptions,
    },

    // tonlib_api.tl, line 261
    #[serde(rename = "raw.sendMessageReturnHash")]
    RawSendMsgReturnHash {
        #[serde(with = "Base64Standard")]
        body: Vec<u8>,
    },

    // tonlib_api.tl, line 265
    #[serde(rename = "sync")]
    Sync {},

    //tonlib_api.tl, line 266
    #[serde(rename = "raw.getAccountState")]
    RawGetAccountState {
        account_address: TLAccountAddress,
    },

    //tonlib_api.tl, line 267
    #[serde(rename = "raw.getAccountStateByTransaction")]
    RawGetAccountStateByTx {
        account_address: TLAccountAddress,
        #[serde(rename = "transaction_id")]
        #[serde(with = "serde_tx_lt_hash_json")]
        tx_id: TxLTHash,
    },

    // tonlib_api.tl, line 268
    #[serde(rename = "raw.getTransactions")]
    RawGetTxs {
        account_address: TLAccountAddress,
        #[serde(rename = "from_transaction_id")]
        #[serde(with = "serde_tx_lt_hash_json")]
        from_tx_id: TxLTHash,
    },

    // tonlib_api.tl, line 269
    #[serde(rename = "raw.getTransactionsV2")]
    RawGetTxsV2 {
        account_address: TLAccountAddress,
        #[serde(rename = "from_transaction_id")]
        #[serde(with = "serde_tx_lt_hash_json")]
        from_tx_id: TxLTHash,
        count: u32,
        try_decode_messages: bool,
    },

    // tonlib_api.tl, line 270
    #[serde(rename = "raw.sendMessage")]
    RawSendMessage {
        #[serde(with = "Base64Standard")]
        body: Vec<u8>,
    },

    // tonlib_api.tl, line 288
    #[serde(rename = "getAccountState")]
    GetAccountState {
        account_address: TLAccountAddress,
    },

    // tonlib_api.tl, line 294
    #[serde(rename = "getConfigParam")]
    GetConfigParam {
        mode: u32,
        param: u32,
    },

    // tonlib_api.tl, line 295
    #[serde(rename = "getConfigAll")]
    GetConfigAll {
        mode: u32,
    },

    // tonlib_api.tl, line 306
    #[serde(rename = "smc.load")]
    SmcLoad {
        account_address: TLAccountAddress,
    },

    // tonlib_api.tl, line 307
    #[serde(rename = "smc.loadByTransaction")]
    SmcLoadByTransaction {
        account_address: TLAccountAddress,
        #[serde(rename = "transaction_id")]
        #[serde(with = "serde_tx_lt_hash_json")]
        tx_id: TxLTHash,
    },

    // tonlib_api.tl, line 308
    #[serde(rename = "smc.forget")]
    SmcForget {
        id: i64,
    },

    // tonlib_api.tl, line 309
    #[serde(rename = "smc.getCode")]
    SmcGetCode {
        id: i64,
    },

    // tonlib_api.tl, line 310
    #[serde(rename = "smc.getData")]
    SmcGetData {
        id: i64,
    },

    // tonlib_api.tl, line 311
    #[serde(rename = "smc.getState")]
    SmcGetState {
        id: i64,
    },

    // tonlib_api.tl, line 312
    #[serde(rename = "smc.runGetMethod")]
    SmcRunGetMethod(()), // Unsupported

    // tonlib_api.tl, line 314
    #[serde(rename = "smc.getLibraries")]
    SmcGetLibraries {
        #[serde(with = "serde_ton_hash_vec_base64")]
        library_list: Vec<TonHash>,
    },

    // tonlib_api.tl, line 315
    #[serde(rename = "smc.getLibrariesExt")]
    SmcGetLibrariesExt {
        list: Vec<TLSmcLibraryQueryExt>,
    },

    // tonlib_api.tl, line 316
    #[serde(rename = "blocks.getMasterchainInfo")]
    BlocksGetMCInfo {},

    // tonlib_api.tl, line 327
    #[serde(rename = "blocks.getShards")]
    BlocksGetShards {
        #[serde(with = "serde_block_id_ext")]
        id: BlockIdExt,
    },

    // tonlib_api.tl, line 328
    #[serde(rename = "blocks.lookupBlock")]
    BlocksLookupBlock {
        mode: i32,
        id: TLBlockId,
        lt: i64,
        utime: i32,
    },

    // tonlib_api.tl, line 329
    #[serde(rename = "blocks.getTransactions")]
    BlocksGetTxs {
        #[serde(with = "serde_block_id_ext")]
        id: BlockIdExt,
        mode: u32,
        count: u32,
        after: TLAccountTxId,
    },

    // tonlib_api.tl, line 330
    #[serde(rename = "blocks.getTransactionsExt")]
    BlocksGetTransactionsExt {
        #[serde(with = "serde_block_id_ext")]
        id: BlockIdExt,
        mode: u32,
        count: u32,
        after: TLAccountTxId,
    },

    // tonlib_api.tl, line 331
    #[serde(rename = "blocks.getBlockHeader")]
    GetBlockHeader {
        #[serde(with = "serde_block_id_ext")]
        id: BlockIdExt,
    },

    // tonlib_ai.tl, line 342
    #[serde(rename = "liteServer.getInfo")]
    LiteServerGetInfo {},

    // tonlib_api.tl, line 352
    SetLogVerbosityLevel {
        new_verbosity_level: u32,
    },
    // tonlib_api.tl, line 355
    GetLogVerbosityLevel {},
}

impl TLRequest {
    pub fn to_c_str_json(&self, extra: &str) -> Result<CString, TonError> {
        let mut value = serde_json::to_value(self)?;
        let obj = value.as_object_mut().unwrap();
        let extra_val = serde_json::Value::from(extra);
        obj.insert(String::from("@extra"), extra_val);
        Ok(CString::new(serde_json::to_string(&value)?)?)
    }
}

#[cfg(test)]
mod tests {
    use crate::clients::tl_client::tl::request::TLRequest;
    use std::ffi::CString;

    #[test]
    fn test_tl_request_to_c_str_json() -> anyhow::Result<()> {
        let req = TLRequest::SetLogVerbosityLevel {
            new_verbosity_level: 100500,
        };
        let cstr: CString = req.to_c_str_json("some_extra")?;
        assert_eq!(
            "{\"@extra\":\"some_extra\",\"@type\":\"setLogVerbosityLevel\",\"new_verbosity_level\":100500}",
            cstr.to_str()?
        );
        Ok(())
    }
}
