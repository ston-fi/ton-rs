//! Private Ledger firmware protocol and session implementation.
pub(crate) mod apdu;
pub(crate) mod client;
pub(crate) mod data;
pub(crate) mod derivation_path;
pub(crate) mod encoding;
pub(crate) mod payload;
pub(crate) mod proof;

#[cfg(test)]
mod _test_protocol;
