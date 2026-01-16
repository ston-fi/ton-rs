use crate::contracts::TonContract;
use crate::errors::TonResult;
use crate::ton_contract;
use async_trait::async_trait;
use ton_core::types::TonAddress;
use ton_macros::ton_methods;

ton_contract!(SBTContract: SBTMethods);

#[async_trait]
#[ton_methods]
pub trait SBTMethods: TonContract {
    async fn get_authority_address(&self) -> TonResult<TonAddress>;
    async fn get_revoked_time(&self) -> TonResult<u64>;
}
