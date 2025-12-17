use crate::block_tlb::TVMStack;
use crate::contracts::TonContract;
use crate::errors::TonError;
use crate::ton_contract;
use async_trait::async_trait;
use ton_core::cell::TonHash;

#[async_trait]
pub trait TonWalletMethods: TonContract {
    async fn seqno(&self) -> Result<u32, TonError> { self.emulate_get_method("seqno", &TVMStack::EMPTY, None).await }

    async fn get_public_key(&self) -> Result<TonHash, TonError> {
        self.emulate_get_method("get_public_key", &TVMStack::EMPTY, None).await
    }
}

ton_contract!(TonWalletContract: TonWalletMethods);
