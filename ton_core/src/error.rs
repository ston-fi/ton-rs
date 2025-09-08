use hex::FromHexError;
use std::env::VarError;
use std::sync::Arc;
use thiserror::Error;

#[macro_export]
macro_rules! bail_tl_core {
    ($($arg:tt)*) => {
        return Err(TLCoreError::Custom(format!($($arg)*)))
    };
}

#[derive(Error, Debug)]
pub enum TLCoreError {
    #[error("Failed to parse metadata")]
    MetadataParseError,
    // ton_hash
    #[error("TonHashWrongLen: Expecting {exp} bytes, got {given}")]
    TonHashWrongLen { exp: usize, given: usize },
    #[error("TonAddressParseError: address={0}, err: {1}")]
    TonAddressParseError(String, String),

    // cell_parser
    #[error("ParserDataUnderflow: Requested {req} bits, but only {left} left")]
    ParserDataUnderflow { req: usize, left: usize },
    #[error("ParserBadPosition: New position is {new_pos}, but data_bits_len is {bits_len}")]
    ParserBadPosition { new_pos: i32, bits_len: usize },
    #[error("ParserWrongSlicePosition: expecting bit_pos=0, next_ref_pos=0. Got bit_position={bit_pos}, next_ref_position={next_ref_pos}")]
    ParserWrongSlicePosition { bit_pos: usize, next_ref_pos: usize },
    #[error("ParserRefsUnderflow: No ref with index={req}")]
    ParserRefsUnderflow { req: usize },
    #[error("ParserCellNotEmpty: Cell is not empty: {bits_left} bits left, {refs_left} refs left")]
    ParserCellNotEmpty { bits_left: usize, refs_left: usize },

    // cell_builder
    #[error("BuilderDataOverflow: Can't write {req} bits: only {left} free bits available")]
    BuilderDataOverflow { req: usize, left: usize },
    #[error("BuilderRefsOverflow: Can't write ref - 4 refs are written already")]
    BuilderRefsOverflow,
    #[error("BuilderNotEnoughData: Can't extract {required_bits} bits from {given} bytes")]
    BuilderNotEnoughData { required_bits: usize, given: usize },
    #[error("BuilderNumberBitsMismatch: Can't write number {number} as {bits} bits")]
    BuilderNumberBitsMismatch { number: String, bits: usize },
    #[error("BuilderMeta: Cell validation error: {0}")]
    BuilderMeta(String),

    // boc
    #[error("BOCEmpty: can't parse BOC from empty slice")]
    BOCEmpty,
    #[error("BOCWrongData: {0}")]
    BOCWrongData(String),
    #[error("BOCSingleRoot: Expected 1 root, got {0}")]
    BOCSingleRoot(usize),

    // tlb
    #[error("TLBWrongData: {0}")]
    TLBWrongData(String),
    #[error("TLBWrongPrefix: expected={exp}, given={given}, exp_bits={bits_exp}, left_bits={bits_left}")]
    TLBWrongPrefix {
        exp: usize,
        given: usize,
        bits_exp: usize,
        bits_left: usize,
    },
    #[error("TLBEnumOutOfOptions: data doesn't match any variant of {0}")]
    TLBEnumOutOfOptions(String),
    #[error("TLBObjectNoValue: No internal value found (method: {0})")]
    TLBObjectNoValue(String),

    // contracts
    #[error("ContractError: {0}")]
    ContractError(String),

    // General errors
    #[error("Custom: {0}")]
    Custom(String),
    #[error("UnexpectedValue: expected: {expected}, actual: {actual}")]
    UnexpectedValue { expected: String, actual: String },

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
    ArcSelf(#[from] Arc<TLCoreError>),
}
