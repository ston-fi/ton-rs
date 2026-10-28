use crate::tlb_adapters::DictKeyAdapterTonHash;
use crate::tlb_adapters::DictValAdapterTLB;
use crate::tlb_adapters::TLBHashMapE;
use crate::ton_core::types::tlb_core::adapters::ConstLen;
use std::collections::HashMap;
use ton_core::TLB;
use ton_core::cell::{TonCell, TonHash};
use ton_core::errors::TonCoreError;
use ton_core::traits::tlb::TLB;
use ton_core::types::TonAddress;
use ton_core::types::tlb_core::TLBRef;

// https://github.com/ton-blockchain/ton/blob/59a8cf0ae5c3062d14ec4c89a04fee80b5fd05c1/crypto/block/block.tlb#L281
#[derive(Debug, Clone, PartialEq, TLB)]
pub struct StateInit {
    #[tlb(bits_len = 5)]
    pub split_depth: Option<u8>,
    pub tick_tock: Option<TickTock>,
    pub code: Option<TLBRef<TonCell>>,
    pub data: Option<TLBRef<TonCell>>,
    #[tlb(adapter = "TLBHashMapE::<DictKeyAdapterTonHash, DictValAdapterTLB<_>>::new(256)")]
    pub library: HashMap<TonHash, SimpleLib>,
}

#[derive(Debug, Clone, PartialEq, TLB)]
pub struct SimpleLib {
    pub public: bool,
    pub root: TLBRef<TonCell>,
}

#[derive(Debug, Clone, PartialEq, TLB)]
pub struct TickTock {
    pub tick: bool,
    pub tock: bool,
}

impl StateInit {
    /// Creates state with code and data, without split depth, tick-tock handlers, or libraries.
    pub fn new<C, D>(code: C, data: D) -> Self
    where
        C: Into<TLBRef<TonCell>>,
        D: Into<TLBRef<TonCell>>,
    {
        StateInit {
            split_depth: None,
            tick_tock: None,
            code: Some(code.into()),
            data: Some(data.into()),
            library: Default::default(),
        }
    }

    pub fn derive_address(&self, workchain: i32) -> Result<TonAddress, TonCoreError> {
        Ok(TonAddress::new(workchain, self.cell_hash()?))
    }
}

#[cfg(test)]
#[path = "../tests/block_tlb/test_state_init.rs"]
mod tests;
