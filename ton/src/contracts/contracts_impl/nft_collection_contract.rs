use crate::contracts::TonContract;
use crate::ton_core::traits::tlb::TLB;
use ton_core::cell::TonCell;
use crate::contracts::ContractClient;
use crate::contracts::contract_methods::NFTCollectionMethods;
use crate::ton_contract;
use crate::ton_core::traits::contract_provider::TonContractState;

ton_contract!(NFTCollectionContract<TonCell>: NFTCollectionMethods);
