use crate::tests::utils::make_lite_client;
use everscale_types::boc::Boc;
use everscale_types::merkle::MerkleProof;
use everscale_types::models::ShardState;
use std::str::FromStr;
use tokio_test::{assert_err, assert_ok};
use ton::block_tlb::BlockIdExt;
use ton::errors::TonError;
use ton::unwrap_lite_rsp;
use ton_core::cell::TonCell;
use ton_core::cell::TonHash;
use ton_core::constants::TON_MASTERCHAIN;
use ton_core::traits::tlb::TLB;
use ton_core::types::TonAddress;
use ton_liteapi::tl::common::Int256;
use ton_liteapi::tl::request::{GetLibrariesWithProof, Request};
use ton_liteapi::tl::response::Response;

#[tokio::test]
async fn test_lite_client() -> anyhow::Result<()> {
    let lite_client = make_lite_client(true).await?;

    // generic interface
    let mc_info_rsp = lite_client.exec(Request::GetMasterchainInfo, None, None).await?;
    let mc_info_generic = unwrap_lite_rsp!(mc_info_rsp, MasterchainInfo)?;
    assert_ne!(mc_info_generic.last.seqno, 0);

    // === specialized interface ===
    let mc_info = lite_client.get_mc_info().await?;
    assert_ne!(mc_info_generic.last.seqno, 0);

    let block_id = lite_client.lookup_mc_block(mc_info.last.seqno).await?;
    assert_eq!(block_id, mc_info.last);

    let usdt_addr = TonAddress::from_str("EQCxE6mUtQJKFnGfaROTKOt1lZbDiiX1kCixRv7Nw2Id_sDs")?;
    let account = lite_client.get_account_state(&usdt_addr, mc_info.last.seqno, None).await?;
    assert!(account.as_account().is_some());

    // fetch zero state
    let system_addr = TonAddress::from_str("Ef8AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAADAU")?;
    let account = lite_client.get_account_state(&system_addr, 0, None).await?;
    assert!(account.as_account().is_some());
    assert_err!(lite_client.get_account_state(&usdt_addr, 0, None).await);
    Ok(())
}

#[tokio::test]
async fn test_library_deployers() -> anyhow::Result<()> {
    const LIBRARY_HASHES: [&str; 3] = [
        "A9338ECD624CA15D37E4A8D9BF677DDC9B84F0E98F05F2FB84C7AFE332A281B4",
        "C00836440D084E44FB94316132AC5A21417EF4F429EE09B5560B5678B334C3E8",
        "C95A2ED22AB516F77F9D4898DC4578E72F18A2448E8F6832334B0B4BF501BC79",
    ];

    let lite_client = make_lite_client(true).await?;
    let mc_info = lite_client.get_mc_info().await?;
    let library_hashes = LIBRARY_HASHES.iter().map(|hash| TonHash::from_str(hash)).collect::<Result<Vec<_>, _>>()?;
    let response = lite_client
        .exec(
            Request::GetLibrariesWithProof(GetLibrariesWithProof {
                id: mc_info.last.into(),
                mode: (),
                library_list: library_hashes.iter().map(|hash| Int256(*hash.as_slice_sized())).collect(),
            }),
            None,
            None,
        )
        .await?;
    let result = unwrap_lite_rsp!(response, LibraryResultWithProof)?;
    let proof = Boc::decode(result.data_proof)?.parse::<MerkleProof>()?;
    let state = proof.cell.as_ref().virtualize().parse::<ShardState>()?;
    let ShardState::Unsplit(state) = state else {
        anyhow::bail!("expected an unsplit masterchain state");
    };

    for library_hash in library_hashes {
        let Some(library) = state.libraries.get(*library_hash.as_slice_sized())? else {
            anyhow::bail!("library {library_hash} not found");
        };

        for publisher in library.publishers.keys() {
            let publisher = publisher?;
            let deployer = TonAddress::new(TON_MASTERCHAIN, TonHash::from_slice_sized(publisher.as_array()));
            println!("{library_hash}: {deployer}");
        }
    }

    Ok(())
}

#[ignore = "requires full (archive) testnet node"]
#[tokio::test]
async fn test_lite_client_testnet() -> anyhow::Result<()> {
    let lite_client = make_lite_client(false).await?;
    let mc_info = lite_client.get_mc_info().await?;
    let usdt_addr = TonAddress::from_str("kQD4HpyO8ilPHHUV4CpiHMqz8F2eWyVOMH10MxTYrY3Emvmu")?;
    let account = lite_client.get_account_state(&usdt_addr, mc_info.last.seqno, None).await?;
    assert!(account.as_account().is_some());

    // fetching zero-block
    let state = lite_client.get_block_state(BlockIdExt::ZERO_BLOCK_ID_TESTNET, None).await?;
    assert_ok!(TonCell::from_boc(state.data));
    Ok(())
}
