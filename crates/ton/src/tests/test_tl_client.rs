use super::LiteNodeFilter;

#[test]
fn test_lite_node_filter_serde_contract() -> anyhow::Result<()> {
    let cases = [
        (LiteNodeFilter::Healthy, "Healthy"),
        (LiteNodeFilter::Archive, "Archive"),
    ];

    for (filter, serialized) in cases {
        assert_eq!(serde_json::to_string(&filter)?, format!("\"{serialized}\""));
        assert_eq!(serde_json::from_str::<LiteNodeFilter>(&format!("\"{serialized}\""))?, filter);
    }

    Ok(())
}
