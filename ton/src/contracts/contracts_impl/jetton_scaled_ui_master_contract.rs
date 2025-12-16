use crate::contracts::ContractClient;
use crate::contracts::TonContract;
use crate::contracts::{JettonMasterMethods, ScaledUIMethods};
use crate::ton_contract;
use crate::ton_core::traits::contract_provider::TonContractState;
use crate::ton_core::traits::tlb::TLB;
use ton_core::cell::TonCell;

// https://github.com/the-ton-tech/TEPs/blob/scaled-ui/text/0000-scaled-ui-jettons.md
ton_contract!(JettonScaledUIMasterContract<TonCell>: JettonMasterMethods, ScaledUIMethods);
