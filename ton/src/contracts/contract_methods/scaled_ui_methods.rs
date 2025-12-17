use crate::block_tlb::TVMStack;
use crate::contracts::TonContract;
use crate::errors::TonError;
use crate::tep::tvm_results::GetDisplayMultiplierResult;
use async_trait::async_trait;

#[async_trait]
pub trait ScaledUIMethods: TonContract {
    async fn get_display_multiplier(&self) -> Result<GetDisplayMultiplierResult, TonError> {
        self.emulate_get_method("get_display_multiplier", &TVMStack::EMPTY, None).await
    }
}
