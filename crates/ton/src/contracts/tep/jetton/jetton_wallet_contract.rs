//! Jetton wallet contract wrapper, get-method trait, and result type.

use crate::contracts::TonContract;
use crate::errors::TonResult;
use crate::ton_contract;
use async_trait::async_trait;
use fastnum::I512;
use ton_core::cell::TonCell;
use ton_core::types::TonAddress;
use ton_macros::{FromTVMStack, ton_methods};

// https://github.com/ton-blockchain/TEPs/blob/master/text/0074-jettons-standard.md#jetton-wallet-smart-contract
ton_contract!(JettonWalletContract: JettonWalletMethods);

#[async_trait]
#[ton_methods]
pub trait JettonWalletMethods: TonContract {
    async fn get_wallet_data(&self) -> TonResult<GetWalletDataResult>;
}

/// Result of `get_wallet_data`.
#[derive(Debug, Clone, PartialEq, Eq, FromTVMStack)]
#[from_tvm_stack(allow_extra = true)]
#[non_exhaustive]
pub struct GetWalletDataResult {
    pub balance: I512,
    pub owner: TonAddress,
    pub master: TonAddress,
    pub wallet_code: TonCell,
}

#[cfg(test)]
#[path = "../../../tests/contracts/tep/jetton/test_jetton_wallet_contract.rs"]
mod tests;
