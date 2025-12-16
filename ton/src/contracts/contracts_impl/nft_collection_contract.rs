use crate::contracts::ContractClient;
use crate::contracts::TonContract;
use crate::contracts::contract_methods::NFTCollectionMethods;
use crate::ton_contract;
use crate::ton_core::traits::contract_provider::TonContractState;
use crate::ton_core::traits::tlb::TLB;
use ton_core::cell::TonCell;

ton_contract!(NFTCollectionContract<TonCell>: NFTCollectionMethods);
