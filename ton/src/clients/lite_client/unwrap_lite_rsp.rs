#[macro_export]
macro_rules! unwrap_lite_rsp {
    ($result:expr, $variant:ident) => {
        match $result {
            Response::$variant(inner) => Ok(inner),
            Response::Error(err) => Err(TonError::LiteClientErrorResponse(err)),
            _ => Err(TonError::LiteClientWrongResponse(stringify!($variant).to_string(), format!("{:?}", $result))),
        }
    };
}
