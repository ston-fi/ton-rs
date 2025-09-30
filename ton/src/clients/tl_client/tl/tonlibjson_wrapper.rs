use crate::clients::tl_client::tl::request::TLRequest;
use crate::clients::tl_client::tl::response::TLResponse;
use crate::errors::TonError;
use tonlib_sys::*;
// Wrapper around ton client with support for TL data types

pub(crate) struct TonLibJsonWrapper {
    ptr: *mut std::os::raw::c_void,
    tag: String,
}

impl TonLibJsonWrapper {
    pub fn new(tag: String) -> Result<TonLibJsonWrapper, TonError> {
        let client_ptr = unsafe { tonlib_client_json_create() };
        if client_ptr.is_null() {
            return Err(TonError::TLClientCreationFailed);
        }
        Ok(TonLibJsonWrapper { ptr: client_ptr, tag })
    }

    pub fn tag(&self) -> &str { self.tag.as_str() }

    pub fn send(&self, req: &TLRequest, extra: &str) -> Result<(), TonError> {
        let c_str = req.to_c_str_json(extra)?;
        log::trace!("[{}] send: {c_str:?}", self.tag);
        unsafe { tonlib_client_json_send(self.ptr, c_str.as_ptr()) };
        Ok(())
    }

    pub fn receive(&self, timeout: f64) -> Option<Result<(TLResponse, Option<String>), TonError>> {
        let c_str = unsafe { tonlib_client_json_receive(self.ptr, timeout) };
        if c_str.is_null() {
            return None;
        }
        unsafe { Some(TLResponse::from_c_str_json(c_str)) }
    }
}

impl Drop for TonLibJsonWrapper {
    fn drop(&mut self) { unsafe { tonlib_client_json_destroy(self.ptr) } }
}

unsafe impl Send for TonLibJsonWrapper {}
unsafe impl Sync for TonLibJsonWrapper {}

#[cfg(test)]
mod tests {
    use crate::clients::tl_client::tl::{request::TLRequest, tonlibjson_wrapper::TonLibJsonWrapper};
    use crate::sys_utils::sys_tonlib_set_verbosity_level;

    #[test]
    fn it_executes_functions() -> anyhow::Result<()> {
        sys_tonlib_set_verbosity_level(1);
        let client = TonLibJsonWrapper::new("test".to_string())?;
        client.send(&TLRequest::GetLogVerbosityLevel {}, "test2")?;
        Ok(())
    }
}
