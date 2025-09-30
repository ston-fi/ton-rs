use crate::block_tlb::TVMStack;
use crate::contracts::ton_contract::ContractCtx;
use crate::contracts::ton_contract::TonContract;
use crate::errors::TonResult;
use crate::tep::tvm_results::*;
use async_trait::async_trait;
use num_bigint::BigInt;
use ton_lib_core::cell::TonCellRef;
use ton_lib_core::ton_contract;

#[ton_contract]
pub struct NFTCollectionContract;
impl NFTCollectionMethods for NFTCollectionContract {}

#[async_trait]
pub trait NFTCollectionMethods: TonContract {
    async fn get_collection_data(&self) -> TonResult<GetCollectionDataResult> {
        let stack_boc = self.emulate_get_method("get_collection_data", &TVMStack::EMPTY).await?;
        Ok(GetCollectionDataResult::from_boc(&stack_boc)?)
    }

    async fn get_nft_content(&self, index: BigInt, individual_content: TonCellRef) -> TonResult<GetNFTContentResult> {
        let mut stack = TVMStack::default();
        stack.push_int(index);
        stack.push_cell(individual_content);

        let stack_boc = self.emulate_get_method("get_nft_content", &stack).await?;

        Ok(GetNFTContentResult::from_boc(&stack_boc)?)
    }

    async fn get_nft_address_by_index<T: Into<BigInt> + Send>(
        &self,
        index: T,
    ) -> TonResult<GetNFTAddressByIndexResult> {
        let mut stack = TVMStack::default();
        stack.push_int(index.into());

        let stack_boc = self.emulate_get_method("get_nft_address_by_index", &stack).await?;
        Ok(GetNFTAddressByIndexResult::from_boc(&stack_boc)?)
    }
}
