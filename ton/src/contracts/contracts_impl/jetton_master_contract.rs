use crate::block_tlb::TVMStack;
use crate::contracts::TonContract;
use crate::errors::TonError;
use crate::tep::tvm_results::{GetDisplayMultiplierResult, GetJettonDataResult, GetWalletAddressResult};
use crate::ton_contract;
use crate::ton_core::traits::tlb::TLB;
use async_trait::async_trait;
use ton_core::types::TonAddress;

// https://github.com/ton-blockchain/TEPs/blob/master/text/0074-jettons-standard.md#jetton-master-contract
ton_contract!(JettonMasterContract: JettonMasterMethods);

// https://github.com/the-ton-tech/TEPs/blob/scaled-ui/text/0000-scaled-ui-jettons.md
ton_contract!(JettonScaledUIMasterContract: JettonMasterMethods, ScaledUIMethods);

#[async_trait]
pub trait JettonMasterMethods: TonContract {
    async fn get_jetton_data(&self) -> Result<GetJettonDataResult, TonError> {
        self.emulate_get_method("get_jetton_data", &TVMStack::EMPTY, None).await
    }

    async fn get_wallet_address(&self, owner: &TonAddress) -> Result<GetWalletAddressResult, TonError> {
        let mut stack = TVMStack::default();
        stack.push_cell_slice(owner.to_cell()?);
        self.emulate_get_method("get_wallet_address", &stack, None).await
    }
}

#[async_trait]
pub trait ScaledUIMethods: TonContract {
    async fn get_display_multiplier(&self) -> Result<GetDisplayMultiplierResult, TonError> {
        self.emulate_get_method("get_display_multiplier", &TVMStack::EMPTY, None).await
    }
}
