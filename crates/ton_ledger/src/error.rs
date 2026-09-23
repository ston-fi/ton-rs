//! Typed protocol and transport failures.
use thiserror::Error;
/// Transport errors preserve backend causes without exposing backend types.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum TransportError {
    #[error("device operation timed out")]
    Timeout,
    #[error("device disconnected")]
    Disconnected,
    #[error("device access denied: {0}")]
    Permission(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("malformed transport frame: {0}")]
    Frame(&'static str),
    #[error("backend error: {0}")]
    Backend(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("no compatible Ledger device found")]
    NoDevice,
    #[error("multiple Ledger devices found; select one explicitly")]
    AmbiguousDevice,
    #[error("Ledger device is already owned by another session in this process; wait for it to close")]
    DeviceBusy,
}
/// Invalid inputs fail before device I/O; uncertain sessions cannot be reused.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum TonLedgerError {
    #[error(transparent)]
    Transport(#[from] TransportError),
    #[error(transparent)]
    Ton(#[from] ton::errors::TonError),
    #[error(transparent)]
    Cell(#[from] ton::ton_core::errors::TonCoreError),
    #[error("invalid Ledger input: {0}")]
    Invalid(&'static str),
    #[error("unsupported wallet version")]
    UnsupportedWallet,
    #[error("unvalidated TON app version {0:?}; source baseline is 2.9.1")]
    UnvalidatedFirmware([u8; 3]),
    #[error("open the TON app on the Ledger")]
    WrongApp,
    #[error("no transport configured and the hid feature is disabled")]
    MissingTransport,
    #[error("session unusable after interrupted or uncertain I/O; reconnect and rebuild the wallet")]
    DirtySession,
    #[error("device rejected the request")]
    UserDenied,
    #[error("device does not support this command")]
    UnsupportedCommand,
    #[error("enable blind signing in the TON app for explicitly permitted opaque data")]
    BlindSigningDisabled,
    #[error("device status 0x{0:04x}")]
    Status(u16),
    #[error("malformed device response: {0}")]
    Response(&'static str),
    #[error("device public key changed")]
    IdentityChanged,
    #[error("device signed a different hash")]
    HashMismatch,
    #[error("invalid Ed25519 signature or public key")]
    Signature,
    #[error("payload requires explicit opaque-signing permission")]
    OpaquePayload,
}

/// Result of a TON Ledger wallet operation.
pub type TonLedgerResult<T> = Result<T, TonLedgerError>;
