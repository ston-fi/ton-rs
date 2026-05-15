mod builder;
mod connection;
mod lite_types;
mod liteapi_serde;
pub mod metrics;
mod req_params;
mod unwrap_lite_rsp;

pub use builder::*;
pub use lite_types::*;
pub use req_params::*;

use crate::block_tlb::{Block, BlockIdExt, MaybeAccount};
use crate::errors::{TonError, TonResult};
use crate::libs_dict::LibsDict;
use crate::lite_client::connection::Connection;
use crate::lite_client::metrics::{LiteClientMetrics, LiteClientMetricsSnapshot, MetricGuard};
use crate::{bail_ton, unwrap_lite_rsp};
use auto_pool::pool::AutoPool;
use everscale_types::boc::Boc;
use everscale_types::cell::HashBytes;
use everscale_types::models::ShardState;
use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::{AtomicU32, AtomicU64};
use std::time::{Duration, Instant};
use tokio_retry::RetryIf;
use tokio_retry::strategy::FixedInterval;
use ton_core::cell::{TonCell, TonHash};
use ton_core::constants::{TON_MASTERCHAIN, TON_SHARD_FULL};
use ton_core::errors::TonCoreError;
use ton_core::traits::tlb::TLB;
use ton_core::types::TonAddress;
use ton_liteapi::tl::common::{AccountId, Int256};
use ton_liteapi::tl::request::*;
use ton_liteapi::tl::response::{BlockState, Response};

const WAIT_MC_SEQNO_MS: u32 = 1000;
const WAIT_CONNECTION_MS: u64 = 5;

#[derive(Clone)]
pub struct LiteClient(Arc<Inner>);

// converts ton_block -> ton_liteapi objects under the hood
impl LiteClient {
    pub fn builder() -> TonResult<Builder> { Builder::new() }

    pub async fn get_mc_info(&self) -> TonResult<MasterchainInfo> {
        let rsp = self.exec(Request::GetMasterchainInfo, None, None).await?;
        let mc_info = unwrap_lite_rsp!(rsp, MasterchainInfo)?;
        Ok(mc_info.into())
    }

    pub async fn lookup_mc_block(&self, seqno: u32) -> TonResult<BlockIdExt> {
        self.lookup_block(TON_MASTERCHAIN, TON_SHARD_FULL, seqno).await
    }

    pub async fn lookup_block(&self, wc: i32, shard: u64, seqno: u32) -> TonResult<BlockIdExt> {
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

    pub async fn get_block(&self, block_id: BlockIdExt, params: Option<LiteReqParams>) -> TonResult<BlockData> {
        let seqno = block_id.seqno;
        let req = Request::GetBlock(GetBlock { id: block_id.into() });
        let rsp = self.exec(req, Some(seqno), params).await?;
        let lite_block_data = unwrap_lite_rsp!(rsp, BlockData)?;
        Ok(BlockData {
            id: lite_block_data.id.into(),
            data: Block::from_boc(lite_block_data.data)?,
        })
    }

    // BlockState is not implemented in block_tlb yet, so we return ton_liteapi::BlockState here
    pub async fn get_block_state(
        &self,
        block_id: BlockIdExt,
        params: Option<LiteReqParams>,
    ) -> Result<BlockState, TonError> {
        let seqno = block_id.seqno;
        let req = Request::GetState(GetState { id: block_id.into() });
        let rsp = self.exec(req, Some(seqno), params).await?;
        unwrap_lite_rsp!(rsp, BlockState)
    }

    pub async fn get_account_state(
        &self,
        address: &TonAddress,
        mc_seqno: u32,
        params: Option<LiteReqParams>,
    ) -> TonResult<MaybeAccount> {
        if mc_seqno == 0 {
            // zero state can't be received from lite-node directly
            // but we can extract it from zero state
            let block_id = if self.0.mainnet {
                BlockIdExt::ZERO_BLOCK_ID
            } else {
                BlockIdExt::ZERO_BLOCK_ID_TESTNET
            };
            let state = self.get_block_state(block_id, params).await?;
            let cell = Boc::decode(&state.data)?;
            let shard_state: ShardState = cell.parse()?;
            let ShardState::Unsplit(unsplit) = shard_state else {
                bail_ton!("zero state must be unsplit")
            };
            let Some((_, account)) = unsplit.load_accounts()?.get(HashBytes(*address.hash.as_slice_sized()))? else {
                bail_ton!("Account with address {} not found in zero block", address)
            };
            let maybe_account = MaybeAccount::from_boc(Boc::encode(account.account.inner()))?;
            return Ok(maybe_account);
        }

        let req = Request::GetAccountState(GetAccountState {
            id: self.lookup_mc_block(mc_seqno).await?.into(),
            account: AccountId {
                workchain: address.workchain,
                id: Int256(*address.hash.as_slice_sized()),
            },
        });
        let rsp = self.exec(req, Some(mc_seqno), params).await?;
        let account_state_rsp = unwrap_lite_rsp!(rsp, AccountState)?;
        Ok(MaybeAccount::from_boc(account_state_rsp.state)?)
    }

    pub async fn get_libs(&self, lib_ids: &[TonHash], params: Option<LiteReqParams>) -> TonResult<LibsDict> {
        self.0.get_libs_impl(lib_ids, params).await
    }

    pub async fn send_msg(&self, body: Vec<u8>, params: Option<LiteReqParams>) -> TonResult<u32> {
        let request = Request::SendMessage(SendMessage { body });
        let rsp = self.exec(request, None, params).await?;
        let status = unwrap_lite_rsp!(rsp, SendMsgStatus)?;
        Ok(status.status)
    }

    pub async fn exec(
        &self,
        req: Request,
        wait_mc_seqno: Option<u32>,
        params: Option<LiteReqParams>,
    ) -> Result<Response, TonError> {
        #[allow(deprecated)]
        self.exec_with_timeout(req, wait_mc_seqno, params).await
    }

    #[deprecated]
    pub async fn exec_with_timeout(
        &self,
        request: Request,
        wait_mc_seqno: Option<u32>,
        params: Option<LiteReqParams>,
    ) -> Result<Response, TonError> {
        self.0.exec_with_retries(request, wait_mc_seqno, params).await
    }

    pub fn metrics(&self) -> LiteClientMetricsSnapshot { self.0.metrics.snapshot() }
}

struct Inner {
    mainnet: bool,
    default_req_params: LiteReqParams,
    conn_pool: AutoPool<Connection>,
    global_req_id: AtomicU64,
    metrics: LiteClientMetrics,
}

impl Inner {
    async fn get_libs_impl(&self, lib_ids: &[TonHash], params: Option<LiteReqParams>) -> TonResult<LibsDict> {
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
        let attempts = AtomicU32::new(0);
        let fi = FixedInterval::new(req_params.retry_waiting);
        let strategy = fi.take(req_params.retries_count as usize);

        let exec_request = || async {
            let is_retry = attempts.fetch_add(1, Relaxed) > 0;
            self.exec_impl(req_id, &wrap_req, req_params.query_timeout, is_retry).await
        };
        RetryIf::spawn(strategy, exec_request, retry_condition).await
    }

    async fn exec_impl(
        &self,
        req_id: u64,
        req: &WrappedRequest,
        req_timeout: Duration,
        is_retry: bool,
    ) -> TonResult<Response> {
        log::trace!("LiteClient exec_impl: req_id={req_id}, req={req:?}");
        let mut metric_guard = MetricGuard::new(&self.metrics, &req.request, is_retry);

        // pool is configured to spin until get connection
        let wait_connection_started = Instant::now();
        let mut conn = self.conn_pool.get_async().await.unwrap();
        metric_guard.record_connection_wait(wait_connection_started.elapsed());

        let result = conn.exec(req.clone(), req_timeout).await;
        metric_guard.set_ok(result.is_ok());
        result
    }
}

fn retry_condition(error: &TonError) -> bool { !matches!(error, TonError::LiteClientWrongResponse(..)) }
