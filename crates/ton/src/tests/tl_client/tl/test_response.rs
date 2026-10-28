use super::*;

use std::ffi::CString;

#[test]
fn test_tl_response_from_c_str_json() -> anyhow::Result<()> {
    let cstr = CString::new("{\"@extra\":\"some_extra\",\"@type\":\"logVerbosityLevel\",\"verbosity_level\":100500}")?;
    let (rsp, extra) = unsafe { TLResponse::from_c_str_json(cstr.as_ptr())? };
    std::mem::forget(cstr); // pointer is free during parsing
    let expected = Some(String::from("some_extra"));
    assert_eq!(extra, expected);
    assert!(matches!(rsp, TLResponse::TLLogVerbosityLevel(_)));
    Ok(())
}
