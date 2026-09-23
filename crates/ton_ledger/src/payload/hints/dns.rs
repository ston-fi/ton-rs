use super::{Hint, LedgerHintEncode, unsupported_record};
use crate::{
    error::TonLedgerResult,
    payload::{
        encoding, exact,
        tlb::{DnsCapabilities, DnsChangeRecord, DnsWalletRecord},
    },
    protocol,
    signing::SigningPolicy,
};
use sha2::{Digest, Sha256};

impl LedgerHintEncode for DnsChangeRecord {
    fn encode_hint(&self, policy: SigningPolicy) -> TonLedgerResult<Hint> {
        let mut output = Vec::new();
        encoding::query_id(&mut output, &self.query_id, policy)?;
        let is_wallet = self.key.as_slice() == Sha256::digest(b"wallet").as_slice();
        output.extend([u8::from(self.record.0.is_some()), u8::from(!is_wallet)]);
        if is_wallet {
            if let Some(record) = &self.record.0 {
                let wallet_record: DnsWalletRecord = exact(record).map_err(unsupported_record)?;
                encoding::address(&mut output, &wallet_record.address, policy)?;
                match wallet_record.capabilities {
                    DnsCapabilities::Absent(_) => output.push(0),
                    DnsCapabilities::Present(list) => output.extend([1, u8::from(list.wallet.is_some())]),
                }
            }
        } else {
            encoding::opaque(policy)?;
            output.extend(self.key.as_slice());
            if let Some(record) = &self.record.0 {
                protocol::cell_ref(&mut output, record)?;
            }
        }
        Ok(Hint { id: 9, data: output })
    }
}
