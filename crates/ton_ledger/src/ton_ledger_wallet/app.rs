//! TON app inspection results.
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
