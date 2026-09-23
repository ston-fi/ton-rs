//! Wallet derivation, display options and signing policy.
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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum SigningPolicy {
    /// Only recognized payloads whose nested content is also displayed.
    #[default]
    ClearOnly,
    /// Permit unknown payloads, state init and opaque nested references by hash.
    AllowOpaque,
}

/// Friendly-address display flag, independent of key derivation and raw address.
/// Firmware always displays bounceable addresses.
#[derive(Debug, Clone, Copy, Default, derive_setters::Setters)]
#[setters(prefix = "with_")]
#[non_exhaustive]
pub struct AddressOptions {
    pub testnet: bool,
}
