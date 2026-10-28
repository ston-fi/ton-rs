use crate::sys_utils::sys_tonlib_set_verbosity_level;
use crate::tl_client::tl::{request::TLRequest, tonlibjson_wrapper::TonLibJsonWrapper};

#[test]
fn it_executes_functions() -> anyhow::Result<()> {
    sys_tonlib_set_verbosity_level(1);
    let client = TonLibJsonWrapper::new("test".to_string())?;
    client.send(&TLRequest::GetLogVerbosityLevel {}, "test2")?;
    Ok(())
}
