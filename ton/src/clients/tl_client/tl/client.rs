use crate::block_tlb::BlockIdExt;
use crate::clients::tl_client::connection::TLConnection;
use crate::clients::tl_client::tl::request::TLRequest;
use crate::clients::tl_client::tl::response::TLResponse;
use crate::clients::tl_client::tl::types::{
    TLAccountTxId, TLBlockId, TLBlocksHeader, TLBlocksMCInfo, TLBlocksShards, TLFullAccountState,
    TLRawFullAccountState, TLRawTxs, TLShortTxId, TLSmcLibraryEntry,
};
use crate::clients::tl_client::RetryStrategy;
use crate::errors::TonError;
use crate::unwrap_tl_rsp;
use async_trait::async_trait;
use tokio_retry::strategy::FixedInterval;
use tokio_retry::RetryIf;
use ton_lib_core::cell::TonHash;
use ton_lib_core::constants::{TON_MASTERCHAIN, TON_SHARD_FULL};
use ton_lib_core::types::{TonAddress, TxLTHash};

#[async_trait]
pub trait TLClientTrait: Send + Sync {
    fn get_connection(&self) -> &TLConnection;
    fn get_retry_strategy(&self) -> &RetryStrategy;

    async fn exec(&self, req: &TLRequest) -> Result<TLResponse, TonError> {
        let retry_strat = self.get_retry_strategy();
        let fi = FixedInterval::new(retry_strat.retry_waiting);
        let strategy = fi.take(retry_strat.retry_count);
        RetryIf::spawn(strategy, || self.get_connection().exec_impl(req), retry_condition).await
    }

    async fn get_mc_info(&self) -> Result<TLBlocksMCInfo, TonError> {
        let req = TLRequest::BlocksGetMCInfo {};
        unwrap_tl_rsp!(self.exec(&req).await?, TLBlocksMCInfo)
    }

    /// * `mode`: Lookup mode: `1` - by `block_id.seqno`, `2` - by `lt`, `4` - by `utime`.
    async fn lookup_block(&self, mode: i32, block_id: TLBlockId, lt: i64, utime: i32) -> Result<BlockIdExt, TonError> {
        let req = TLRequest::BlocksLookupBlock {
            mode,
            id: block_id,
            lt,
            utime,
        };
        unwrap_tl_rsp!(self.exec(&req).await?, TLBlockIdExt)
    }

    async fn lookup_mc_block(&self, seqno: u32) -> Result<BlockIdExt, TonError> {
        let block_id = TLBlockId {
            workchain: TON_MASTERCHAIN,
            shard: TON_SHARD_FULL as i64,
            seqno: seqno as i32,
        };
        self.lookup_block(1, block_id, 0, 0).await
    }

    async fn get_block_header(&self, block_id: BlockIdExt) -> Result<TLBlocksHeader, TonError> {
        let req = TLRequest::GetBlockHeader { id: block_id };
        unwrap_tl_rsp!(self.exec(&req).await?, TLBlocksHeader)
    }

    /// May return less libraries when requested
    /// Check it on user side if you need it
    async fn get_libs(&self, lib_ids: Vec<TonHash>) -> Result<Vec<TLSmcLibraryEntry>, TonError> {
        let req = TLRequest::SmcGetLibraries { library_list: lib_ids };
        let result = unwrap_tl_rsp!(self.exec(&req).await?, TLSmcLibraryResult)?;
        if result.result.is_empty() {
            return Ok(vec![]);
        }
        Ok(result.result)
    }

    async fn get_config_boc_param(&self, mode: u32, param: u32) -> Result<Vec<u8>, TonError> {
        let req = TLRequest::GetConfigParam { mode, param };
        Ok(unwrap_tl_rsp!(self.exec(&req).await?, TLConfigInfo)?.config.bytes)
    }

    // TODO find out about mode. Use 0 by default - it works well
    async fn get_config_boc_all(&self, mode: u32) -> Result<Vec<u8>, TonError> {
        let req = TLRequest::GetConfigAll { mode };
        Ok(unwrap_tl_rsp!(self.exec(&req).await?, TLConfigInfo)?.config.bytes)
    }

    async fn get_account_state(&self, address: TonAddress) -> Result<TLFullAccountState, TonError> {
        let req = TLRequest::GetAccountState {
            account_address: address.into(),
        };
        Ok(*unwrap_tl_rsp!(self.exec(&req).await?, TLFullAccountState)?)
    }

    async fn get_account_state_raw(&self, address: TonAddress) -> Result<TLRawFullAccountState, TonError> {
        let req = TLRequest::RawGetAccountState {
            account_address: address.into(),
        };
        unwrap_tl_rsp!(self.exec(&req).await?, TLRawFullAccountState)
    }

    async fn get_account_state_raw_by_tx(
        &self,
        address: TonAddress,
        tx_id: TxLTHash,
    ) -> Result<TLRawFullAccountState, TonError> {
        let req = TLRequest::RawGetAccountStateByTx {
            account_address: address.into(),
            tx_id,
        };
        unwrap_tl_rsp!(self.exec(&req).await?, TLRawFullAccountState)
    }

    async fn get_account_txs(&self, address: TonAddress, from_tx: TxLTHash) -> Result<TLRawTxs, TonError> {
        let req = TLRequest::RawGetTxs {
            account_address: address.into(),
            from_tx_id: from_tx,
        };
        unwrap_tl_rsp!(self.exec(&req).await?, TLRawTxs)
    }

    async fn get_account_txs_v2(
        &self,
        address: TonAddress,
        from_tx: TxLTHash,
        count: usize,
        try_decode_msg: bool,
    ) -> Result<TLRawTxs, TonError> {
        if count > 16 {
            return Err(TonError::TLWrongArgs(format!("get_raw_transactions_v2: count <= 16 supported, got {count}")));
        }
        let req = TLRequest::RawGetTxsV2 {
            account_address: address.into(),
            from_tx_id: from_tx,
            count: count as u32,
            try_decode_messages: try_decode_msg,
        };
        unwrap_tl_rsp!(self.exec(&req).await?, TLRawTxs)
    }

    async fn get_block_shards(&self, block_id: BlockIdExt) -> Result<TLBlocksShards, TonError> {
        let req = TLRequest::BlocksGetShards { id: block_id };
        unwrap_tl_rsp!(self.exec(&req).await?, TLBlocksShards)
    }

    async fn get_block_txs(&self, block_id: &BlockIdExt) -> Result<Vec<TLShortTxId>, TonError> {
        let mut after = TLAccountTxId {
            address_hash: TonHash::ZERO,
            lt: 0,
        };
        let mut incomplete = true;
        let mut txs = vec![];
        while incomplete {
            let mode = if after.lt == 0 { 7 } else { 7 + 128 };
            let req = TLRequest::BlocksGetTxs {
                id: block_id.clone(),
                mode,
                count: 256,
                after,
            };
            let response = unwrap_tl_rsp!(self.exec(&req).await?, TLBlocksTxs)?;
            if response.txs.is_empty() {
                break;
            }
            let last = &response.txs.last().unwrap();
            after = TLAccountTxId {
                address_hash: last.address_hash.clone(),
                lt: last.lt,
            };
            txs.extend(response.txs);
            incomplete = response.incomplete;
        }
        Ok(txs)
    }

    async fn send_msg(&self, body: Vec<u8>) -> Result<TonHash, TonError> {
        let req = TLRequest::RawSendMsgReturnHash { body };
        let rsp = unwrap_tl_rsp!(self.exec(&req).await?, TLRawExtMessageInfo)?;
        Ok(TonHash::from_vec(rsp.hash)?)
    }

    // TODO is not tested
    async fn sync(&self) -> Result<BlockIdExt, TonError> {
        let req = TLRequest::Sync {};
        unwrap_tl_rsp!(self.exec(&req).await?, TLBlockIdExt)
    }
}

fn retry_condition(error: &TonError) -> bool {
    match error {
        TonError::TLClientResponseError { code, .. } => *code == 500,
        _ => false,
    }
}
