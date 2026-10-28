use crate::block_tlb::*;
use ton_core::TLB;
use ton_core::cell::{TonCell, TonHash};
use ton_core::types::tlb_core::TLBCoins;
use ton_core::types::tlb_core::{MsgAddressInt, TLBRef};

#[derive(Default, Debug, Clone, PartialEq, TLB)]
pub struct ShardAccount {
    pub account: TLBRef<MaybeAccount>,
    pub last_tx_hash: TonHash,
    pub last_tx_lt: u64,
}

// https://github.com/ton-blockchain/ton/blob/59a8cf0ae5c3062d14ec4c89a04fee80b5fd05c1/crypto/block/block.tlb#L259
// intentionally implemented as enum - Account can't be used directly
/// Optional account value in a shard state.
#[derive(Debug, Clone, PartialEq, TLB)]
#[non_exhaustive]
pub enum MaybeAccount {
    None(AccountNone),
    Account(Box<Account>),
}

#[derive(Debug, Clone, PartialEq, TLB)]
#[tlb(prefix = 0b0, bits_len = 1)]
pub struct AccountNone;

#[derive(Debug, Clone, PartialEq, TLB)]
#[tlb(prefix = 0b1, bits_len = 1)]
pub struct Account {
    pub addr: MsgAddressInt,
    pub storage_stat: StorageInfo,
    pub storage: AccountStorage,
}

impl ShardAccount {
    pub const NON_EXIST: ShardAccount = ShardAccount {
        account: TLBRef::new(MaybeAccount::None(AccountNone)),
        last_tx_hash: TonHash::ZERO,
        last_tx_lt: 0,
    };
}

impl Default for MaybeAccount {
    fn default() -> Self {
        MaybeAccount::None(AccountNone)
    }
}

#[rustfmt::skip]
impl MaybeAccount {
    pub fn as_active(&self) -> Option<&AccountStateActive> { self.as_account()?.storage.state.as_active() }
    pub fn as_frozen(&self) -> Option<&AccountStateFrozen> { self.as_account()?.storage.state.as_frozen() }
    pub fn get_code(&self) -> Option<&TonCell> { self.as_active()?.state_init.code.as_deref() }
    pub fn get_data(&self) -> Option<&TonCell> { self.as_active()?.state_init.data.as_deref() }
    pub fn get_balance(&self) -> Option<&TLBCoins> { Some(&self.as_account()?.storage.balance.coins) }

    pub fn as_active_mut(&mut self) -> Option<&mut AccountStateActive> { self.as_account_mut()?.storage.state.as_active_mut() }
    pub fn as_frozen_mut(&mut self) -> Option<&mut AccountStateFrozen> { self.as_account_mut()?.storage.state.as_frozen_mut() }
    pub fn get_code_mut(&mut self) -> Option<&mut TonCell> { self.as_active_mut()?.state_init.code.as_deref_mut() }
    pub fn get_data_mut(&mut self) -> Option<&mut TonCell> { self.as_active_mut()?.state_init.data.as_deref_mut() }
    pub fn get_balance_mut(&mut self) -> Option<&mut TLBCoins> { Some(&mut self.as_account_mut()?.storage.balance.coins) }
}

#[cfg(test)]
#[path = "../../tests/block_tlb/account_types/test_shard_account.rs"]
mod tests;
