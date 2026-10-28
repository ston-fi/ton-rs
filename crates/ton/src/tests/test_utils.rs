use super::*;
use std::str::FromStr;
use tokio_test::assert_ok;
use ton_core::cell::TonHash;
use ton_core::types::TonAddress;

#[tokio::test]
async fn test_load_cached_tx() -> anyhow::Result<()> {
    let cache_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/resources/tests");
    let address = TonAddress::from_str("EQCGScrZe1xbyWqWDvdI6mzP-GAcAWFv6ZXuaJOuSqemxku4")?;
    let lt = 64954068000009;
    let hash = TonHash::from_str("16befdc4512ca3ffaa2919e1f0d7635588edcb9fa7d3990fe83e89275c291cc7")?;
    let tx = assert_ok!(load_cached_tx(cache_dir, &address, lt, &hash, true).await);
    assert_eq!(tx.lt, lt as u64);
    assert_eq!(tx.cell_hash()?, hash);
    Ok(())
}

#[tokio::test]
async fn test_load_cached_contract_state() -> anyhow::Result<()> {
    let cache_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/resources/tests");
    let address = TonAddress::from_str("EQCGScrZe1xbyWqWDvdI6mzP-GAcAWFv6ZXuaJOuSqemxku4")?;
    let lt = 64954068000009;
    let hash = TonHash::from_str("16befdc4512ca3ffaa2919e1f0d7635588edcb9fa7d3990fe83e89275c291cc7")?;
    let state = assert_ok!(load_cached_contract_state(cache_dir, &address, lt, &hash, true).await);
    assert_eq!(state.address, address);
    assert_eq!(state.last_tx_id, TxLTHash::new(lt, hash));
    Ok(())
}
