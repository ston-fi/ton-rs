use crate::bail_ton;
use crate::block_tlb::Tx;
use crate::contracts::ContractClient;
use crate::contracts::tl_provider::TLProvider;
use crate::errors::TonResult;
use crate::tl_client::{TLClient, TLClientTrait};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
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
        fs::write(&cache_path, serde_json::to_string_pretty(&TonContractStateSerial::from(state.deref()))?)?;
    }
    let json = fs::read_to_string(cache_path.clone())?;
    let state: TonContractStateSerial = serde_json::from_str(&json)?;
    Ok(Arc::new(state.into()))
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

#[serde_as]
#[derive(Serialize, Deserialize)]
struct TonContractStateSerial {
    pub mc_seqno: Option<u32>,
    #[serde(with = "crate::ton_core::serde::serde_ton_address_base64_url")]
    pub address: TonAddress,
    #[serde(with = "crate::ton_core::serde::serde_tx_lt_hash_json")]
    pub last_tx_id: TxLTHash,
    #[serde_as(as = "Option<Arc<_>>")]
    pub code_boc: Option<Arc<Vec<u8>>>,
    #[serde_as(as = "Option<Arc<_>>")]
    pub data_boc: Option<Arc<Vec<u8>>>,
    #[serde(with = "crate::ton_core::serde::serde_ton_hash_hex_opt")]
    pub frozen_hash: Option<TonHash>,
    pub balance: i64,
}

impl From<&TonContractState> for TonContractStateSerial {
    fn from(state: &TonContractState) -> Self {
        TonContractStateSerial {
            mc_seqno: state.mc_seqno,
            address: state.address.clone(),
            last_tx_id: state.last_tx_id.clone(),
            code_boc: state.code_boc.clone(),
            data_boc: state.data_boc.clone(),
            frozen_hash: state.frozen_hash.clone(),
            balance: state.balance,
        }
    }
}

impl From<TonContractStateSerial> for TonContractState {
    fn from(serial: TonContractStateSerial) -> Self {
        TonContractState {
            mc_seqno: serial.mc_seqno,
            address: serial.address,
            last_tx_id: serial.last_tx_id,
            code_boc: serial.code_boc,
            data_boc: serial.data_boc,
            frozen_hash: serial.frozen_hash,
            balance: serial.balance,
        }
    }
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
