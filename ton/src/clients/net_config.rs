use crate::block_tlb::BlockIdExt;
use crate::errors::TonError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::{exists, File};
use std::io::Read;

pub const TON_NET_CONF_MAINNET_PUBLIC: &str = include_str!("../../resources/net_config/mainnet_public.json");
pub const TON_NET_CONF_TESTNET_PUBLIC: &str = include_str!("../../resources/net_config/testnet_public.json");

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
    pub fn new(json: &str) -> Result<Self, TonError> { Ok(serde_json::from_str(json)?) }

    pub fn get_json(mainnet: bool) -> String {
        match get_default_net_conf_throw(mainnet) {
            Ok(net_conf) => net_conf,
            Err(err) => {
                log::error!("Failed to load net config_types: {err}. Using default (mainnet: {mainnet})");
                match mainnet {
                    true => TON_NET_CONF_MAINNET_PUBLIC.to_string(),
                    false => TON_NET_CONF_TESTNET_PUBLIC.to_string(),
                }
            }
        }
    }

    pub fn from_env_path(env_var: &str, fallback_json: &str) -> Result<Self, TonError> {
        let path = match std::env::var(env_var) {
            Ok(path) => path,
            Err(_) => return TonNetConfig::new(fallback_json),
        };
        if !std::path::Path::new(&path).exists() {
            log::warn!("TonNetConfig env_var {env_var} is set to {path}, but file does not exist. Using fallback");
            return TonNetConfig::new(fallback_json);
        }
        TonNetConfig::new(&std::fs::read_to_string(path)?)
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> { serde_json::to_string(self) }

    pub fn get_init_block_seqno(&self) -> u64 { self.validator.init_block["seqno"].as_u64().unwrap_or(0) }

    pub fn set_init_block(&mut self, block_id: &BlockIdExt) {
        self.validator.init_block["workchain"] = serde_json::json!(block_id.shard_ident.workchain);
        self.validator.init_block["shard"] = serde_json::json!(block_id.shard_ident.shard as i64);
        self.validator.init_block["seqno"] = serde_json::json!(block_id.seqno);
        self.validator.init_block["root_hash"] = serde_json::json!(block_id.root_hash.to_base64());
        self.validator.init_block["file_hash"] = serde_json::json!(block_id.file_hash.to_base64());
    }
}

fn get_default_net_conf_throw(mainnet: bool) -> Result<String, TonError> {
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
            let mut new_conf = String::new();
            let mut file = File::open(&path)?;
            file.read_to_string(&mut new_conf)?;
            net_conf = new_conf;
            log::info!("Using TON_NET_CONF from {path}")
        } else {
            log::warn!("env_var {env_var_name} is set, but path {path} is not available");
        }
    } else {
        log::info!("env_var {env_var_name} is not set, using default net config");
    }
    Ok(net_conf)
}
