use crate::errors::TonError;
use crate::tep::metadata::Metadata;
use crate::tep::metadata::*;
use crate::tep::snake_data::SnakeData;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Debug;
use ton_lib_core::cell::TonHash;

#[derive(Serialize, PartialEq, Eq, Deserialize, Debug, Clone)]
pub struct NFTItemMetadata {
    pub name: Option<String>,
    pub description: Option<String>,
    pub image: Option<String>,
    pub content_url: Option<String>,
    pub attributes: Option<Value>,
}

impl Metadata for NFTItemMetadata {
    fn from_data(dict: &HashMap<TonHash, SnakeData>, json: Option<&str>) -> Result<Self, TonError> {
        let mut external_meta: Option<Self> =
            json.map(serde_json::from_str).transpose().map_err(|_| TonError::MetadataParseError)?;
        Ok(NFTItemMetadata {
            name: META_NAME.use_string_or(external_meta.as_mut().and_then(|x| x.name.take()), dict),
            description: META_DESCRIPTION
                .use_string_or(external_meta.as_mut().and_then(|x| x.description.take()), dict),
            content_url: META_URI.use_string_or(external_meta.as_mut().and_then(|x| x.content_url.take()), dict),
            image: META_IMAGE.use_string_or(external_meta.as_mut().and_then(|x| x.image.take()), dict),
            attributes: META_ATTRIBUTES.use_value_or(external_meta.as_mut().and_then(|x| x.attributes.take()), dict),
        })
    }
}
