use crate::contracts::TonContract;
use crate::errors::TonResult;
use crate::ton_contract;
use async_trait::async_trait;
use ton_core::cell::TonHash;
use ton_macros::ton_methods;

#[async_trait]
#[ton_methods]
pub trait TonWalletMethods: TonContract {
    async fn seqno(&self) -> TonResult<u32>;
    async fn get_public_key(&self) -> TonResult<TonHash>;
}

ton_contract!(TonWalletContract: TonWalletMethods);
