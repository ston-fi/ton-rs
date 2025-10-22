use crate::contracts::contract_methods::JettonMasterMethods;
use crate::contracts::ContractClient;
use crate::contracts::TonContract;
use crate::ton_contract;
use crate::ton_core::traits::contract_provider::TonContractState;

// https://github.com/ton-blockchain/TEPs/blob/master/text/0074-jettons-standard.md#jetton-master-contract
ton_contract!(JettonMasterContract: JettonMasterMethods);
