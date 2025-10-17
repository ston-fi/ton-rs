use crate::block_tlb::TVMStack;
use crate::contracts::ton_contract::TonContract;
use crate::contracts::ContractClient;
use crate::errors::TonError;
use crate::tep::tvm_results::{GetWalletDataResult, TVMResult};
use crate::ton_contract;
use crate::ton_lib_core::traits::contract_provider::TonContractState;
use async_trait::async_trait;

ton_contract!(JettonWalletContract: JettonWalletMethods);

#[async_trait]
pub trait JettonWalletMethods: TonContract {
    async fn get_wallet_data(&self) -> Result<GetWalletDataResult, TonError> {
        let stack_boc = self.emulate_get_method("get_wallet_data", &TVMStack::EMPTY).await?;
        Ok(GetWalletDataResult::from_boc(stack_boc)?)
    }
}
