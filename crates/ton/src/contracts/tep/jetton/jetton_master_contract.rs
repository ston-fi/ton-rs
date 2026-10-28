//! Jetton master contract wrappers, get-method traits, and result types.

use crate::contracts::TonContract;
use crate::contracts::tep::metadata::metadata_content::MetadataContent;
use crate::errors::TonResult;
use crate::ton_contract;
use async_trait::async_trait;
use fastnum::I512;
use ton_core::cell::TonCell;
use ton_core::traits::tlb::TLB;
use ton_core::types::{Coins, TonAddress};
use ton_macros::{FromTVMStack, ton_methods};

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

/// Result of `get_jetton_data`.
#[derive(Debug, Clone, PartialEq, FromTVMStack)]
#[from_tvm_stack(allow_extra = true)]
#[non_exhaustive]
pub struct GetJettonDataResult {
    pub total_supply: Coins,
    pub mintable: bool,
    pub admin: TonAddress,
    pub content: TonCell,
    pub wallet_code: TonCell,
}

impl GetJettonDataResult {
    /// Parses the metadata content cell.
    pub fn content_parsed(&self) -> Result<MetadataContent, ton_core::errors::TonCoreError> {
        MetadataContent::from_cell(&self.content)
    }
}

/// Result of `get_wallet_address`.
#[derive(Debug, Clone, PartialEq, Eq, FromTVMStack)]
#[non_exhaustive]
pub struct GetWalletAddressResult {
    pub address: TonAddress,
}

/// Result of `get_display_multiplier`.
#[derive(Debug, Clone, PartialEq, FromTVMStack)]
#[from_tvm_stack(ensure_empty = true)]
#[non_exhaustive]
pub struct GetDisplayMultiplierResult {
    pub numerator: I512,
    pub denominator: I512,
}

// TVMType trait implementation tested in assert_jetton_master_scaled_ui

#[cfg(test)]
#[path = "../../../tests/contracts/tep/jetton/test_jetton_master_contract.rs"]
mod tests;
