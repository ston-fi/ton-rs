use crate::block_tlb::TVMStack;
use crate::contracts::ton_contract::{ContractCtx, TonContract};
use crate::errors::TonError;
use crate::tep::tvm_results::{GetWalletDataResult, TVMResult};
use async_trait::async_trait;
use ton_lib_core::ton_contract;

#[ton_contract]
pub struct JettonWalletContract;
impl JettonWalletMethods for JettonWalletContract {}

#[async_trait]
pub trait JettonWalletMethods: TonContract {
    async fn get_wallet_data(&self) -> Result<GetWalletDataResult, TonError> {
        let stack_boc = self.emulate_get_method("get_wallet_data", &TVMStack::EMPTY).await?;
        Ok(GetWalletDataResult::from_boc(&stack_boc)?)
    }
}
