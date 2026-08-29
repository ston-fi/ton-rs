use crate::tl_client::tl::Base64Standard;
use crate::tl_client::tl::ser_de::*;
use crate::ton_core::serde::*;
use std::borrow::Cow;
use std::fmt::Debug;

use crate::block_tlb::BlockIdExt;
use serde::{Deserialize, Serialize};
use serde_aux::prelude::*;
use ton_core::cell::TonHash;
use ton_core::types::{TonAddress, TxLTHash};

/// Tonlib key-store configuration.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(tag = "@type")]
#[non_exhaustive]
pub enum TLKeyStoreType {
    #[serde(rename = "keyStoreTypeDirectory")]
    Directory { directory: String },
    #[serde(rename = "keyStoreTypeInMemory")]
    InMemory,
}

// tonlib_api.tl_api, line 26
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TLConfig {
    #[serde(rename = "config")]
    pub net_config_json: String,
    pub blockchain_name: Option<String>,
    pub use_callbacks_for_network: bool,
    pub ignore_cache: bool,
}

// tonlib_api.tl_api, line 28
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TLOptions {
    pub config: TLConfig,
    pub keystore_type: TLKeyStoreType,
}

/// Tonlib configuration defaults.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(tag = "@type", rename = "options.configInfo")]
#[non_exhaustive]
pub struct TLOptionsConfigInfo {
    pub default_wallet_id: String,
    pub default_rwallet_init_public_key: String,
}

/// Tonlib initialization information.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct TLOptionsInfo {
    pub config_info: TLOptionsConfigInfo,
}

// tonlib_api.tl_api, line 44
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TLAccountAddress {
    #[serde(rename = "account_address", with = "serde_ton_address_hex")]
    pub address: TonAddress,
}

impl From<TonAddress> for TLAccountAddress {
    fn from(address: TonAddress) -> Self {
        TLAccountAddress { address }
    }
}

impl From<TLAccountAddress> for TonAddress {
    fn from(tl_address: TLAccountAddress) -> Self {
        tl_address.address
    }
}

// tonlib_api.tl_api, line 50
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TLBlockId {
    pub workchain: i32,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub shard: i64,
    pub seqno: i32,
}

/// Raw account state returned by tonlib.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct TLRawFullAccountState {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub balance: i64,
    #[serde(with = "Base64Standard")]
    pub code: Vec<u8>,
    #[serde(with = "Base64Standard")]
    pub data: Vec<u8>,
    #[serde(rename = "last_transaction_id")]
    #[serde(with = "serde_tx_lt_hash_json")]
    pub last_tx_id: TxLTHash,
    #[serde(with = "serde_block_id_ext")]
    pub block_id: BlockIdExt,
    #[serde(with = "Base64Standard")]
    pub frozen_hash: Vec<u8>,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub sync_utime: i64,
}

/// Raw message returned by tonlib.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct TLRawMsg {
    pub source: TLAccountAddress,
    pub destination: TLAccountAddress,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub value: i64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub fwd_fee: i64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub ihr_fee: i64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub created_lt: i64,
    #[serde(with = "Base64Standard")]
    pub body_hash: Vec<u8>,
    pub msg_data: TLMsgData,
}

/// Raw transaction returned by tonlib.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct TLRawTx {
    pub address: TLAccountAddress,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub utime: i64,
    #[serde(with = "Base64Standard")]
    pub data: Vec<u8>,
    #[serde(rename = "transaction_id")]
    #[serde(with = "serde_tx_lt_hash_json")]
    pub tx_id: TxLTHash,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub fee: i64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub storage_fee: i64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub other_fee: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_msg: Option<TLRawMsg>,
    pub out_msgs: Vec<TLRawMsg>,
}

/// Page of raw transactions returned by tonlib.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct TLRawTxs {
    #[serde(rename = "transactions")]
    pub txs: Vec<TLRawTx>,
    #[serde(rename = "previous_transaction_id")]
    #[serde(with = "serde_tx_lt_hash_json")]
    pub last_tx_id: TxLTHash,
}
/// Information about a submitted external message.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct TLRawExtMessageInfo {
    #[serde(with = "Base64Standard")]
    pub hash: Vec<u8>,
}

// tonlib_api.tl_api, line 60
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TLPChanConfig {
    pub alice_public_key: String,
    pub alice_address: TLAccountAddress,
    pub bob_public_key: String,
    pub bob_address: TLAccountAddress,
    pub init_timeout: i32,
    pub close_timeout: i32,
    pub channel_id: i64,
}

// tonlib_api.tl_api, line 68
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TLRWalletLimit {
    pub seconds: i32,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub value: i64,
}

// tonlib_api.tl_api, line 69
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TLRWalletConfig {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub start_at: i64,
    pub limits: Vec<TLRWalletLimit>,
}

/// Account state decoded by tonlib.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(tag = "@type")]
#[non_exhaustive]
pub enum TLAccountState {
    #[serde(rename = "raw.accountState")]
    Raw {
        #[serde(with = "Base64Standard")]
        code: Vec<u8>,
        #[serde(with = "Base64Standard")]
        data: Vec<u8>,
        #[serde(with = "Base64Standard")]
        frozen_hash: Vec<u8>,
    },
    #[serde(rename = "ton_wallet.v3.accountState")]
    WalletV3 {
        #[serde(deserialize_with = "deserialize_number_from_string")]
        wallet_id: i64,
        seqno: i32,
    },
    #[serde(rename = "ton_wallet.v4.accountState")]
    WalletV4 {
        #[serde(deserialize_with = "deserialize_number_from_string")]
        wallet_id: i64,
        seqno: i32,
    },
    #[serde(rename = "ton_wallet.highload.v1.accountState")]
    WalletHighloadV1 {
        #[serde(deserialize_with = "deserialize_number_from_string")]
        wallet_id: i64,
        seqno: i32,
    },
    #[serde(rename = "ton_wallet.highload.v2.accountState")]
    WalletHighloadV2 {
        #[serde(deserialize_with = "deserialize_number_from_string")]
        wallet_id: i64,
    },
    #[serde(rename = "dns.accountState")]
    DNS {
        #[serde(deserialize_with = "deserialize_number_from_string")]
        wallet_id: i64,
    },
    #[serde(rename = "rwallet.accountState")]
    RWallet {
        #[serde(deserialize_with = "deserialize_number_from_string")]
        wallet_id: i64,
        seqno: i32,
        #[serde(deserialize_with = "deserialize_number_from_string")]
        unlocked_balance: i64,
        config: TLRWalletConfig,
    },
    #[serde(rename = "uninited.accountState")]
    Uninited {
        #[serde(with = "Base64Standard")]
        frozen_hash: Vec<u8>,
    },
    #[serde(rename = "pchan.accountState")]
    PChan {
        config: TLPChanConfig,
        state: TLPChanState,
        description: String,
    },
}

/// Payment-channel state decoded by tonlib.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(tag = "@type")]
#[non_exhaustive]
pub enum TLPChanState {
    #[serde(rename = "pchan.stateInit")]
    Init {
        #[serde(rename = "signed_A")]
        signed_a: bool,
        #[serde(rename = "signed_B")]
        signed_b: bool,
        #[serde(rename = "min_A")]
        min_a: i64,
        #[serde(rename = "min_B")]
        min_b: i64,
        expire_at: i64,
        #[serde(rename = "A")]
        a: i64,
        #[serde(rename = "B")]
        b: i64,
    },
    #[serde(rename = "pchan.stateClose")]
    Close {
        #[serde(rename = "signed_A")]
        signed_a: bool,
        #[serde(rename = "signed_B")]
        signed_b: bool,
        #[serde(rename = "min_A")]
        min_a: i64,
        #[serde(rename = "min_B")]
        min_b: i64,
        expire_at: i64,
        #[serde(rename = "A")]
        a: i64,
        #[serde(rename = "B")]
        b: i64,
    },
    #[serde(rename = "pchan.statePayout")]
    Payout {
        #[serde(rename = "A")]
        a: i64,
        #[serde(rename = "B")]
        b: i64,
    },
}

/// Full account state returned by tonlib.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct TLFullAccountState {
    pub address: TLAccountAddress,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub balance: i64,
    #[serde(rename = "last_transaction_id")]
    #[serde(with = "serde_tx_lt_hash_json")]
    pub last_tx_id: TxLTHash,
    #[serde(with = "serde_block_id_ext")]
    pub block_id: BlockIdExt,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub sync_utime: i64,
    pub account_state: TLAccountState,
    pub revision: i32,
}

/// Tonlib synchronization state.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(tag = "@type")]
#[non_exhaustive]
pub enum TLSyncState {
    #[serde(rename = "syncStateDone")]
    Done,
    #[serde(rename = "syncStateInProgress")]
    InProgress {
        from_seqno: i32,
        to_seqno: i32,
        current_seqno: i32,
    },
}

/// Message data decoded by tonlib.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(tag = "@type")]
#[non_exhaustive]
pub enum TLMsgData {
    #[serde(rename = "msg.dataRaw")]
    Raw {
        #[serde(with = "Base64Standard")]
        body: Vec<u8>,
        #[serde(with = "Base64Standard")]
        init_state: Vec<u8>,
    },
    #[serde(rename = "msg.dataText")]
    Text {
        #[serde(with = "Base64Standard")]
        text: Vec<u8>,
    },
    #[serde(rename = "msg.dataDecryptedText")]
    DecryptedText {
        #[serde(with = "Base64Standard")]
        text: Vec<u8>,
    },
    #[serde(rename = "msg.dataEncryptedText")]
    EncryptedText {
        #[serde(with = "Base64Standard")]
        text: Vec<u8>,
    },
}

/// Loaded smart-contract handle.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct TLSmcInfo {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub id: i64,
}

/// Tonlib get-method identifier.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(tag = "@type")]
#[non_exhaustive]
pub enum TLSmcMethodId {
    #[serde(rename = "smc.methodIdNumber")]
    Number { number: i32 },
    #[serde(rename = "smc.methodIdName")]
    Name { name: Cow<'static, str> },
}

// tonlib_api.tl_api, line 184 - unsupported
// #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
// pub struct TLSmcRunResult {}

/// Smart-contract library returned by tonlib.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct TLSmcLibraryEntry {
    #[serde(with = "Base64Standard")]
    pub hash: Vec<u8>,
    #[serde(with = "Base64Standard")]
    pub data: Vec<u8>,
}

/// Smart-contract library lookup result.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct TLSmcLibraryResult {
    pub result: Vec<TLSmcLibraryEntry>,
}
/// Extended smart-contract library query.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(tag = "@type", rename_all = "camelCase")]
#[non_exhaustive]
pub enum TLSmcLibraryQueryExt {
    #[serde(rename = "smc.libraryQueryExt.one")]
    One {
        #[serde(with = "serde_ton_hash_base64")]
        hash: TonHash,
    },

    // tonlib_api.tl_api, line 190
    #[serde(rename = "smc.libraryQueryExt.scanBoc")]
    ScanBoc {
        #[serde(with = "Base64Standard")]
        boc: Vec<u8>,
        max_libs: i32,
    },
}
/// Extended smart-contract library result.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct TLSmcLibraryResultExt {
    #[serde(with = "Base64Standard")]
    pub dict_boc: Vec<u8>,
    #[serde(with = "serde_ton_hash_vec_base64")]
    pub libs_ok: Vec<TonHash>,
    #[serde(with = "serde_ton_hash_vec_base64")]
    pub libs_not_found: Vec<TonHash>,
}

/// Tonlib synchronization update.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct TLUpdateSyncState {
    pub sync_state: TLSyncState,
}

// tonlib_api.tl_api, line 209
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TLLogVerbosityLevel {
    pub verbosity_level: u32,
}

/// Connected lite-server information.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct TLLiteServerInfo {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub now: i64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub version: i32,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub capabilities: i64,
}

/// Masterchain information returned by tonlib.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct TLBlocksMCInfo {
    #[serde(with = "serde_block_id_ext")]
    pub last: BlockIdExt,
    #[serde(with = "Base64Standard")]
    pub state_root_hash: Vec<u8>,
    #[serde(with = "serde_block_id_ext")]
    pub init: BlockIdExt,
}

/// Shard block IDs returned by tonlib.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct TLBlocksShards {
    #[serde(with = "serde_block_id_ext_vec")]
    pub shards: Vec<BlockIdExt>,
}

// tonlib_api.tl_api, line 221
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TLAccountTxId {
    #[serde(with = "serde_ton_hash_base64")]
    #[serde(rename = "account")]
    pub address_hash: TonHash,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub lt: i64,
}

/// Compact transaction identifier returned by tonlib.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct TLShortTxId {
    pub mode: u32,
    #[serde(with = "serde_ton_hash_base64")]
    #[serde(rename = "account")]
    pub address_hash: TonHash,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub lt: i64,
    #[serde(with = "serde_ton_hash_base64")]
    #[serde(rename = "hash")]
    pub tx_hash: TonHash,
}

/// Block transaction page returned by tonlib.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct TLBlocksTxs {
    #[serde(with = "serde_block_id_ext")]
    pub id: BlockIdExt,
    pub req_count: i32,
    pub incomplete: bool,
    #[serde(rename = "transactions")]
    pub txs: Vec<TLShortTxId>,
}

/// Extended block transaction page returned by tonlib.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct TLBlocksTransactionsExt {
    #[serde(with = "serde_block_id_ext")]
    pub id: BlockIdExt,
    pub req_count: i32,
    pub incomplete: bool,
    #[serde(rename = "transactions")]
    pub txs: Vec<TLRawTx>,
}

/// Blockchain configuration returned by tonlib.
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
#[non_exhaustive]
pub struct TLConfigInfo {
    pub config: TLTvmCell,
}

/// TVM cell returned by tonlib.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Debug)]
#[non_exhaustive]
pub struct TLTvmCell {
    #[serde(with = "Base64Standard")]
    pub bytes: Vec<u8>,
}

/// Block header returned by tonlib.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct TLBlocksHeader {
    #[serde(with = "serde_block_id_ext")]
    pub id: BlockIdExt,
    pub global_id: i32,
    pub version: i32,
    pub flags: i32,
    pub after_merge: bool,
    pub after_split: bool,
    pub before_split: bool,
    pub want_merge: bool,
    pub want_split: bool,
    pub validator_list_hash_short: i32,
    pub catchain_seqno: i32,
    pub min_ref_mc_seqno: i32,
    pub is_key_block: bool,
    pub prev_key_block_seqno: i32,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub start_lt: i64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub end_lt: i64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub gen_utime: i64,
    pub vert_seqno: Option<i32>,
    #[serde(with = "serde_block_id_ext_vec_opt")]
    pub prev_blocks: Option<Vec<BlockIdExt>>,
}

/// Asynchronous tonlib update.
#[derive(Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub enum TLUpdate {
    SyncState(TLUpdateSyncState),
}
