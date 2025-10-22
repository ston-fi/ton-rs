use crate::block_tlb::TVMStack;
use crate::contracts::TonContract;
use crate::errors::TonError;
use async_trait::async_trait;
use ton_core::cell::TonHash;
use ton_core::traits::tlb::TLB;

#[async_trait]
pub trait TonWalletMethods: TonContract {
    async fn seqno(&self) -> Result<u32, TonError> {
        let stack_boc = self.emulate_get_method("seqno", &TVMStack::EMPTY).await?;
        let seqno_int = TVMStack::from_boc(stack_boc)?.pop_tiny_int()?;
        if seqno_int < 0 {
            return Err(TonError::UnexpectedValue {
                expected: "non-negative integer".to_string(),
                actual: seqno_int.to_string(),
            });
        }
        Ok(seqno_int as u32)
    }

    async fn get_public_key(&self) -> Result<TonHash, TonError> {
        let stack_boc = self.emulate_get_method("get_public_key", &TVMStack::EMPTY).await?;
        Ok(TonHash::from_num(&TVMStack::from_boc(stack_boc)?.pop_int()?)?)
    }
}
