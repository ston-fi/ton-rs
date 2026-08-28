//! Conversion from TEP metadata dictionaries and JSON.

use crate::contracts::tep::metadata::metadata_content::MetadataDict;
use crate::errors::TonError;

pub trait Metadata: Sized {
    fn from_data(dict: &MetadataDict, json: Option<&str>) -> Result<Self, TonError>;

    fn from_json(json: &str) -> Result<Self, TonError> { Self::from_data(&MetadataDict::new(), Some(json)) }
    fn from_dict(dict: &MetadataDict) -> Result<Self, TonError> { Self::from_data(dict, None) }
}
