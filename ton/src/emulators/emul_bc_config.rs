use crate::errors::TonError;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use std::ffi::CString;
use std::ops::Deref;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct EmulBCConfig(Arc<CString>);

impl Deref for EmulBCConfig {
    type Target = CString;

    fn deref(&self) -> &Self::Target { &self.0 }
}

impl From<Arc<CString>> for EmulBCConfig {
    fn from(config: Arc<CString>) -> Self { Self(config) }
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
