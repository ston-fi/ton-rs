use crate::contracts::TonContract;
use crate::errors::TonResult;
use crate::tep::tvm_result::{GetDisplayMultiplierResult, GetJettonDataResult};
use crate::ton_contract;
use async_trait::async_trait;
use ton_core::types::TonAddress;
use ton_macros::ton_methods;

// https://github.com/ton-blockchain/TEPs/blob/master/text/0074-jettons-standard.md#jetton-master-contract
ton_contract!(JettonMasterContract: JettonMasterMethods);

// https://github.com/the-ton-tech/TEPs/blob/scaled-ui/text/0000-scaled-ui-jettons.md
ton_contract!(JettonScaledUIMasterContract: JettonMasterMethods, ScaledUIMethods);

#[async_trait]
#[ton_methods]
pub trait JettonMasterMethods: TonContract {
    async fn get_jetton_data(&self) -> TonResult<GetJettonDataResult>;
    async fn get_wallet_address(&self, owner: &TonAddress) -> TonResult<TonAddress>;
}

#[async_trait]
#[ton_methods]
pub trait ScaledUIMethods: TonContract {
    async fn get_display_multiplier(&self) -> TonResult<GetDisplayMultiplierResult>;
}
