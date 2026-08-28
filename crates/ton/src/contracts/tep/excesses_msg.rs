//! Excess-response message defined by TEP-74.

use ton_core::TLB;

#[derive(Clone, Debug, PartialEq, TLB)]
#[tlb(prefix = 0xd53276db, bits_len = 32, ensure_empty = true)]
pub struct ExcessesMsg {
    pub query_id: u64,
}
