use super::*;
use std::str::FromStr;
use tokio_test::assert_err;

#[test]
fn test_tx_lt_hash_from_str() -> anyhow::Result<()> {
    let tx_lt_hash = TxLTHash::from_str("12345:abcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcd")?;
    assert_eq!(tx_lt_hash.lt, 12345);
    assert_eq!(tx_lt_hash.hash, TonHash::from_str("abcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcd")?);

    assert_err!(TxLTHash::from_str("123"));
    assert_err!(TxLTHash::from_str("xxx:123"));
    assert_err!(TxLTHash::from_str("123:zzz"));
    Ok(())
}
