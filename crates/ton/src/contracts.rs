//! Contract state loading and provider-neutral TVM get-method execution.

mod contract_client;
pub mod tep;
mod ton_contract;
mod tvm_get_method;

pub use contract_client::*;
pub use ton_contract::*;
pub use tvm_get_method::*;

#[cfg(test)]
#[path = "tests/test_contracts.rs"]
mod tests;
