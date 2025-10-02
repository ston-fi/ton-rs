use std::str::FromStr;
use tokio_test::assert_ok;
use ton_lib::block_tlb::{BlockIdExt, ShardIdent};
use ton_lib::tl_client::tl::client::TLClientTrait;

use crate::tests::utils::make_tl_client;
use ton_lib::tl_client::tl::tl_types::TLAccountState;
use ton_lib::tl_client::TLClient;
use ton_lib_core::cell::{TonCell, TonHash};
use ton_lib_core::traits::tlb::TLB;
use ton_lib_core::types::TonAddress;

#[tokio::test]
async fn test_tl_client_default() -> anyhow::Result<()> {
    let client = make_tl_client(true, true).await?;
    let mc_info = client.get_mc_info().await?;
    assert_ne!(mc_info.last.seqno, 0);

    // tl_client methods
    assert_tl_client_lookup_mc_block(&client, mc_info.last.seqno - 100).await?; // another node may be behind
    assert_tl_client_get_block_header(&client, &mc_info.last).await?;
    assert_tl_client_get_config(&client).await?;
    assert_tl_client_get_libs(&client).await?;
    assert_tl_client_get_account_state(&client).await?;
    assert_tl_client_get_account_txs(&client).await?;
    assert_tl_client_get_block_txs(&client).await?;

    Ok(())
}

#[tokio::test]
async fn test_tl_client_default_async_context() -> anyhow::Result<()> {
    let tl_client = make_tl_client(true, true).await?;
    let res = async { tl_client.get_mc_info().await }.await?;
    assert_ne!(res.last.seqno, 0);
    Ok(())
}

async fn assert_tl_client_lookup_mc_block(client: &TLClient, seqno: u32) -> anyhow::Result<()> {
    let block = client.lookup_mc_block(seqno).await?;
    assert_eq!(block.seqno, seqno);
    Ok(())
}

async fn assert_tl_client_get_block_header(client: &TLClient, block_id: &BlockIdExt) -> anyhow::Result<()> {
    let header = client.get_block_header(block_id.clone()).await?;
    assert_eq!(&header.id, block_id);
    Ok(())
}

async fn assert_tl_client_get_config(client: &TLClient) -> anyhow::Result<()> {
    let config = client.get_config_boc_all(0).await?;
    assert_ok!(TonCell::from_boc(&config));

    let config = client.get_config_boc_param(0, 34).await?;
    let cell = assert_ok!(TonCell::from_boc(&config));
    let mut parser = cell.parser();
    let value: u8 = TLB::read(&mut parser)?;
    assert_eq!(value, 0x12);
    Ok(())
}

async fn assert_tl_client_get_libs(client: &TLClient) -> anyhow::Result<()> {
    // https://tonviewer.com/EQCGScrZe1xbyWqWDvdI6mzP-GAcAWFv6ZXuaJOuSqemxku4
    let lib_id = TonHash::from_str("A9338ECD624CA15D37E4A8D9BF677DDC9B84F0E98F05F2FB84C7AFE332A281B4")?;
    let libs = client.get_libs(vec![lib_id.clone()]).await?;
    assert_eq!(libs.len(), 1);
    assert_eq!(libs[0].hash, lib_id.to_vec());
    Ok(())
}

async fn assert_tl_client_get_account_state(client: &TLClient) -> anyhow::Result<()> {
    let expected_code = TonCell::from_boc_hex("b5ee9c72010218010005bb000114ff00f4a413f4bcf2c80b0102016202030202cb0405020120141502f3d0cb434c0c05c6c238ecc200835c874c7c0608405e351466ea44c38601035c87e800c3b51343e803e903e90353534541168504d3214017e809400f3c58073c5b333327b55383e903e900c7e800c7d007e800c7e80004c5c3e0e80b4c7c04074cfc044bb51343e803e903e9035353449a084190adf41eeb8c089a0607001da23864658380e78b64814183fa0bc0019635355161c705f2e04904fa4021fa4430c000f2e14dfa00d4d120d0d31f018210178d4519baf2e0488040d721fa00fa4031fa4031fa0020d70b009ad74bc00101c001b0f2b19130e254431b0803fa82107bdd97deba8ee7363805fa00fa40f82854120a70546004131503c8cb0358fa0201cf1601cf16c921c8cb0113f40012f400cb00c9f9007074c8cb02ca07cbffc9d05008c705f2e04a12a14414506603c85005fa025003cf1601cf16ccccc9ed54fa40d120d70b01c000b3915be30de02682102c76b973bae30235250a0b0c018e2191729171e2f839206e938124279120e2216e94318128739101e25023a813a0738103a370f83ca00270f83612a00170f836a07381040982100966018070f837a0bcf2b025597f0900ec82103b9aca0070fb02f828450470546004131503c8cb0358fa0201cf1601cf16c921c8cb0113f40012f400cb00c920f9007074c8cb02ca07cbffc9d0c8801801cb0501cf1658fa02029858775003cb6bcccc9730017158cb6acce2c98011fb005005a04314c85005fa025003cf1601cf16ccccc9ed540044c8801001cb0501cf1670fa027001cb6a8210d53276db01cb1f0101cb3fc98042fb0001fc145f04323401fa40d2000101d195c821cf16c9916de2c8801001cb055004cf1670fa027001cb6a8210d173540001cb1f500401cb3f23fa4430c0008e35f828440470546004131503c8cb0358fa0201cf1601cf16c921c8cb0113f40012f400cb00c9f9007074c8cb02ca07cbffc9d012cf1697316c127001cb01e2f400c90d04f882106501f354ba8e223134365145c705f2e04902fa40d1103402c85005fa025003cf1601cf16ccccc9ed54e0258210fb88e119ba8e2132343603d15131c705f2e0498b025512c85005fa025003cf1601cf16ccccc9ed54e034248210235caf52bae30237238210cb862902bae302365b2082102508d66abae3026c310e0f101100088050fb0002ec3031325033c705f2e049fa40fa00d4d120d0d31f01018040d7212182100f8a7ea5ba8e4d36208210595f07bcba8e2c3004fa0031fa4031f401d120f839206e943081169fde718102f270f8380170f836a0811a7770f836a0bcf2b08e138210eed236d3ba9504d30331d19434f2c048e2e2e30d50037012130044335142c705f2e049c85003cf16c9134440c85005fa025003cf1601cf16ccccc9ed54001e3002c705f2e049d4d4d101ed54fb0400188210d372158cbadc840ff2f000ce31fa0031fa4031fa4031f401fa0020d70b009ad74bc00101c001b0f2b19130e25442162191729171e2f839206e938124279120e2216e94318128739101e25023a813a0738103a370f83ca00270f83612a00170f836a07381040982100966018070f837a0bcf2b000c082103b9aca0070fb02f828450470546004131503c8cb0358fa0201cf1601cf16c921c8cb0113f40012f400cb00c920f9007074c8cb02ca07cbffc9d0c8801801cb0501cf1658fa02029858775003cb6bcccc9730017158cb6acce2c98011fb000025bd9adf6a2687d007d207d206a6a6888122f82402027116170085adbcf6a2687d007d207d206a6a688a2f827c1400b82a3002098a81e46581ac7d0100e78b00e78b6490e4658089fa00097a00658064fc80383a6465816503e5ffe4e84000cfaf16f6a2687d007d207d206a6a68bf99e836c1783872ebdb514d9c97c283b7f0ae5179029e2b6119c39462719e4f46ed8f7413e62c780a417877407e978f01a40711411b1acb773a96bdd93fa83bb5ca8435013c8c4b3ac91f4589b4780a38646583fa0064a18040")?;
    let usdt_master = TonAddress::from_str("EQCxE6mUtQJKFnGfaROTKOt1lZbDiiX1kCixRv7Nw2Id_sDs")?;

    let usdt_state = client.get_account_state(usdt_master.clone()).await?;
    let TLAccountState::Raw { code, .. } = usdt_state.account_state else {
        panic!("Expected Raw account state");
    };
    assert_eq!(TonCell::from_boc(&code)?, expected_code);

    let usdt_state_raw = client.get_account_state_raw(usdt_master.clone()).await?;
    assert_eq!(TonCell::from_boc(&usdt_state_raw.code)?, expected_code);

    let mut usdt_by_tx =
        client.get_account_state_raw_by_tx(usdt_master.clone(), usdt_state_raw.last_tx_id.clone()).await?;
    // these field doesn't relate to the state
    usdt_by_tx.sync_utime = usdt_state_raw.sync_utime;
    usdt_by_tx.block_id = usdt_state_raw.block_id.clone();
    assert_eq!(usdt_state_raw, usdt_by_tx);
    Ok(())
}

async fn assert_tl_client_get_account_txs(client: &TLClient) -> anyhow::Result<()> {
    let usdt_master = TonAddress::from_str("EQCxE6mUtQJKFnGfaROTKOt1lZbDiiX1kCixRv7Nw2Id_sDs")?;
    let usdt_state_raw = client.get_account_state_raw(usdt_master.clone()).await?;

    let raw_txs = client.get_account_txs(usdt_master.clone(), usdt_state_raw.last_tx_id.clone()).await?;
    assert!(!raw_txs.txs.is_empty());
    assert_eq!(raw_txs.txs[0].tx_id, usdt_state_raw.last_tx_id);

    let raw_txs_v2 = client.get_account_txs_v2(usdt_master, usdt_state_raw.last_tx_id.clone(), 1, false).await?;
    assert!(!raw_txs_v2.txs.is_empty());
    assert_eq!(raw_txs_v2.txs[0].tx_id, usdt_state_raw.last_tx_id);
    Ok(())
}

async fn assert_tl_client_get_block_txs(client: &TLClient) -> anyhow::Result<()> {
    // https://ton.cx/block/-1:8000000000000000:48930047
    let mc_block_id = BlockIdExt {
        shard_ident: ShardIdent {
            workchain: -1,
            shard: 0x8000000000000000,
        },
        seqno: 48930047,
        root_hash: TonHash::from_str("rgdhCx1ihw8nbyWwm5iYXhHQJS/SHGZwMOt6XekblSI=")?,
        file_hash: TonHash::from_str("ycxyfwlJ3LjgU2kWsXi+h7C/eeSrNRnIVtrjqXW5UHU=")?,
    };
    let tx_ids = client.get_block_txs(&mc_block_id).await?;
    assert_eq!(tx_ids.len(), 3);
    assert_eq!(tx_ids[0].lt, 58411111000001);
    assert_eq!(tx_ids[1].lt, 58411111000002);
    assert_eq!(tx_ids[2].lt, 58411111000003);

    // https://ton.cx/block/0:6000000000000000:54144203
    let shard_block_id = BlockIdExt {
        shard_ident: ShardIdent {
            workchain: 0,
            shard: 0x2000000000000000,
        },
        seqno: 54144203,
        root_hash: TonHash::from_str("PbfpKHGtlV3K59zVBvaZojCVHCKQNnivn6z3Wk5/cng=")?,
        file_hash: TonHash::from_str("a7KbIqKMR9bE83euGZG0kgIcx1BeBLP04CVenkj4v4E=")?,
    };
    let tx_ids = client.get_block_txs(&shard_block_id).await?;
    assert_eq!(tx_ids.len(), 16);
    assert_eq!(tx_ids[0].lt, 58424288000001);
    assert_eq!(
        TonAddress::new(0, tx_ids[0].address_hash.clone()),
        TonAddress::from_str("EQAJkSJl2NMvTBiXW9Yyq5yjLZxs1mS06sbNycl9MMGekmk_")?
    );
    assert_eq!(tx_ids[0].tx_hash, TonHash::from_str("g+UtOuTdzlYXuAVndQ0UMV3jA5amwovuJ/ovGV4ykzg=")?);

    // order doesn't match with ton.cx view ¯\_(ツ)_/¯
    assert_eq!(tx_ids[15].lt, 58424288000001);
    assert_eq!(
        TonAddress::new(0, tx_ids[15].address_hash.clone()),
        TonAddress::from_str("EQA4xAXkdxtX6dG3E7EAG704jm_2tK8dOTit-6e0WwEKdo6X")?
    );
    assert_eq!(tx_ids[15].tx_hash, TonHash::from_str("zer/c8RprjxJ80pQkEeQHBEAW+qr9dDSilEkGTTZ8iI=")?);

    // TODO would be nice to find a block with more than 256 txs to check inner loop
    Ok(())
}
