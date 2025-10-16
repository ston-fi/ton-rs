use crate::errors::TonError;
use crate::tep::metadata::MetadataDict;

pub trait Metadata: Sized {
    fn from_data(dict: &MetadataDict, json: Option<&str>) -> Result<Self, TonError>;

    fn from_json(json: &str) -> Result<Self, TonError> { Self::from_data(&MetadataDict::new(), Some(json)) }
    fn from_dict(dict: &MetadataDict) -> Result<Self, TonError> { Self::from_data(dict, None) }
}
