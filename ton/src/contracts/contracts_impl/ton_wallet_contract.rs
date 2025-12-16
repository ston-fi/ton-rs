use crate::contracts::TonContract;
use crate::contracts::ContractClient;
use crate::ton_core::traits::tlb::TLB;
use crate::ton_core::traits::contract_provider::TonContractState;
use ton_core::cell::TonCell;
use crate::contracts::TonWalletMethods;
use crate::ton_contract;

ton_contract!(TonWalletContract<TonCell>: TonWalletMethods);