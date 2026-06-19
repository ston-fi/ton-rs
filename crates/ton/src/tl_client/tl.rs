use base64_serde::base64_serde_type;
base64_serde_type!(pub Base64Standard, base64::engine::general_purpose::STANDARD);

mod request;
mod request_ctx;
mod response;
mod ser_de;
mod tl_types;
pub(super) mod tonlibjson_wrapper;
mod unwrap_tl_rsp;

pub use request::*;
pub use request_ctx::*;
pub use response::*;
pub use tl_types::*;
