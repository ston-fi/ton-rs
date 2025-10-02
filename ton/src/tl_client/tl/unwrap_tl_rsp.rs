#[macro_export]
macro_rules! unwrap_tl_rsp {
    ($result:expr, $variant:ident) => {
        match $result {
            TLResponse::$variant(inner) => Ok(inner),
            TLResponse::Error { code, message } => Err(TonError::TLClientResponseError { code, message }),
            _ => Err(TonError::TLClientWrongResponse(stringify!($variant).to_string(), format!("{:?}", $result))),
        }
    };
}
