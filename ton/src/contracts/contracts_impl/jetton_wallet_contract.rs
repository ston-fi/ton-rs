use crate::block_tlb::TVMStack;
use crate::contracts::TonContract;
use crate::errors::{TonError, TonResult};
use crate::tep::tvm_result::GetWalletDataResult;
use crate::ton_contract;
use async_trait::async_trait;
use ton_macros::ton_method;

// https://github.com/ton-blockchain/TEPs/blob/master/text/0074-jettons-standard.md#jetton-wallet-smart-contract
ton_contract!(JettonWalletContract: JettonWalletMethods);

#[async_trait]
pub trait JettonWalletMethods: TonContract {
    #[ton_method]
    async fn get_wallet_data(&self) -> TonResult<GetWalletDataResult>;

    // async fn get_wallet_data(&self) -> Result<GetWalletDataResult, TonError> { self.emulate_get_method("get_wallet_data", &crate::block_tlb::TVMStack::EMPTY, None).await }

    // async fn get_wallet_data(&self) -> Result<GetWalletDataResult, TonError> {  self.emulate_get_method("get_wallet_data", &crate::block_tlb::TVMStack::EMPTY, None).await }
    // async fn get_wallet_data(&self) -> Result<GetWalletDataResult, TonError> { self.emulate_get_method("get_wallet_data", &crate::block_tlb::TVMStack::EMPTY, None).await }
}
