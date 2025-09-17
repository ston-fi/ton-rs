use crate::block_tlb::TVMStack;
use crate::contracts::ton_contract::TonContract;
use crate::errors::TonError;
use crate::tep::tvm_results::GetNFTContentResult;
use crate::tep::tvm_results::TVMResult;
use async_trait::async_trait;
use num_bigint::BigInt;
use ton_lib_core::cell::TonCellRef;

#[async_trait]
pub trait GetNFTContent: TonContract {
    async fn get_nft_content(
        &self,
        index: BigInt,
        individual_content: TonCellRef,
    ) -> Result<GetNFTContentResult, TonError> {
        let mut stack = TVMStack::default();
        stack.push_int(index);
        stack.push_cell(individual_content);

        let stack_boc = self.emulate_get_method("get_nft_content", &stack).await?;

        Ok(GetNFTContentResult::from_boc(&stack_boc)?)
    }
}
