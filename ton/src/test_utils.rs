use crate::bail_ton;
use crate::block_tlb::Tx;
use crate::contracts::ContractClient;
use crate::contracts::tl_provider::TLProvider;
use crate::errors::TonResult;
use crate::tl_client::{TLClient, TLClientTrait};
use std::fs;
use std::ops::Deref;
use std::sync::Arc;
use ton_core::cell::TonHash;
use ton_core::traits::contract_provider::TonContractState;
use ton_core::traits::tlb::TLB;
use ton_core::types::{TonAddress, TxLTHash};

pub async fn load_cached_tx(
    cache_dir: &str,
    address: &TonAddress,
    lt: i64,
    hash: &TonHash,
    mainnet: bool,
) -> TonResult<Tx> {
    fs::create_dir_all(cache_dir)?;
    let cache_path = format!("{cache_dir}/tx_{hash}_{lt}.hex");
    let tx_id = TxLTHash::new(lt, hash.clone());
    if fs::metadata(&cache_path).is_err() {
        log::debug!("tx {tx_id} not found in cache, loading from network...");
        let tx = load_tx(address, &tx_id, mainnet).await?;
        fs::write(&cache_path, tx.to_boc_hex()?)?;
    }
    let bytes = fs::read_to_string(&cache_path)?;
    Ok(Tx::from_boc_hex(&bytes)?)
}

pub async fn load_cached_contract_state(
    cache_dir: &str,
    address: &TonAddress,
    lt: i64,
    hash: &TonHash,
    mainnet: bool,
) -> TonResult<Arc<TonContractState>> {
    fs::create_dir_all(cache_dir)?;
    let cache_path = format!("{cache_dir}/contract_state_{address}_{hash}_{lt}.json");
    let tx_id = TxLTHash::new(lt, hash.clone());
    if fs::metadata(cache_path.clone()).is_err() {
        log::debug!("state for {tx_id} not found in cache, loading from network...");
        let state = load_contract_state(address, &tx_id, mainnet).await?;
        fs::write(&cache_path, serde_json::to_string_pretty(state.deref())?)?;
    }
    let json = fs::read_to_string(cache_path.clone())?;
    let state = serde_json::from_str(&json)?;
    Ok(Arc::new(state))
}

async fn load_tx(address: &TonAddress, tx_id: &TxLTHash, mainnet: bool) -> TonResult<Tx> {
    let client = TLClient::builder()?.with_mainnet(mainnet).build().await?;
    let mut rsp = client.get_account_txs_v2(address.clone(), tx_id.clone(), 1, false).await?;
    if rsp.txs.is_empty() {
        bail_ton!("tx {tx_id} not found for account {address} with mainnet={mainnet}");
    }
    Ok(Tx::from_boc(rsp.txs.remove(0).data)?)
}

async fn load_contract_state(
    address: &TonAddress,
    tx_id: &TxLTHash,
    mainnet: bool,
) -> TonResult<Arc<TonContractState>> {
    let client = TLClient::builder()?.with_mainnet(mainnet).build().await?;
    let provider = TLProvider::new(client);
    let contract_client = ContractClient::builder(provider)?.build()?;
    contract_client.get_contract(address, Some(tx_id)).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use tokio_test::assert_ok;
    use ton_core::cell::TonHash;
    use ton_core::types::TonAddress;

    #[tokio::test]
    async fn test_load_cached_tx() -> anyhow::Result<()> {
        let cache_dir = "./resources/tests";
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
        let cache_dir = "./resources/tests";
        let address = TonAddress::from_str("EQCGScrZe1xbyWqWDvdI6mzP-GAcAWFv6ZXuaJOuSqemxku4")?;
        let lt = 64954068000009;
        let hash = TonHash::from_str("16befdc4512ca3ffaa2919e1f0d7635588edcb9fa7d3990fe83e89275c291cc7")?;
        let state = assert_ok!(load_cached_contract_state(cache_dir, &address, lt, &hash, true).await);
        assert_eq!(state.address, address);
        assert_eq!(state.last_tx_id, TxLTHash::new(lt, hash));
        Ok(())
    }
}
