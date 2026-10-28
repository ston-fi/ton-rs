use crate::tlb_adapters::DictValAdapterTLB;
use crate::tlb_adapters::{DictKeyAdapterTonHash, TLBHashMap};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use ton_core::TLB;
use ton_core::cell::{TonCell, TonHash};
use ton_core::errors::TonCoreError;
use ton_core::types::tlb_core::TLBRef;

/// Contains dict (TLBHashMap), no 'present' marker in root cell
#[derive(Debug, Clone, PartialEq, Default, TLB)]
pub struct LibsDict {
    #[tlb(adapter = "TLBHashMap::<DictKeyAdapterTonHash, DictValAdapterTLB<_>>::new(256)")]
    data: HashMap<TonHash, TLBRef<TonCell>>,
}

impl LibsDict {
    /// Creates a dictionary keyed by each supplied library cell's hash.
    pub fn new<I: IntoIterator<Item = TonCell>>(libs: I) -> Result<Self, TonCoreError> {
        let mut data = HashMap::new();
        for lib in libs {
            data.insert(*lib.hash()?, lib.into());
        }
        Ok(Self { data })
    }
}

impl From<HashMap<TonHash, TonCell>> for LibsDict {
    fn from(data: HashMap<TonHash, TonCell>) -> Self {
        let inner_data = data.into_iter().map(|(k, v)| (k, TLBRef::from(v))).collect::<HashMap<_, _>>();
        Self { data: inner_data }
    }
}

impl Deref for LibsDict {
    type Target = HashMap<TonHash, TLBRef<TonCell>>;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for LibsDict {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

#[cfg(test)]
#[path = "tests/test_libs_dict.rs"]
mod tests;
