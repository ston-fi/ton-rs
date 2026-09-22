//! TON app inspection and address display options.
/// Source-validated firmware identity; this does not attest the installed binary.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct AppInfo {
    pub name: String,
    pub version: [u8; 3],
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct AppSettings {
    pub blind_signing: bool,
    pub expert_mode: bool,
}
/// Friendly-address display flag, independent of key derivation and raw address.
/// Firmware always displays bounceable addresses.
#[derive(Debug, Clone, Copy, Default, derive_setters::Setters)]
#[setters(prefix = "with_")]
#[non_exhaustive]
pub struct AddressOptions {
    pub testnet: bool,
}
