mod builder;
mod connection;
mod lite_types;
mod liteapi_serde;
mod req_params;
mod unwrap_lite_rsp;

pub use builder::*;
pub use lite_types::*;
pub use req_params::*;

use crate::block_tlb::{BlockIdExt, MaybeAccount};
use crate::errors::TonError;
use crate::libs_dict::LibsDict;
use crate::lite_client::connection::Connection;
use crate::unwrap_lite_rsp;
use auto_pool::pool::AutoPool;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering::Relaxed;
use std::time::Duration;
use tokio_retry::RetryIf;
use tokio_retry::strategy::FixedInterval;
use ton_core::cell::{TonCell, TonHash};
use ton_core::constants::{TON_MASTERCHAIN, TON_SHARD_FULL};
use ton_core::errors::{TonCoreError, TonCoreResult};
use ton_core::traits::tlb::TLB;
use ton_core::types::TonAddress;
use ton_liteapi::tl::common::{AccountId, Int256};
use ton_liteapi::tl::request::*;
use ton_liteapi::tl::response::{BlockData, Response};

const WAIT_MC_SEQNO_MS: u32 = 5000;
const WAIT_CONNECTION_MS: u64 = 5;

#[derive(Clone)]
pub struct LiteClient(Arc<Inner>);

// converts ton_block -> ton_liteapi objects under the hood
impl LiteClient {
    pub fn builder() -> TonCoreResult<Builder> { Builder::new() }

    pub async fn get_mc_info(&self) -> Result<MasterchainInfo, TonError> {
        let rsp = self.exec(Request::GetMasterchainInfo, None, None).await?;
        let mc_info = unwrap_lite_rsp!(rsp, MasterchainInfo)?;
        Ok(mc_info.into())
    }

    pub async fn lookup_mc_block(&self, seqno: u32) -> Result<BlockIdExt, TonError> {
        self.lookup_block(TON_MASTERCHAIN, TON_SHARD_FULL, seqno).await
    }

    pub async fn lookup_block(&self, wc: i32, shard: u64, seqno: u32) -> Result<BlockIdExt, TonError> {
        let req = Request::LookupBlock(LookupBlock {
            mode: (),
            id: ton_liteapi::tl::common::BlockId {
                workchain: wc,
                shard,
                seqno,
            },
            seqno: Some(()),
            lt: None,
            utime: None,
            with_state_update: None,
            with_value_flow: None,
            with_extra: None,
            with_shard_hashes: None,
            with_prev_blk_signatures: None,
        });
        let rsp = self.exec(req, Some(seqno), None).await?;
        let lite_id = unwrap_lite_rsp!(rsp, BlockHeader)?.id;
        Ok(lite_id.into())
    }

    pub async fn get_block(&self, block_id: BlockIdExt, params: Option<LiteReqParams>) -> Result<BlockData, TonError> {
        let seqno = block_id.seqno;
        let req = Request::GetBlock(GetBlock { id: block_id.into() });
        let rsp = self.exec(req, Some(seqno), params).await?;
        unwrap_lite_rsp!(rsp, BlockData)
    }

    pub async fn get_account_state(
        &self,
        address: &TonAddress,
        mc_seqno: u32,
        params: Option<LiteReqParams>,
    ) -> Result<MaybeAccount, TonError> {
        let req = Request::GetAccountState(GetAccountState {
            id: self.lookup_mc_block(mc_seqno).await?.into(),
            account: AccountId {
                workchain: address.workchain,
                id: Int256(*address.hash.as_slice_sized()),
            },
        });
        let rsp = self.exec_with_timeout(req, Some(mc_seqno), params).await?;
        let account_state_rsp = unwrap_lite_rsp!(rsp, AccountState)?;
        Ok(MaybeAccount::from_boc(account_state_rsp.state)?)
    }

    pub async fn get_libs(&self, lib_ids: &[TonHash], params: Option<LiteReqParams>) -> Result<LibsDict, TonError> {
        self.0.get_libs_impl(lib_ids, params).await
    }

    pub async fn exec(
        &self,
        req: Request,
        wait_mc_seqno: Option<u32>,
        params: Option<LiteReqParams>,
    ) -> Result<Response, TonError> {
        self.exec_with_timeout(req, wait_mc_seqno, params).await
    }

    pub async fn exec_with_timeout(
        &self,
        request: Request,
        wait_mc_seqno: Option<u32>,
        params: Option<LiteReqParams>,
    ) -> Result<Response, TonError> {
        self.0.exec_with_retries(request, wait_mc_seqno, params).await
    }
}

struct Inner {
    default_req_params: LiteReqParams,
    conn_pool: AutoPool<Connection>,
    global_req_id: AtomicU64,
}

impl Inner {
    async fn get_libs_impl(&self, lib_ids: &[TonHash], params: Option<LiteReqParams>) -> Result<LibsDict, TonError> {
        let mut libs_dict = LibsDict::default();
        for chunk in lib_ids.chunks(16) {
            let request = Request::GetLibraries(GetLibraries {
                library_list: chunk.iter().map(|x| Int256(*x.as_slice_sized())).collect(),
            });
            let rsp = self.exec_with_retries(request, None, params).await?;
            let result = unwrap_lite_rsp!(rsp, LibraryResult)?;
            let dict_items = result
                .result
                .into_iter()
                .map(|x| {
                    let hash = TonHash::from_slice_sized(&x.hash.0);
                    let lib = TonCell::from_boc(x.data)?;
                    Ok::<_, TonCoreError>((hash, lib))
                })
                .collect::<Result<Vec<_>, TonCoreError>>()?;

            let req_cnt = chunk.len();
            let rsp_cnt = dict_items.len();
            if req_cnt != rsp_cnt {
                let got_hashes: Vec<_> = dict_items.iter().map(|x| &x.0).collect();
                log::warn!(
                    "[get_libs_impl] expected {req_cnt} libs, got {rsp_cnt}:\n\
                    requested: {chunk:?}\n\
                    got: {got_hashes:?}",
                );
            }
            for item in dict_items {
                libs_dict.insert(item.0, item.1.into());
            }
        }

        Ok(libs_dict)
    }

    async fn exec_with_retries(
        &self,
        req: Request,
        wait_seqno: Option<u32>,
        params: Option<LiteReqParams>,
    ) -> Result<Response, TonError> {
        let wrap_req = WrappedRequest {
            wait_masterchain_seqno: wait_seqno.map(|seqno| WaitMasterchainSeqno {
                seqno,
                timeout_ms: WAIT_MC_SEQNO_MS,
            }),
            request: req,
        };
        let req_params = params.as_ref().unwrap_or(&self.default_req_params);
        let req_id = self.global_req_id.fetch_add(1, Relaxed);
        let fi = FixedInterval::new(req_params.retry_waiting);
        let strategy = fi.take(req_params.retries_count as usize);
        let exec_request = || async { self.exec_impl(req_id, &wrap_req, req_params.query_timeout).await };
        RetryIf::spawn(strategy, exec_request, retry_condition).await
    }

    async fn exec_impl(&self, req_id: u64, req: &WrappedRequest, req_timeout: Duration) -> Result<Response, TonError> {
        log::trace!("LiteClient exec_impl: req_id={req_id}, req={req:?}");
        // pool is configured to spin until get connection
        let mut conn = self.conn_pool.get_async().await.unwrap();
        conn.exec(req.clone(), req_timeout).await
    }
}

fn retry_condition(error: &TonError) -> bool { !matches!(error, TonError::LiteClientWrongResponse(..)) }
