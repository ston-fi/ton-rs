use crate::block_tlb::TVMStack;
use crate::contracts::TonContract;
use crate::errors::TonError;
use async_trait::async_trait;
use ton_core::cell::TonHash;

#[async_trait]
pub trait TonWalletMethods: TonContract {
    async fn seqno(&self) -> Result<u32, TonError> {
        let wallet_seqno: u32 = self.emulate_get_method("seqno", &TVMStack::EMPTY, None).await?;
        Ok(wallet_seqno)
    }

    async fn get_public_key(&self) -> Result<TonHash, TonError> {
        let wallet_pk: TonHash = self.emulate_get_method("get_public_key", &TVMStack::EMPTY, None).await?;
        Ok(wallet_pk)
    }
}
