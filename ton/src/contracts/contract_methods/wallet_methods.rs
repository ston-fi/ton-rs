use crate::block_tlb::TVMStack;
use crate::contracts::TonContract;
use crate::errors::TonError;
use async_trait::async_trait;
use ton_core::TLB;
use ton_core::cell::TonHash;
use ton_core::traits::tlb::TLB;

#[derive(TLB)]
struct TonWalletPK {
    pub pk: TonHash,
}

#[derive(TLB)]
struct TonWalletSeqno {
    pub seqno: i64,
}

#[async_trait]
pub trait TonWalletMethods: TonContract {
    async fn seqno(&self) -> Result<u32, TonError> {
        let wallet_seqno = self.emulate_get_method::<_, TonWalletSeqno>("seqno", &TVMStack::EMPTY, None).await?;
        let seqno_int = wallet_seqno.seqno;
        if seqno_int < 0 {
            return Err(TonError::UnexpectedValue {
                expected: "non-negative integer".to_string(),
                actual: seqno_int.to_string(),
            });
        }
        Ok(seqno_int as u32)
    }

    async fn get_public_key(&self) -> Result<TonHash, TonError> {
        let wallet_pk = self.emulate_get_method::<_, TonWalletPK>("get_public_key", &TVMStack::EMPTY, None).await?;
        Ok(wallet_pk.pk)
    }
}
