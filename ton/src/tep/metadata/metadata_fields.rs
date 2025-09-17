use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::LazyLock;
use ton_lib_core::cell::TonHash;

use crate::tep::snake_data::SnakeData;

pub struct MetadataField(TonHash);
pub static META_NAME: LazyLock<MetadataField> = LazyLock::new(|| MetadataField::new("name"));
pub static META_DESCRIPTION: LazyLock<MetadataField> = LazyLock::new(|| MetadataField::new("description"));
pub static META_IMAGE: LazyLock<MetadataField> = LazyLock::new(|| MetadataField::new("image"));
pub static META_SYMBOL: LazyLock<MetadataField> = LazyLock::new(|| MetadataField::new("symbol"));
pub static META_IMAGE_DATA: LazyLock<MetadataField> = LazyLock::new(|| MetadataField::new("image_data"));
pub static META_DECIMALS: LazyLock<MetadataField> = LazyLock::new(|| MetadataField::new("decimals"));
pub static META_URI: LazyLock<MetadataField> = LazyLock::new(|| MetadataField::new("uri"));
pub static META_CONTENT_URL: LazyLock<MetadataField> = LazyLock::new(|| MetadataField::new("content_url"));
pub static META_ATTRIBUTES: LazyLock<MetadataField> = LazyLock::new(|| MetadataField::new("attributes"));
pub static META_SOCIAL_LINKS: LazyLock<MetadataField> = LazyLock::new(|| MetadataField::new("social_links"));
pub static META_MARKETPLACE: LazyLock<MetadataField> = LazyLock::new(|| MetadataField::new("marketplace"));

impl Deref for MetadataField {
    type Target = TonHash;
    fn deref(&self) -> &Self::Target { &self.0 }
}

impl MetadataField {
    fn new(name: &str) -> MetadataField {
        let mut hasher = Sha256::new();
        hasher.update(name);
        let slice = &hasher.finalize()[..];
        let key = TonHash::from_slice(slice).unwrap_or(TonHash::ZERO);
        MetadataField(key)
    }

    pub fn use_string_or(&self, src: Option<String>, dict: &HashMap<TonHash, SnakeData>) -> Option<String> {
        if src.is_some() {
            return src;
        };

        dict.get(self).map(|x| x.as_str().to_string())
    }

    pub fn use_value_or(&self, src: Option<Value>, dict: &HashMap<TonHash, SnakeData>) -> Option<Value> {
        if src.is_some() {
            return src;
        };
        match dict.get(self) {
            Some(attr_str) => {
                let json_val = Value::String(attr_str.as_str().to_string());
                Some(Value::Array(vec![json_val]))
            }
            None => None,
        }
    }
}
