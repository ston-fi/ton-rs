use crate::tests::utils::make_tl_client;
use futures_util::try_join;
use std::str::FromStr;
use tokio_test::assert_ok;
use ton::contracts::tl_provider::TLProvider;
use ton::contracts::{
    ContractClient, JettonMasterContract, JettonMasterMethods, JettonWalletContract, JettonWalletMethods, TonContract,
};
use ton::tl_client::TLClient;
use ton_core::cell::TonHash;
use ton_core::traits::contract_provider::{TonContractState, TonProvider};
use ton_core::types::{TonAddress, TxLTHash};

#[tokio::test]
async fn test_contract_client() -> anyhow::Result<()> {
    let tl_client = make_tl_client(true, true).await?;

    #[rustfmt::skip]
    let res = try_join!(
        assert_tl_provider_works(tl_client.clone()),
        assert_contract_client_tl_provider(tl_client.clone()),
    );
    assert_ok!(res);
    Ok(())
}

async fn assert_tl_provider_works(tl_client: TLClient) -> anyhow::Result<()> {
    let tl_provider = TLProvider::new(tl_client);

    let usdt_master = TonAddress::from_str("EQCxE6mUtQJKFnGfaROTKOt1lZbDiiX1kCixRv7Nw2Id_sDs")?;

    let last_seqno = tl_provider.last_mc_seqno().await?;
    assert_ne!(last_seqno, 0);

    let latest_state = tl_provider.load_state(usdt_master.clone(), None).await?;
    assert_eq!(latest_state.address, usdt_master);

    let state_by_tx = tl_provider.load_state(usdt_master.clone(), Some(latest_state.last_tx_id.clone())).await?;
    assert_eq!(state_by_tx, latest_state);

    let bc_config = tl_provider.load_bc_config(None).await?;
    assert!(!bc_config.is_empty());

    let lib_id = TonHash::from_str("A9338ECD624CA15D37E4A8D9BF677DDC9B84F0E98F05F2FB84C7AFE332A281B4")?;
    let libs = tl_provider.load_libs(vec![lib_id.clone()], None).await?;
    assert_eq!(libs.len(), 1);
    assert_eq!(libs[0].0, lib_id);

    let latest_txs_per_address = tl_provider.load_latest_tx_per_address(50140309).await?;
    assert_eq!(latest_txs_per_address.len(), 87);

    for (address, tx_id) in [
        (
            // some random address with few txs
            TonAddress::from_str("EQBF0nJnIPRNlEtpLUBcfah2b0I7Xf09sGDk7EDZeafhBL1o")?,
            TxLTHash::from_str("59686385000060:964d5e59d55e99669306b8e3223fed8cc3b5b3440c7005de1276fe0f0be8a644")?,
        ),
        (
            // some random address with few txs
            TonAddress::from_str("EQBrTU_6DhGDkQejzdVetYpMouyyjYKg47vOBKfnkiTNXQAs")?,
            TxLTHash::from_str("59686385000044:6e16887202c3c4e05f989a49d1a1786a73d5d440fc86ff515d5cd4bc075b69b4")?,
        ),
        (
            // some random address with few txs
            TonAddress::from_str("EQCHpmLKmQAOgKwrr-O2vkdRvr0Sq-ztnu4-XhoaQfUmSl4A")?,
            TxLTHash::from_str("59686385000046:bf92f00671be16ba7a755c8ca0f8d136c727bd305edff6e64b8bcc2572bc3dee")?,
        ),
        (
            // contains only 1 tx
            TonAddress::from_str("EQCU7X49nR0dBxuuy1IHxxAFMgoMySoZpOlHlwh4vLY1FWrY")?,
            TxLTHash::from_str("59686385000028:3125d7ae7f3a107d629f3a87890730c15d1699561e5fb9003d9faebebd67c1ef")?,
        ),
        (
            // masterchain account
            TonAddress::from_str("Ef8zMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzM0vF")?,
            TxLTHash::from_str("59686387000002:46a13b55bcff63a27903c657eb852e2817bffe7295eda0a6e6e592934810dfe7")?,
        ),
    ] {
        assert!(latest_txs_per_address.contains(&(address, tx_id)));
    }

    Ok(())
}

async fn assert_contract_client_tl_provider(tl_client: TLClient) -> anyhow::Result<()> {
    let ctr_cli = ContractClient::builder(TLProvider::new(tl_client)).with_default_caches().build()?;

    let usdt_master = TonAddress::from_str("EQCxE6mUtQJKFnGfaROTKOt1lZbDiiX1kCixRv7Nw2Id_sDs")?;

    assert_eq!(ctr_cli.cache_stats().get("state_latest_req").copied(), Some(0));
    assert_eq!(ctr_cli.cache_stats().get("state_latest_miss").copied(), Some(0));
    let _contract = JettonMasterContract::new(&ctr_cli, &usdt_master, None).await?;
    assert_eq!(ctr_cli.cache_stats().get("state_latest_req").copied(), Some(1));
    assert_eq!(ctr_cli.cache_stats().get("state_latest_miss").copied(), Some(1));

    let tx_id = TxLTHash::new(
        59663842000027,
        TonHash::from_str("7d90294122887b3ee8c3ee534eaf2d62533445dff4646ad9c9dbd05ab404baaf")?,
    );
    let _contract = JettonMasterContract::new(&ctr_cli, &usdt_master, Some(tx_id.clone())).await?;
    assert_eq!(ctr_cli.cache_stats().get("state_latest_req").copied(), Some(1));
    assert_eq!(ctr_cli.cache_stats().get("state_latest_miss").copied(), Some(1));
    assert_eq!(ctr_cli.cache_stats().get("state_by_tx_req").copied(), Some(1));
    assert_eq!(ctr_cli.cache_stats().get("state_by_tx_miss").copied(), Some(1));

    let _contract = JettonMasterContract::new(&ctr_cli, &usdt_master, Some(tx_id.clone())).await?;
    assert_eq!(ctr_cli.cache_stats().get("state_latest_req").copied(), Some(1));
    assert_eq!(ctr_cli.cache_stats().get("state_latest_miss").copied(), Some(1));
    assert_eq!(ctr_cli.cache_stats().get("state_by_tx_req").copied(), Some(2));
    assert_eq!(ctr_cli.cache_stats().get("state_by_tx_miss").copied(), Some(1));
    Ok(())
}

#[tokio::test]
async fn test_contract_client_tl_provider_dynamic_libs_testnet() -> anyhow::Result<()> {
    let tl_client = make_tl_client(false, true).await?;

    let ctr_cli = ContractClient::builder(TLProvider::new(tl_client)).with_default_caches().build()?;
    let dyn_lib_master_addr = TonAddress::from_str("kQAjmiNekXMED_a-Ps7whmYgfdT32Z9_kIEzk5F_Bnh-jTFb")?;
    let dyn_lib_wallet_addr = TonAddress::from_str("kQAsm4uCgpdK5B7msqcd4Pe27C6IakdFsxGwygkgkX-kC56Q")?;

    // test master
    let master_ctr = JettonMasterContract::new(&ctr_cli, &dyn_lib_master_addr, None).await?;
    let jetton_data = master_ctr.get_jetton_data().await?;
    assert_eq!(
        jetton_data.admin,
        TonAddress::from_str("0:476cbaf6ab9fe4f72c328e5053caeed6919ca0edae2d075c18fd445335b8d04c")?
    );

    // test wallet
    let wallet_ctr = JettonWalletContract::new(&ctr_cli, &dyn_lib_wallet_addr, None).await?;
    let wallet_data = wallet_ctr.get_wallet_data().await?;
    assert_eq!(
        wallet_data.owner,
        TonAddress::from_str("0:2cf3b5b8c891e517c9addbda1c0386a09ccacbb0e3faf630b51cfc8152325acb")?
    );
    Ok(())
}
