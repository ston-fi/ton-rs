//! Hardened TON account derivation.
use crate::error::{TonLedgerError, TonLedgerResult};
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum DerivationPath {
    /// Conventional 44'/607'/network'/chain'/account'/0'.
    Ton { account: u32, testnet: bool },
    /// Unhardened indexes, starting with 44, 607; used exactly as supplied.
    Custom(Vec<u32>),
}
impl Default for DerivationPath {
    fn default() -> Self {
        Self::Ton {
            account: 0,
            testnet: false,
        }
    }
}
impl DerivationPath {
    pub(crate) fn encode(&self, workchain: i32) -> TonLedgerResult<Vec<u8>> {
        let p = match self {
            Self::Ton { account, testnet } => vec![
                44,
                607,
                u32::from(*testnet),
                if workchain == -1 { 255 } else { 0 },
                *account,
                0,
            ],
            Self::Custom(p) => p.clone(),
        };
        if !(3..=10).contains(&p.len()) || p.get(..2) != Some(&[44, 607]) || p.iter().any(|v| *v >= 0x80000000) {
            return Err(TonLedgerError::Invalid("path must have 3..10 unhardened indexes with prefix 44/607"));
        }
        let mut out = vec![p.len() as u8];
        for v in p {
            out.extend((v | 0x80000000).to_be_bytes());
        }
        Ok(out)
    }
}
