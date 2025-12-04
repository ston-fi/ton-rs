#[macro_export]
macro_rules! unwrap_tl_rsp {
    ($result:expr, $variant:ident) => {
        match $result {
            TLResponse::$variant(inner) => Ok(inner),
            TLResponse::Error { code, msg } => Err(TonError::TLClientResponseError { code, msg }),
            _ => Err(TonError::TLClientWrongResponse(stringify!($variant).to_string(), format!("{:?}", $result))),
        }
    };
}
