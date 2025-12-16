use crate::contracts::TonContract;
use crate::ton_core::traits::tlb::TLB;
use ton_core::cell::TonCell;
use crate::contracts::ContractClient;
use crate::contracts::contract_methods::JettonWalletMethods;
use crate::ton_contract;
use crate::ton_core::traits::contract_provider::TonContractState;

// https://github.com/ton-blockchain/TEPs/blob/master/text/0074-jettons-standard.md#jetton-wallet-smart-contract
ton_contract!(JettonWalletContract<TonCell>: JettonWalletMethods);
