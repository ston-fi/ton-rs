use hex::FromHexError;
use std::env::VarError;
use std::sync::Arc;
use thiserror::Error;

#[macro_export]
macro_rules! bail_ton_core {
    ($($arg:tt)*) => {
        return Err(TonCoreError::Custom(format!($($arg)*)))
    };
}

#[macro_export]
macro_rules! bail_ton_core_data {
    ($($arg:tt)*) => {
        return Err(TonCoreError::data(module_path!(), format!($($arg)*)))
    };
}

#[derive(Error, Debug)]
pub enum TonCoreError {
    #[error("DataError: [{producer}] {msg}")]
    DataError { producer: String, msg: String },

    // tlb
    #[error("TLBWrongPrefix: expected={exp}, given={given}, exp_bits={bits_exp}, left_bits={bits_left}")]
    TLBWrongPrefix {
        exp: usize,
        given: usize,
        bits_exp: usize,
        bits_left: usize,
    },
    #[error("TLBEnumOutOfOptions: data doesn't match any variant of {0}")]
    TLBEnumOutOfOptions(String),

    // contracts
    #[error("ContractError: {0}")]
    ContractError(String),

    // General errors
    #[error("Custom: {0}")]
    Custom(String),

    // handling external errors
    #[error("{0}")]
    IO(#[from] std::io::Error),
    #[error("{0}")]
    FromHex(#[from] FromHexError),
    #[error("{0}")]
    Base64Error(#[from] base64::DecodeError),
    #[error("{0}")]
    ParseInt(#[from] std::num::ParseIntError),
    #[error("{0}")]
    FromUtf8(#[from] std::string::FromUtf8Error),
    #[error("{0}")]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error("{0}")]
    NulError(#[from] std::ffi::NulError),
    #[error("{0}")]
    SystemTimeError(#[from] std::time::SystemTimeError),

    #[error("{0}")]
    ParseBigIntError(#[from] num_bigint::ParseBigIntError),
    #[error("{0}")]
    VarError(#[from] VarError),
    #[error("{0}")]
    BoxedError(#[from] Box<dyn std::error::Error + Send + Sync>),
    #[error("{0}")]
    ArcError(#[from] Arc<dyn std::error::Error + Send + Sync>),
    #[error("{0}")]
    ArcSelf(#[from] Arc<TonCoreError>),
}

impl TonCoreError {
    pub fn data<P: Into<String>, M: Into<String>>(producer: P, msg: M) -> Self {
        Self::DataError {
            producer: producer.into(),
            msg: msg.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test::assert_err;

    fn make_data_error() -> Result<(), TonCoreError> {
        let val = "42";
        bail_ton_core_data!("some_error, val={val}");
    }

    #[test]
    fn test_bail_ton_core_data() {
        let rs = make_data_error();
        let err = assert_err!(rs);
        assert_eq!(err.to_string(), "DataError: [ton_lib_core::errors::tests] some_error, val=42");
    }
}
