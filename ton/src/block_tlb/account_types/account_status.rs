use ton_lib_core::TLB;

// https://github.com/ton-blockchain/ton/blob/ed4682066978f69ffa38dd98912ca77d4f660f66/crypto/block/block.tlb#L271
#[derive(Debug, Clone, PartialEq, TLB)]
pub enum AccountStatus {
    Uninit(AccountStatusUninit),
    Frozen(AccountStatusFrozen),
    Active(AccountStatusActive),
    NonExist(AccountStatusNotExist),
}

#[derive(Debug, Clone, PartialEq, TLB)]
#[tlb(prefix = 0b00, bits_len = 2)]
pub struct AccountStatusUninit;

#[derive(Debug, Clone, PartialEq, TLB)]
#[tlb(prefix = 0b01, bits_len = 2)]
pub struct AccountStatusFrozen;

#[derive(Debug, Clone, PartialEq, TLB)]
#[tlb(prefix = 0b10, bits_len = 2)]
pub struct AccountStatusActive;

#[derive(Debug, Clone, PartialEq, TLB)]
#[tlb(prefix = 0b11, bits_len = 2)]
pub struct AccountStatusNotExist;

impl Default for AccountStatus {
    fn default() -> Self { AccountStatus::NonExist(AccountStatusNotExist) }
}
