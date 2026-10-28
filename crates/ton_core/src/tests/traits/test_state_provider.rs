use super::*;
use serde_json::json;
use std::str::FromStr;

#[test]
fn test_contract_state_json_roundtrip() -> anyhow::Result<()> {
    let address = TonAddress::from_str("EQCGScrZe1xbyWqWDvdI6mzP-GAcAWFv6ZXuaJOuSqemxku4")?;
    let last_hash = TonHash::from_str("16befdc4512ca3ffaa2919e1f0d7635588edcb9fa7d3990fe83e89275c291cc7")?;
    let frozen_hash = TonHash::from_slice(&[3; 32])?;
    let state = ContractState::new(address, TxLTHash::new(64_954_068_000_009, last_hash), 123)
        .with_mc_seqno(42)
        .with_code_boc(Arc::new(vec![1, 2]))
        .with_data_boc(Arc::new(vec![3, 4]))
        .with_frozen_hash(frozen_hash);

    let json = serde_json::to_value(&state)?;
    assert_eq!(
        json,
        json!({
            "mc_seqno": 42,
            "address": "EQCGScrZe1xbyWqWDvdI6mzP-GAcAWFv6ZXuaJOuSqemxku4",
            "last_tx_id": {
                "lt": "64954068000009",
                "hash": "Fr79xFEso/+qKRnh8NdjVYjty5+n05kP6D6JJ1wpHMc="
            },
            "code_boc": [1, 2],
            "data_boc": [3, 4],
            "frozen_hash": "0303030303030303030303030303030303030303030303030303030303030303",
            "balance": 123
        })
    );
    assert_eq!(serde_json::from_value::<ContractState>(json)?, state);
    Ok(())
}
