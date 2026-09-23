//! TON app inspection results.
/// App-reported identity accepted by the version policy; this does not attest the installed binary.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct AppInfo {
    /// Validated app name (`TON`).
    pub name: String,
    /// Reported major, minor and patch version; only major version 2 is accepted.
    pub version: [u8; 3],
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
/// Current flags reported by the TON app; obtained through the wallet.
pub struct AppSettings {
    /// Whether the device permits blind signing.
    pub blind_signing: bool,
    /// Whether the device reports expert mode enabled.
    pub expert_mode: bool,
}
