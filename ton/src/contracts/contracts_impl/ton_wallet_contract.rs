use crate::contracts::TonContract;
use crate::errors::{TonResult};
use crate::ton_contract;
use async_trait::async_trait;
use ton_core::cell::TonHash;
use ton_macros::ton_method;

#[async_trait]
pub trait TonWalletMethods: TonContract {
    #[ton_method]
    async fn seqno(&self) -> TonResult<u32>;

    #[ton_method]
    async fn get_public_key(&self) -> TonResult<TonHash>;
}

ton_contract!(TonWalletContract: TonWalletMethods);
