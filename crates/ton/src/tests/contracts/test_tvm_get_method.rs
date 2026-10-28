use super::TVMGetMethodID;

#[test]
fn test_hex_format() -> anyhow::Result<()> {
    let method_id: TVMGetMethodID = 0x1234beef.into();
    let s = format!("{method_id}");
    assert_eq!(s, "#1234beef");
    Ok(())
}

#[test]
fn test_opcode() -> anyhow::Result<()> {
    let p = "transfer query_id:uint64 amount:VarUInteger 16 destination:MsgAddress \
    response_destination:MsgAddress custom_payload:Maybe ^Cell forward_ton_amount:VarUInteger 16 \
    forward_payload:Either Cell ^Cell = InternalMsgBody";
    let method_id: TVMGetMethodID = TVMGetMethodID::from_prototype(p);
    assert_eq!(method_id, TVMGetMethodID::Number(0x0f8a7ea5));
    Ok(())
}

#[test]
fn test_serde_contract() -> anyhow::Result<()> {
    let number = TVMGetMethodID::Number(0x1234);
    assert_eq!(serde_json::to_string(&number)?, "4660");
    assert_eq!(serde_json::from_str::<TVMGetMethodID>("4660")?, number);

    let name = TVMGetMethodID::from("get_wallet_data");
    assert_eq!(serde_json::to_string(&name)?, "\"get_wallet_data\"");
    assert_eq!(serde_json::from_str::<TVMGetMethodID>("\"get_wallet_data\"")?, name);

    Ok(())
}
