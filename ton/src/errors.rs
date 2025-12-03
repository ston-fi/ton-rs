use crate::tep::metadata::MetadataContent;
use hmac::digest::crypto_common;
use reqwest::StatusCode;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::time::error::Elapsed;
use ton_core::cell::TonHash;
use ton_core::errors::TonCoreError;
use ton_core::types::{TonAddress, TxLTHash};
use ton_liteapi::tl::request::Request;
use ton_liteapi::types::LiteError;

#[macro_export]
macro_rules! bail_ton {
    ($($arg:tt)*) => {
        return Err(TonError::Custom(format!($($arg)*)))
    };
}

pub type TonResult<T> = Result<T, TonError>;

#[derive(Error, Debug)]
pub enum TonError {
    // handling system errors such as mutex.lock(), system_time, etc.
    #[error("SystemError: {0}")]
    SystemError(String),
    #[error("TLCoreError: {0}")]
    TLCoreError(#[from] TonCoreError),
    #[error("{0}")]
    ArcSelf(Arc<TonError>),
    #[error("Failed to parse metadata")]
    MetadataParseError,
    #[error("NetRequestTimeout: {msg}, timeout={timeout:?}")]
    NetRequestTimeout { msg: String, timeout: Duration },

    // LiteClient
    #[error("LiteClientErrorResponse: {0:?}")]
    LiteClientErrorResponse(ton_liteapi::tl::response::Error),
    #[error("LiteClientWrongResponse: expected {0}, got {1}")]
    LiteClientWrongResponse(String, String),
    #[error("LiteClientLiteError: {0}")]
    LiteClientLiteError(#[from] LiteError),
    #[error("LiteClientConnTimeout: {0:?}")]
    LiteClientConnTimeout(Duration),
    #[error("LiteClientReqTimeout: {0:?}")]
    LiteClientReqTimeout(Box<(Request, Duration)>),

    // TonlibClient
    #[error("TLClientCreationFailed: tonlib_client_json_create returns null")]
    TLClientCreationFailed,
    #[error("TLClientWrongResponse: expected type: {0}, got: {1}")]
    TLClientWrongResponse(String, String),
    #[error("TLClientResponseError: code: {code}, msg: {msg}")]
    TLClientResponseError { code: i32, msg: String },
    #[error("TLWrongArgs: {0}")]
    TLWrongArgs(String),
    #[error("TLSendError: fail to send request: {0}")]
    TLSendError(String),
    #[error("TLExecError: method: {method}, code: {code}, msg: {msg}")]
    TLExecError { method: String, code: i32, msg: String },
    #[error("TLWrongUsage: {0}")]
    TLWrongUsage(String),

    // Emulators
    #[error("TVMEmulatorCreationFailed: emulator_create returns null")]
    EmulatorCreationFailed,
    #[error("TVMEmulatorSetFailed: fail to set param: {0}")]
    EmulatorSetParamFailed(&'static str),
    #[error("EmulatorNullResponse: emulator returns nullptr")]
    EmulatorNullResponse,
    #[error("TVMEmulatorResponseParseError: {field}, raw_response: {raw_response}")]
    EmulatorParseResponseError { field: &'static str, raw_response: String },
    #[error("EmulatorEmulationError: vm_exit_code: {vm_exit_code:?}, response_raw: {response_raw}")]
    EmulatorEmulationError {
        vm_exit_code: Option<i32>,
        response_raw: String,
    },
    #[error("EmulatorPoolTimeout: timeout {0:.2?} reached")]
    EmulatorPoolTimeout(Duration),
    #[error("EmulatorMissingLibrary: missing library with hash {0}")]
    EmulatorMissingLibrary(TonHash),
    #[error("EmulatorTooManyLibraries: reach libraries limit ({0})")]
    EmulatorTooManyLibraries(usize),

    // TVMStack
    #[error("TVMStackError: fail to pop specified type. expected: {0}, got: {1}")]
    TVMStackWrongType(String, String),
    #[error("TVMStackError: stack is empty")]
    TVMStackEmpty,

    // Mnemonic
    #[error("MnemonicWordsCount: expected 24 words, got {0}")]
    MnemonicWordsCount(usize),
    #[error("MnemonicWord: unexpected word {0}")]
    MnemonicWord(String),
    #[error("MnemonicFirstByte: first byte can't be {0}")]
    MnemonicFirstByte(u8),
    #[error("MnemonicFirstBytePassless: first byte can't be {0}")]
    MnemonicFirstBytePassless(u8),

    // General errors
    #[error("UnexpectedValue: expected: {expected}, actual: {actual}")]
    UnexpectedValue { expected: String, actual: String },

    #[error("TonContractNotFull: contract {address} has no {missing_field} at tx_id {tx_id:?}")]
    TonContractNotFull {
        address: TonAddress,
        tx_id: Option<TxLTHash>,
        missing_field: String,
    },
    #[error("CustomError: {0}")]
    Custom(String),

    #[error("MetaLoaderError: {0}")]
    MetaLoaderError(#[from] MetaLoaderError),

    #[error("{0}")]
    HmacInvalidLen(#[from] crypto_common::InvalidLength),
    #[error("{0}")]
    NullError(#[from] std::ffi::NulError),
    #[error("{0}")]
    DecodeError(#[from] base64::DecodeError),
    #[error("{0}")]
    UTF8Error(#[from] std::str::Utf8Error),
    #[error("{0}")]
    FromHexError(#[from] hex::FromHexError),
    #[error("{0}")]
    ElapsedError(#[from] Elapsed),
    #[error("{0}")]
    AdnlError(#[from] adnl::AdnlError),

    #[error("{0}")]
    ParseInt(#[from] std::num::ParseIntError),
    #[error("{0}")]
    FromUtf8(#[from] std::string::FromUtf8Error),
    #[error("{0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("Transport error ({0})")]
    TransportError(#[from] reqwest::Error),
}

#[derive(Debug, Error)]
pub enum MetaLoaderError {
    #[error("Unsupported content layout (Metadata content: {0:?})")]
    ContentLayoutUnsupported(Box<MetadataContent>),

    #[error("Failed to load jetton metadata (URI: {uri}, response status code: {status})")]
    LoadMetadataFailed { uri: String, status: StatusCode },

    #[error("IpfsLoaderError path: {path}, status: {status}, msg: {msg}")]
    IpfsLoadError {
        path: String,
        status: StatusCode,
        msg: String,
    },
}

impl TonError {
    pub fn system<T: ToString>(msg: T) -> Self { TonError::SystemError(msg.to_string()) }
}

impl From<TonError> for TonCoreError {
    fn from(err: TonError) -> Self {
        match err {
            TonError::TLCoreError(err) => err,
            other => TonCoreError::Custom(other.to_string()),
        }
    }
}

impl From<&TonError> for TonCoreError {
    fn from(err: &TonError) -> Self { TonCoreError::Custom(err.to_string()) }
}

impl From<Arc<TonError>> for TonError {
    fn from(err: Arc<TonError>) -> Self { Self::ArcSelf(err) }
}
