use crate::block_tlb::TVMStack;
use crate::contracts::ContractClient;
use crate::contracts::TonContract;
use crate::errors::TonError;
use crate::ton_contract;
use crate::ton_core::traits::contract_provider::TonContractState;
use crate::ton_core::traits::tlb::TLB;
use async_trait::async_trait;
use ton_core::cell::{TonCell, TonHash};

#[async_trait]
pub trait TonWalletMethods: TonContract {
    async fn seqno(&self) -> Result<u32, TonError> { self.emulate_get_method("seqno", &TVMStack::EMPTY, None).await }

    async fn get_public_key(&self) -> Result<TonHash, TonError> {
        self.emulate_get_method("get_public_key", &TVMStack::EMPTY, None).await
    }
}

ton_contract!(TonWalletContract: TonWalletMethods);
