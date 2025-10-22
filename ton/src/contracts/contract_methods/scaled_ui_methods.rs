use crate::block_tlb::TVMStack;
use crate::contracts::TonContract;
use crate::errors::TonError;
use crate::tep::tvm_results::{GetDisplayMultiplierResult, TVMResult};
use async_trait::async_trait;

#[async_trait]
pub trait ScaledUIMethods: TonContract {
    async fn get_display_multiplier(&self) -> Result<GetDisplayMultiplierResult, TonError> {
        let stack_boc = self.emulate_get_method("get_display_multiplier", &TVMStack::EMPTY).await?;
        Ok(GetDisplayMultiplierResult::from_boc(stack_boc)?)
    }
}
