use super::*;
use tokio_test::assert_err;

fn make_data_error() -> Result<(), TonCoreError> {
    let val = "42";
    bail_ton_core_data!("some_error, val={val}");
}

#[test]
fn test_bail_ton_core_data() {
    let rs = make_data_error();
    let err = assert_err!(rs);
    assert_eq!(err.to_string(), "DataError: [ton_core::errors::tests] some_error, val=42");
}
