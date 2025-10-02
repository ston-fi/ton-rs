use crate::block_tlb::*;
use ton_lib_core::cell::TonHash;
use ton_lib_core::types::tlb_core::VarLenBytes;
use ton_lib_core::TLB;

#[derive(Debug, Clone, PartialEq, TLB)]
pub struct StorageUsed {
    pub cells: VarLenBytes<u64, 3>,
    pub bits: VarLenBytes<u64, 3>,
}

#[derive(Debug, Clone, PartialEq, TLB)]
pub struct StorageInfo {
    pub used: StorageUsed,
    pub storage_extra: MaybeStorageExtraInfo,
    pub last_paid: u32,
    pub due_payment: Option<Coins>,
}

#[derive(Debug, Clone, PartialEq, TLB)]
pub struct AccountStorage {
    pub last_tx_lt: u64,
    pub balance: CurrencyCollection,
    pub state: AccountState,
}

#[derive(Debug, Clone, PartialEq, TLB)]
pub enum MaybeStorageExtraInfo {
    None(StorageExtraInfoNone),
    Info(StorageExtraInfo),
}

#[derive(Debug, Clone, PartialEq, TLB)]
#[tlb(prefix = 0b000, bits_len = 3)]
pub struct StorageExtraInfoNone;

#[derive(Debug, Clone, PartialEq, TLB)]
#[tlb(prefix = 0b001, bits_len = 3)]
pub struct StorageExtraInfo {
    pub dict_hash: TonHash,
}
