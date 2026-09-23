#![warn(missing_docs)]
#![doc = include_str!("../README.md")]
//! TON Ledger V3R2/V4R2 signing with local message and Ed25519 verification.
//! The wallet owns an exclusive session; cancelled or uncertain operations require
//! dropping it and reconnecting. No provider, balance cache or broadcasting is owned here.
pub mod error;
pub mod ton_ledger_wallet;
pub mod transports;

mod protocol;
