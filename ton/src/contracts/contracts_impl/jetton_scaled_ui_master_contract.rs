use crate::contracts::contract_methods::{JettonMasterMethods, ScaledUIMethods};
use crate::contracts::ContractClient;
use crate::contracts::TonContract;
use crate::ton_contract;
use crate::ton_core::traits::contract_provider::TonContractState;

// https://github.com/the-ton-tech/TEPs/blob/scaled-ui/text/0000-scaled-ui-jettons.md
ton_contract!(JettonScaledUIMasterContract: JettonMasterMethods, ScaledUIMethods);
