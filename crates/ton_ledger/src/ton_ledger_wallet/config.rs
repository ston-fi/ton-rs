//! Wallet derivation, display options and signing policy.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
/// Key path configuration. Input components are unhardened; encoding hardens once.
pub enum DerivationPath {
    /// Conventional 44'/607'/network'/chain'/account'/0'.
    Ton {
        /// Account index below 2^31.
        account: u32,
        /// Selects network component 1 instead of 0; does not prevent cross-network replay.
        testnet: bool,
    },
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
/// Policy for transaction fields the device cannot display semantically.
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
    /// Displays a testnet-friendly address; defaults to false and does not change the key.
    pub testnet: bool,
}
