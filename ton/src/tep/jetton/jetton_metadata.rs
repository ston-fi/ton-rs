use crate::errors::TonError;
use crate::tep::metadata::*;
use crate::tep::snake_data::SnakeData;
use serde::Deserialize;
use serde::Serialize;
use serde_aux::prelude::*;
use std::collections::HashMap;
use std::fmt::Debug;
use ton_lib_core::cell::TonHash;

#[derive(Serialize, PartialEq, Eq, Deserialize, Debug, Clone)]
pub struct JettonMetadata {
    pub name: Option<String>,
    pub uri: Option<String>,
    pub symbol: Option<String>,
    pub description: Option<String>,
    pub image: Option<String>,
    pub image_data: Option<Vec<u8>>,
    ///Optional. If not specified, 9 is used by default. UTF8 encoded string with number from 0 to 255.
    #[serde(default, deserialize_with = "deserialize_option_number_from_string")]
    pub decimals: Option<u8>,
}

impl Metadata for JettonMetadata {
    fn from_data(dict: &HashMap<TonHash, SnakeData>, json: Option<&str>) -> Result<Self, TonError> {
        let mut external_meta: Option<JettonMetadata> =
            json.map(serde_json::from_str).transpose().map_err(|_| TonError::MetadataParseError)?;

        let decimals = match external_meta.as_mut().and_then(|x| x.decimals.take()) {
            Some(dec) => Some(dec),
            None => META_DECIMALS.use_string_or(None, dict).map(|v| v.as_str().parse::<u8>().unwrap()),
        };

        Ok(JettonMetadata {
            name: META_NAME.use_string_or(external_meta.as_mut().and_then(|x| x.name.take()), dict),
            uri: META_URI.use_string_or(external_meta.as_mut().and_then(|x| x.uri.take()), dict),
            symbol: META_SYMBOL.use_string_or(external_meta.as_mut().and_then(|x| x.symbol.take()), dict),
            description: META_DESCRIPTION
                .use_string_or(external_meta.as_mut().and_then(|x| x.description.take()), dict),
            image: META_IMAGE.use_string_or(external_meta.as_mut().and_then(|x| x.image.take()), dict),
            image_data: external_meta
                .as_mut()
                .and_then(|x| x.image_data.take())
                .or(dict.get(&META_IMAGE_DATA).map(|elem| elem.as_slice().to_vec())),
            decimals,
        })
    }
}
