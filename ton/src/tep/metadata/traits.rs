use std::collections::HashMap;

use crate::errors::TonError;
use crate::tep::snake_data::SnakeData;
use ton_lib_core::cell::TonHash;
pub trait Metadata: Sized {
    fn from_data(dict: &HashMap<TonHash, SnakeData>, json: Option<&str>) -> Result<Self, TonError>;

    fn from_json(json: &str) -> Result<Self, TonError> {
        Self::from_data(&HashMap::<TonHash, SnakeData>::new(), Some(json))
    }
    fn from_dict(dict: &HashMap<TonHash, SnakeData>) -> Result<Self, TonError> { Self::from_data(dict, None) }
}
