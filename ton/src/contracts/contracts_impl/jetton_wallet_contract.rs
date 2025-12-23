use crate::contracts::TonContract;
use crate::errors::TonResult;
use crate::tep::tvm_result::GetWalletDataResult;
use crate::ton_contract;
use async_trait::async_trait;
use ton_macros::ton_methods;

// https://github.com/ton-blockchain/TEPs/blob/master/text/0074-jettons-standard.md#jetton-wallet-smart-contract
ton_contract!(JettonWalletContract: JettonWalletMethods);

#[async_trait]
#[ton_methods]
pub trait JettonWalletMethods: TonContract {
    async fn get_wallet_data(&self) -> TonResult<GetWalletDataResult>;
}
