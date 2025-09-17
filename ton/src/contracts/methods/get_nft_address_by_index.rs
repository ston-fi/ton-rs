use crate::block_tlb::TVMStack;
use crate::contracts::ton_contract::TonContract;
use crate::errors::TonError;
use crate::tep::tvm_results::GetNFTAddressByIndexResult;
use crate::tep::tvm_results::TVMResult;
use async_trait::async_trait;
use num_bigint::BigInt;

#[async_trait]
pub trait GetNFTAddressByIndex: TonContract {
    async fn get_nft_address_by_index<T: Into<BigInt> + Send>(
        &self,
        index: T,
    ) -> Result<GetNFTAddressByIndexResult, TonError> {
        let mut stack = TVMStack::default();
        stack.push_int(index.into());

        let stack_boc = self.emulate_get_method("get_nft_address_by_index", &stack).await?;
        Ok(GetNFTAddressByIndexResult::from_boc(&stack_boc)?)
    }
}
