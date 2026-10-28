use crate::tl_client::tl::request::TLRequest;
use std::ffi::CString;

#[test]
fn test_tl_request_to_c_str_json() -> anyhow::Result<()> {
    let req = TLRequest::SetLogVerbosityLevel {
        new_verbosity_level: 100500,
    };
    let cstr: CString = req.to_c_str_json("some_extra")?;
    assert_eq!(
        "{\"@extra\":\"some_extra\",\"@type\":\"setLogVerbosityLevel\",\"new_verbosity_level\":100500}",
        cstr.to_str()?
    );
    Ok(())
}
