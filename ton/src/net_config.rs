use crate::bail_ton;
use crate::block_tlb::BlockIdExt;
use crate::errors::{TonError, TonResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::exists;

pub const TON_NET_CONF_MAINNET_PUBLIC: &str = include_str!("../resources/net_config/mainnet_public.json");
pub const TON_NET_CONF_TESTNET_PUBLIC: &str = include_str!("../resources/net_config/testnet_public.json");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TonNetConfig {
    #[serde(rename = "@type")]
    pub conf_type: Value,
    pub dht: Value,
    #[serde(rename = "liteservers")]
    pub lite_endpoints: Vec<LiteEndpoint>,
    pub validator: Validator,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiteEndpoint {
    pub ip: i32,
    pub port: u16,
    pub id: LiteID,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiteID {
    #[serde(rename = "@type")]
    pub config_type: Value,
    pub key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validator {
    #[serde(rename = "@type")]
    pub config_type: Value,
    pub zero_state: Value,
    pub init_block: Value,
    pub hardforks: Value,
}

impl TonNetConfig {
    pub fn new(json: &str) -> TonResult<Self> { Ok(serde_json::from_str(json)?) }

    pub fn from_path(path: &str) -> TonResult<Self> { TonNetConfig::new(&std::fs::read_to_string(path)?) }

    pub fn from_env_path(env_var: &str) -> TonResult<Self> {
        if let Ok(path) = std::env::var(env_var) {
            return TonNetConfig::from_path(&path);
        }
        bail_ton!("environment variable \"{env_var}\" is not set")
    }

    /// Takes `TON_NET_CONF_MAINNET_PATH` or `TON_NET_CONF_TESTNET_PATH` from env if set,
    /// Otherwise uses built-in default config for mainnet or testnet
    pub fn new_default(mainnet: bool) -> TonResult<Self> { Self::new(&get_default_net_conf(mainnet)?) }

    pub fn to_json(&self) -> TonResult<String> { Ok(serde_json::to_string(self)?) }

    pub fn get_init_block_seqno(&self) -> u64 { self.validator.init_block["seqno"].as_u64().unwrap_or(0) }

    pub fn set_init_block(&mut self, block_id: &BlockIdExt) {
        self.validator.init_block["workchain"] = serde_json::json!(block_id.shard_ident.workchain);
        self.validator.init_block["shard"] = serde_json::json!(block_id.shard_ident.shard as i64);
        self.validator.init_block["seqno"] = serde_json::json!(block_id.seqno);
        self.validator.init_block["root_hash"] = serde_json::json!(block_id.root_hash.to_base64());
        self.validator.init_block["file_hash"] = serde_json::json!(block_id.file_hash.to_base64());
    }
}

fn get_default_net_conf(mainnet: bool) -> TonResult<String> {
    let env_var_name = match mainnet {
        true => "TON_NET_CONF_MAINNET_PATH",
        false => "TON_NET_CONF_TESTNET_PATH",
    };
    let mut net_conf = match mainnet {
        true => TON_NET_CONF_MAINNET_PUBLIC.to_string(),
        false => TON_NET_CONF_TESTNET_PUBLIC.to_string(),
    };

    if let Ok(path) = std::env::var(env_var_name) {
        if exists(&path)? {
            net_conf = std::fs::read_to_string(&path)?;
            log::info!("Using TON_NET_CONF from {path}")
        } else {
            log::warn!("env_var {env_var_name} is set, but path {path} is not available");
        }
    } else {
        log::info!("env_var {env_var_name} is not set, using default net config");
    }
    Ok(net_conf)
}
