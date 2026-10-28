use super::*;

#[test]
fn test_serde_ton_hash() -> anyhow::Result<()> {
    use serde_json::json;

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestStruct {
        #[serde(with = "serde_ton_hash_base64")]
        hash: TonHash,
        #[serde(with = "serde_ton_hash_vec_base64")]
        hash_vec: Vec<TonHash>,
        #[serde(with = "serde_ton_hash_hex_opt")]
        hash_opt: Option<TonHash>,
    }

    let val = TestStruct {
        hash: TonHash::from_slice(&[1u8; 32])?,
        hash_vec: vec![TonHash::from_slice(&[2u8; 32])?, TonHash::from_slice(&[3u8; 32])?],
        hash_opt: Some(TonHash::from_slice(&[3u8; 32])?),
    };
    let val_json_str = serde_json::to_string(&val)?;
    let val_json = serde_json::from_str::<serde_json::Value>(&val_json_str)?;

    let expected_json = json!({
        "hash": "AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE=",
        "hash_vec": [
            "AgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgI=",
            "AwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwM="
        ],
        "hash_opt": "0303030303030303030303030303030303030303030303030303030303030303",
    });
    assert_eq!(val_json, expected_json);
    let parsed_val = serde_json::from_str::<TestStruct>(&val_json.to_string())?;
    assert_eq!(parsed_val, val);
    Ok(())
}

#[test]
fn test_default_serde_ton_address_ton_hash() -> anyhow::Result<()> {
    use serde_json::json;

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestStruct {
        address: TonAddress,
        hash: TonHash,
        hash_opt: Option<TonHash>,
    }

    let address = TonAddress::from_str("EQCGScrZe1xbyWqWDvdI6mzP-GAcAWFv6ZXuaJOuSqemxku4")?;
    let hash = TonHash::from_str("16befdc4512ca3ffaa2919e1f0d7635588edcb9fa7d3990fe83e89275c291cc7")?;

    let val = TestStruct {
        address,
        hash,
        hash_opt: Some(hash),
    };

    let val_json_str = serde_json::to_string(&val)?;
    let val_json = serde_json::from_str::<serde_json::Value>(&val_json_str)?;
    let expected_json = json!({
        "address": "EQCGScrZe1xbyWqWDvdI6mzP-GAcAWFv6ZXuaJOuSqemxku4",
        "hash": "16befdc4512ca3ffaa2919e1f0d7635588edcb9fa7d3990fe83e89275c291cc7",
        "hash_opt": "16befdc4512ca3ffaa2919e1f0d7635588edcb9fa7d3990fe83e89275c291cc7",
    });
    assert_eq!(val_json, expected_json);

    let parsed_val = serde_json::from_str::<TestStruct>(&val_json.to_string())?;
    assert_eq!(parsed_val, val);

    let val_none = TestStruct {
        address,
        hash,
        hash_opt: None,
    };
    let val_none_json = serde_json::from_str::<serde_json::Value>(&serde_json::to_string(&val_none)?)?;
    assert_eq!(
        val_none_json,
        json!({
            "address": "EQCGScrZe1xbyWqWDvdI6mzP-GAcAWFv6ZXuaJOuSqemxku4",
            "hash": "16befdc4512ca3ffaa2919e1f0d7635588edcb9fa7d3990fe83e89275c291cc7",
            "hash_opt": null,
        })
    );
    let parsed_none = serde_json::from_str::<TestStruct>(&val_none_json.to_string())?;
    assert_eq!(parsed_none, val_none);
    Ok(())
}
