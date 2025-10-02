use crate::block_tlb::StateInit;
use ton_lib_core::cell::TonHash;
use ton_lib_core::TLB;

#[derive(Debug, Clone, PartialEq, TLB)]
pub enum AccountState {
    Uninit(AccountStateUninit),
    Frozen(AccountStateFrozen),
    Active(AccountStateActive),
}

#[derive(Debug, Clone, PartialEq, TLB)]
#[tlb(prefix = 0b00, bits_len = 2)]
pub struct AccountStateUninit;

#[derive(Debug, Clone, PartialEq, TLB)]
#[tlb(prefix = 0b01, bits_len = 2)]
pub struct AccountStateFrozen {
    pub state_hash: TonHash,
}

#[derive(Debug, Clone, PartialEq, TLB)]
#[tlb(prefix = 0b1, bits_len = 1)]
pub struct AccountStateActive {
    pub state_init: StateInit,
}
