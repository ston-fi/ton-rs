use super::*;
use tokio_test::{assert_err, assert_ok};
use ton_core::cell::TonCell;

#[test]
fn test_write_up_to_4_msgs() -> anyhow::Result<()> {
    let mut builder = TonCell::builder();
    let msgs = vec![TonCell::empty().to_owned(), TonCell::empty().to_owned()];
    let msgs_modes = vec![1, 2];
    assert_ok!(write_up_to_4_msgs(&mut builder, &msgs, &msgs_modes));

    let mut builder = TonCell::builder();
    let msgs = vec![TonCell::empty().to_owned(); 4];
    let msgs_modes = vec![1, 2, 3, 4];
    assert_ok!(write_up_to_4_msgs(&mut builder, &msgs, &msgs_modes));

    let mut builder = TonCell::builder();
    let msgs = vec![TonCell::empty().to_owned(); 4];
    let msgs_modes = vec![1, 2, 3, 4, 5];
    assert_err!(write_up_to_4_msgs(&mut builder, &msgs, &msgs_modes));

    Ok(())
}
