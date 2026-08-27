//! TON clients, contract wrappers, wallet support, and protocol types.
//!
//! Provider-neutral contract APIs are available by default. Enable
//! `lite-client` for ADNL access or `tonlibjson` for native clients and emulators.

#[doc(hidden)]
pub use async_trait::async_trait as __async_trait;
pub use ton_core; // re-export
pub mod block_tlb;
pub mod contracts;
pub mod errors;
pub mod libs_dict;
#[cfg(feature = "lite-client")]
pub mod lite_client;
pub mod net_config;
pub mod tlb_adapters;
pub mod ton_wallet;

#[cfg(feature = "tonlibjson")]
pub mod emulators;
#[cfg(feature = "tonlibjson")]
pub mod sys_utils;
#[cfg(feature = "tonlibjson")]
pub mod test_utils;
#[cfg(feature = "tonlibjson")]
pub mod tl_client;
