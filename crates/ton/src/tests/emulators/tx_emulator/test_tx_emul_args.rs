use crate::emulators::tx_emulator::{TXEmulOrdArgs, create_test_tx_emul_ord_args};

#[test]
fn test_tx_emul_args_serialize() -> anyhow::Result<()> {
    let test_shard_acc = crate::emulators::tx_emulator::tests::TEST_SHARD_ACCOUNT.clone();
    let ord_args = create_test_tx_emul_ord_args(
        crate::emulators::tx_emulator::tests::TEST_MSG_IN_EXT.clone(),
        &test_shard_acc,
        &crate::emulators::tx_emulator::tests::BC_CONFIG,
        *crate::emulators::tx_emulator::tests::TEST_RAND_SEED,
        1738323935,
        53483578000001,
    )?;

    let dumped = serde_json::to_string_pretty(&ord_args)?;

    let loaded: TXEmulOrdArgs = serde_json::from_str(&dumped)?;
    assert_eq!(ord_args, loaded);

    Ok(())
}
