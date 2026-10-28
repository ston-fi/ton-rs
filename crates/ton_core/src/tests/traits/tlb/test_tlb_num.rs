use super::*;
use fastnum::{i256, i512, u256, u512};
use tokio_test::assert_ok;
#[test]
fn test_tlb_num() -> anyhow::Result<()> {
    assert_ok!((-1i8).to_cell());
    assert_ok!(1u8.to_cell());
    assert_ok!((-32i32).to_cell());
    assert_ok!((32u32).to_cell());
    assert_ok!(i256!(-123).to_cell());
    assert_ok!(u256!(123).to_cell());
    assert_ok!(i512!(234).to_cell());
    assert_ok!(u512!(234).to_cell());
    Ok(())
}
