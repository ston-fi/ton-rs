//! Infallible configuration followed by validated async connection.
use super::TonLedgerWallet;
use crate::{
    client::Client,
    derivation_path::DerivationPath,
    error::{TonLedgerError, TonLedgerResult},
    signing::SigningPolicy,
    traits::Transport,
};
use derive_setters::Setters;
use std::time::Duration;
use ton::ton_wallet::{WALLET_ID_DEFAULT, WalletVersion};
#[derive(Setters)]
#[setters(prefix = "with_", strip_option)]
pub struct Builder {
    #[setters(skip)]
    version: WalletVersion,
    #[setters(skip)]
    transport: Option<Box<dyn Transport>>,
    derivation_path: DerivationPath,
    workchain: i32,
    wallet_id: i32,
    signing_policy: SigningPolicy,
    request_timeout: Duration,
    approval_timeout: Duration,
}
impl Builder {
    pub(super) fn new(version: WalletVersion) -> Self {
        Self {
            version,
            transport: None,
            derivation_path: DerivationPath::default(),
            workchain: 0,
            wallet_id: WALLET_ID_DEFAULT,
            signing_policy: SigningPolicy::default(),
            request_timeout: Duration::from_secs(10),
            approval_timeout: Duration::from_secs(180),
        }
    }
    /// Supplies an exclusive custom session instead of default USB discovery.
    pub fn with_transport(mut self, transport: impl Transport + 'static) -> Self {
        self.transport = Some(Box::new(transport));
        self
    }
    /// Validates all configuration before discovery; checks app and binds the public key.
    pub async fn build(self) -> TonLedgerResult<TonLedgerWallet> {
        if !matches!(self.version, WalletVersion::V3R2 | WalletVersion::V4R2) {
            return Err(TonLedgerError::UnsupportedWallet);
        }
        if !matches!(self.workchain, 0 | -1) {
            return Err(TonLedgerError::Invalid("workchain must be 0 or -1"));
        }
        if self.request_timeout.is_zero()
            || self.approval_timeout.is_zero()
            || std::time::Instant::now().checked_add(self.request_timeout).is_none()
            || std::time::Instant::now().checked_add(self.approval_timeout).is_none()
        {
            return Err(TonLedgerError::Invalid("timeouts must be positive and representable"));
        }
        let path = self.derivation_path.encode(self.workchain)?;
        let transport = match self.transport {
            Some(t) => t,
            None => {
                #[cfg(feature = "hid")]
                {
                    Box::new(crate::transports::hid::HidTransport::connect_default(self.request_timeout).await?)
                        as Box<dyn Transport>
                }
                #[cfg(not(feature = "hid"))]
                {
                    return Err(TonLedgerError::MissingTransport);
                }
            },
        };
        let mut client = Client::new(transport, self.request_timeout, self.approval_timeout);
        client.app_info().await?;
        let public_key = client.key(&path).await?;
        ed25519_dalek::VerifyingKey::from_bytes(&public_key).map_err(|_| TonLedgerError::Signature)?;
        let address = super::state_init(self.version, &public_key, self.wallet_id)?.derive_address(self.workchain)?;
        Ok(TonLedgerWallet {
            client,
            version: self.version,
            public_key,
            address,
            wallet_id: self.wallet_id,
            derivation_path: self.derivation_path,
            path,
            policy: self.signing_policy,
        })
    }
}
