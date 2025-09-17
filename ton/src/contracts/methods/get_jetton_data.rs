use crate::block_tlb::TVMStack;
use crate::contracts::ton_contract::TonContract;
use crate::errors::TonError;
use crate::tep::tvm_results::GetJettonDataResult;
use crate::tep::tvm_results::TVMResult;
use async_trait::async_trait;

#[async_trait]
pub trait GetJettonData: TonContract {
    async fn get_jetton_data(&self) -> Result<GetJettonDataResult, TonError> {
        let stack_boc = self.emulate_get_method("get_jetton_data", &TVMStack::EMPTY).await?;
        Ok(GetJettonDataResult::from_boc(&stack_boc)?)
    }
}
