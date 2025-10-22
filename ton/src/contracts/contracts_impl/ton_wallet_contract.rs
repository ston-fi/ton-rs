use crate::contracts::contract_methods::TonWalletMethods;
use crate::contracts::ContractClient;
use crate::contracts::TonContract;
use crate::ton_contract;
use crate::ton_core::traits::contract_provider::TonContractState;

ton_contract!(TonWalletContract: TonWalletMethods);
