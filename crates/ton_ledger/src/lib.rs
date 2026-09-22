#![doc = include_str!("../README.md")]
//! TON Ledger V3R2/V4R2 signing with local message and Ed25519 verification.
//! The wallet owns an exclusive session; cancelled or uncertain operations require
//! dropping it and reconnecting. No provider, balance cache or broadcasting is owned here.
pub mod app;
mod client;
pub mod data;
pub mod derivation_path;
pub mod error;
mod payload;
pub mod proof;
mod protocol;
pub mod signing;
pub mod ton_ledger_wallet;
pub mod traits;
pub mod transports;

#[cfg(test)]
mod _test_protocol;
