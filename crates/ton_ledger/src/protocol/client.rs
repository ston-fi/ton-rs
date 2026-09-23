use crate::{
    error::{TonLedgerError, TonLedgerResult, TransportError},
    protocol::apdu,
    ton_ledger_wallet::app::{AppInfo, AppSettings},
    transports::Transport,
};
use ed25519_dalek::{Signature, VerifyingKey};
use std::time::Duration;
use tokio::time::{Instant, timeout_at};

/// Owns the transport and keeps the session dirty from first I/O until a known
/// terminal response. Dropping an operation future intentionally skips reset.
pub(crate) struct Client {
    transport: Box<dyn Transport>,
    dirty: bool,
    request_timeout: Duration,
    approval_timeout: Duration,
}
impl Client {
    pub(crate) fn new(transport: Box<dyn Transport>, request_timeout: Duration, approval_timeout: Duration) -> Self {
        Self {
            transport,
            dirty: false,
            request_timeout,
            approval_timeout,
        }
    }
    pub(crate) async fn request(
        &mut self,
        ins: u8,
        p1: u8,
        p2: u8,
        data: &[u8],
        approval: bool,
    ) -> TonLedgerResult<Vec<u8>> {
        let command = apdu::command(ins, p1, p2, data)?;
        self.begin()?;
        let budget = if approval { self.approval_timeout } else { self.request_timeout };
        let res = self.exchange(&command, Instant::now() + budget).await;
        self.finish(&res);
        res
    }
    fn begin(&mut self) -> TonLedgerResult<()> {
        if self.dirty {
            return Err(TonLedgerError::DirtySession);
        }
        self.dirty = true;
        Ok(())
    }
    fn finish<T>(&mut self, res: &TonLedgerResult<T>) {
        // Cancellation never reaches here. Only a complete successful operation or
        // terminal user denial establishes a known idle protocol state.
        if res.is_ok() || matches!(res, Err(TonLedgerError::UserDenied)) {
            self.dirty = false;
        }
    }
    async fn exchange(&mut self, command: &[u8], deadline: Instant) -> TonLedgerResult<Vec<u8>> {
        let remaining = deadline.saturating_duration_since(Instant::now());
        let bytes = timeout_at(deadline, self.transport.exchange(command, remaining))
            .await
            .map_err(|_| TransportError::Timeout)??;
        apdu::response(bytes)
    }
    pub(crate) async fn chunked(&mut self, ins: u8, path: &[u8], data: &[u8]) -> TonLedgerResult<Vec<u8>> {
        if data.is_empty() || data.len() > 510 {
            return Err(TonLedgerError::Invalid("chunked payload must contain 1..510 bytes"));
        }
        let first = apdu::command(ins, 0, 3, path)?;
        self.begin()?;
        let res = self.chunked_inner(ins, &first, data).await;
        self.finish(&res);
        res
    }
    async fn chunked_inner(&mut self, ins: u8, first: &[u8], data: &[u8]) -> TonLedgerResult<Vec<u8>> {
        let deadline = Instant::now() + self.request_timeout;
        if !self.exchange(first, deadline).await?.is_empty() {
            return Err(TonLedgerError::Response("nonempty path acknowledgement"));
        }
        let chunks = data.chunks(255);
        let count = chunks.len();
        for (i, chunk) in chunks.enumerate() {
            let last = i + 1 == count;
            let command = apdu::command(ins, 0, if last { 0 } else { 2 }, chunk)?;
            let result =
                self.exchange(&command, if last { Instant::now() + self.approval_timeout } else { deadline }).await?;
            if last {
                return Ok(result);
            }
            if !result.is_empty() {
                return Err(TonLedgerError::Response("nonempty chunk acknowledgement"));
            }
        }
        Err(TonLedgerError::Invalid("empty signing payload"))
    }
    pub(crate) async fn app_info(&mut self) -> TonLedgerResult<AppInfo> {
        let name = self.request(4, 0, 0, &[], false).await?;
        if name != b"TON" {
            return Err(TonLedgerError::WrongApp);
        }
        let version: [u8; 3] = self
            .request(3, 0, 0, &[], false)
            .await?
            .try_into()
            .map_err(|_| TonLedgerError::Response("version length"))?;
        if version[0] != 2 {
            return Err(TonLedgerError::UnvalidatedFirmware(version));
        }
        Ok(AppInfo {
            name: "TON".into(),
            version,
        })
    }
    pub(crate) async fn settings(&mut self) -> TonLedgerResult<AppSettings> {
        let flags = self.request(10, 0, 0, &[], false).await?;
        if flags.len() != 1 || flags[0] & !3 != 0 {
            return Err(TonLedgerError::Response("settings flags"));
        }
        Ok(AppSettings {
            blind_signing: flags[0] & 1 != 0,
            expert_mode: flags[0] & 2 != 0,
        })
    }
    pub(crate) async fn key(&mut self, path: &[u8]) -> TonLedgerResult<[u8; 32]> {
        self.request(5, 0, 0, path, false).await?.try_into().map_err(|_| TonLedgerError::Response("public key length"))
    }
    pub(crate) fn invalidate(&mut self) {
        self.dirty = true;
    }
}
pub(crate) fn verify(response: &[u8], key: &[u8; 32], hash: &[u8], preimage: &[u8]) -> TonLedgerResult<[u8; 64]> {
    if response.len() != 98 || response[0] != 64 || response[65] != 32 {
        return Err(TonLedgerError::Response("signature framing"));
    }
    if &response[66..] != hash {
        return Err(TonLedgerError::HashMismatch);
    }
    let sig: [u8; 64] = response[1..65].try_into().map_err(|_| TonLedgerError::Response("signature length"))?;
    let key = VerifyingKey::from_bytes(key).map_err(|_| TonLedgerError::Signature)?;
    key.verify_strict(preimage, &Signature::from_bytes(&sig)).map_err(|_| TonLedgerError::Signature)?;
    Ok(sig)
}
