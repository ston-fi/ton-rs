pub use ton_lib_macros::*; // re-export
pub mod bits_utils;
pub mod cell;
pub mod constants;
pub mod errors;
pub mod traits;
pub mod types;

pub static TON_TESTNET: std::sync::LazyLock<bool> = std::sync::LazyLock::new(|| {
    let env_var = std::env::var("TON_TESTNET").unwrap_or("false".to_string());
    env_var.to_lowercase() == "true"
});
