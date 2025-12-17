use crate::errors::TonError;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::ffi::CString;
use std::ops::Deref;
use std::sync::Arc;

// Custom serialization for EmulBCConfig
pub mod serde_emul_bc_config {
    use super::*;
    use serde::de::Error;

    pub fn serialize<S: Serializer>(config: &EmulBCConfig, serializer: S) -> Result<S::Ok, S::Error> {
        let base64_str = config.to_string_lossy();
        serializer.serialize_str(&base64_str)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<EmulBCConfig, D::Error> {
        let s = String::deserialize(deserializer)?;
        EmulBCConfig::from_boc_base64(&s).map_err(Error::custom)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct EmulBCConfig(Arc<CString>);

impl Deref for EmulBCConfig {
    type Target = CString;

    fn deref(&self) -> &Self::Target { &self.0 }
}

impl From<Arc<CString>> for EmulBCConfig {
    fn from(config: Arc<CString>) -> Self { Self(config) }
}

impl Serialize for EmulBCConfig {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serde_emul_bc_config::serialize(self, serializer)
    }
}

impl<'de> Deserialize<'de> for EmulBCConfig {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        serde_emul_bc_config::deserialize(deserializer)
    }
}

impl EmulBCConfig {
    pub fn from_boc(config_boc: &[u8]) -> Result<Self, TonError> { Self::from_boc_base64(&STANDARD.encode(config_boc)) }
    pub fn from_boc_hex(config_boc_hex: &str) -> Result<Self, TonError> {
        Self::from_boc_base64(&STANDARD.encode(hex::decode(config_boc_hex)?))
    }
    pub fn from_boc_base64(config_boc_base64: &str) -> Result<Self, TonError> {
        Ok(Self(Arc::new(CString::new(config_boc_base64)?)))
    }
    pub fn to_boc(&self) -> Result<Vec<u8>, TonError> {
        let base64_str = self.to_string_lossy();
        Ok(STANDARD.decode(base64_str.as_ref())?)
    }
    pub fn to_boc_hex(&self) -> Result<String, TonError> {
        let boc_bytes = self.to_boc()?;
        Ok(hex::encode(boc_bytes))
    }
}
