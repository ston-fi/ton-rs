use super::WalletVersion;

#[test]
fn test_wallet_version_serde_contract() -> anyhow::Result<()> {
    let version = WalletVersion::V4R2;
    let serialized = "\"V4R2\"";

    assert_eq!(serde_json::to_string(&version)?, serialized);
    assert_eq!(serde_json::from_str::<WalletVersion>(serialized)?, version);

    Ok(())
}
