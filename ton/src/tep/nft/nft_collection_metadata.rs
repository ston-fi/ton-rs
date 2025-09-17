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
pub struct NFTCollectionMetadata {
    pub image: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub social_links: Option<Value>,
    pub marketplace: Option<String>,
}

impl Metadata for NFTCollectionMetadata {
    fn from_data(dict: &HashMap<TonHash, SnakeData>, json: Option<&str>) -> Result<Self, TonError> {
        let mut external_meta: Option<NFTCollectionMetadata> =
            json.map(serde_json::from_str).transpose().map_err(|_| TonError::MetadataParseError)?;

        Ok(NFTCollectionMetadata {
            image: META_IMAGE.use_string_or(external_meta.as_mut().and_then(|x| x.image.take()), dict),
            name: META_NAME.use_string_or(external_meta.as_mut().and_then(|x| x.name.take()), dict),
            description: META_DESCRIPTION
                .use_string_or(external_meta.as_mut().and_then(|x| x.description.take()), dict),
            social_links: META_SOCIAL_LINKS
                .use_value_or(external_meta.as_mut().and_then(|x| x.social_links.take()), dict),
            marketplace: META_MARKETPLACE
                .use_string_or(external_meta.as_mut().and_then(|x| x.marketplace.take()), dict),
        })
    }
}
