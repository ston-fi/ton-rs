//! Typed protocol and transport failures.
use thiserror::Error;
/// Transport errors preserve backend causes without exposing backend types.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum TransportError {
    /// The operation exceeded its budget; an OS operation or prompt may still be active.
    #[error("device operation timed out")]
    Timeout,
    /// The device or its worker is no longer available.
    #[error("device disconnected")]
    Disconnected,
    /// OS access was denied; inspect the preserved cause and device permissions.
    #[error("device access denied: {0}")]
    Permission(#[source] Box<dyn std::error::Error + Send + Sync>),
    /// The transport received an unsupported or malformed frame.
    #[error("malformed transport frame: {0}")]
    Frame(&'static str),
    /// Native backend failure with its original source.
    #[error("backend error: {0}")]
    Backend(#[source] Box<dyn std::error::Error + Send + Sync>),
    /// USB discovery found no compatible application interface.
    #[error("no compatible Ledger device found")]
    NoDevice,
    /// USB discovery found several devices; select a descriptor explicitly.
    #[error("multiple Ledger devices found; select one explicitly")]
    AmbiguousDevice,
    /// The backend is owned or quarantined; uncertain BLE cleanup requires process restart.
    #[error("Ledger device is owned or quarantined in this process; failed BLE cleanup requires process restart")]
    DeviceBusy,
}
/// Invalid inputs fail before device I/O; uncertain sessions cannot be reused.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum TonLedgerError {
    /// Transport failure; inspect the underlying error for recovery requirements.
    #[error(transparent)]
    Transport(#[from] TransportError),
    /// TON wallet or message construction failed.
    #[error(transparent)]
    Ton(#[from] ton::errors::TonError),
    /// Cell parsing or serialization failed.
    #[error(transparent)]
    Cell(#[from] ton::ton_core::errors::TonCoreError),
    /// Input violates a supported Ledger encoding or configuration constraint.
    #[error("invalid Ledger input: {0}")]
    Invalid(&'static str),
    /// Only V3R2 and V4R2 wallet code is supported.
    #[error("unsupported wallet version")]
    UnsupportedWallet,
    /// The app major version is outside the supported TON 2.x series.
    #[error("unvalidated TON app version {0:?}; expected major version 2")]
    UnvalidatedFirmware([u8; 3]),
    /// The device did not report the TON app.
    #[error("open the TON app on the Ledger")]
    WrongApp,
    /// No custom transport was provided and default USB support is disabled.
    #[error("no transport configured and the hid feature is disabled")]
    MissingTransport,
    /// An interrupted or uncertain operation requires dropping and rebuilding the wallet.
    #[error("session unusable after interrupted or uncertain I/O; reconnect and rebuild the wallet")]
    DirtySession,
    /// The device rejected approval; the protocol is idle and the session can be reused.
    #[error("device rejected the request")]
    UserDenied,
    /// The device rejected an unsupported instruction.
    #[error("device does not support this command")]
    UnsupportedCommand,
    /// The requested opaque operation requires enabling blind signing on the device.
    #[error("enable blind signing in the TON app for explicitly permitted opaque data")]
    BlindSigningDisabled,
    /// Unrecognized device status word, preserved for diagnosis.
    #[error("device status 0x{0:04x}")]
    Status(u16),
    /// A device reply violated the expected format.
    #[error("malformed device response: {0}")]
    Response(&'static str),
    /// The device returned a different public key; the session is invalidated.
    #[error("device public key changed")]
    IdentityChanged,
    /// The signed hash differs from local reconstruction; the session is invalidated.
    #[error("device signed a different hash")]
    HashMismatch,
    /// The public key or Ed25519 signature is invalid.
    #[error("invalid Ed25519 signature or public key")]
    Signature,
    /// The transaction requires explicit `AllowOpaque` policy.
    #[error("payload requires explicit opaque-signing permission")]
    OpaquePayload,
}

/// Result of a TON Ledger wallet operation.
pub type TonLedgerResult<T> = Result<T, TonLedgerError>;
