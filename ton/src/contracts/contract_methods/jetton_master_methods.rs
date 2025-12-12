use crate::block_tlb::TVMStack;
use crate::contracts::TonContract;
use crate::errors::TonError;
use crate::tep::tvm_results::{GetJettonDataResult, GetWalletAddressResult, TVMResult};
use async_trait::async_trait;
use ton_core::traits::tlb::TLB;
use ton_core::types::TonAddress;

#[async_trait]
pub trait JettonMasterMethods: TonContract {
    async fn get_jetton_data(&self) -> Result<GetJettonDataResult, TonError> {
        self.emulate_get_method::<_, GetJettonDataResult>("get_jetton_data", &TVMStack::EMPTY, None).await
    }

    async fn get_wallet_address(&self, owner: &TonAddress) -> Result<GetWalletAddressResult, TonError> {
        let mut stack = TVMStack::default();
        stack.push_cell_slice(owner.to_cell()?);
        self.emulate_get_method::<_, GetWalletAddressResult>("get_wallet_address", &stack, None).await
    }
}
