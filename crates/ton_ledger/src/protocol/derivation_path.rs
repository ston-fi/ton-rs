//! Firmware derivation-path encoding.
use crate::{
    error::{TonLedgerError, TonLedgerResult},
    ton_ledger_wallet::config::DerivationPath,
};

pub(crate) fn encode(path: &DerivationPath, workchain: i32) -> TonLedgerResult<Vec<u8>> {
    let p = match path {
        DerivationPath::Ton { account, testnet } => vec![
            44,
            607,
            u32::from(*testnet),
            if workchain == -1 { 255 } else { 0 },
            *account,
            0,
        ],
        DerivationPath::Custom(p) => p.clone(),
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
