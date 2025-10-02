use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Weak};
use std::thread;
use std::time::{Duration, Instant};

use crate::errors::TonError;
use crate::sys_utils::sys_tonlib_set_verbosity_level;
use crate::tl_client::config::{LiteNodeFilter, TLClientConfig};
use crate::tl_client::tl::tonlibjson_wrapper::TonLibJsonWrapper;
use crate::tl_client::tl::*;
use crate::tl_client::*;
use crate::unwrap_tl_rsp;
use async_trait::async_trait;
use tokio::sync::{oneshot, Mutex, Semaphore};
use ton_lib_core::constants::{TON_MASTERCHAIN, TON_SHARD_FULL};

static CONNECTION_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Clone)]
pub struct TLConnection {
    inner: Arc<Inner>,
}

struct Inner {
    tonlibjson_wrapper: TonLibJsonWrapper,
    active_requests: Mutex<HashMap<u64, TLRequestCtx>>,
    semaphore: Arc<Semaphore>,
    next_request_id: AtomicU64,
    callbacks: TLCallbacksStore,
}

#[async_trait]
impl TLClientTrait for TLConnection {
    fn get_connection(&self) -> &TLConnection { self }

    fn get_retry_strategy(&self) -> &RetryStrategy {
        static NO_RETRY: RetryStrategy = RetryStrategy {
            retry_count: 0,
            retry_waiting: Duration::from_millis(0),
        };
        &NO_RETRY
    }
}

impl TLConnection {
    pub async fn new(config: &TLClientConfig, semaphore: Arc<Semaphore>) -> Result<TLConnection, TonError> {
        new_connection_checked(config, semaphore).await
    }

    pub async fn exec_impl(&self, req: &TLRequest) -> Result<TLResponse, TonError> { self.inner.exec_impl(req).await }

    async fn init(&self, options: TLOptions) -> Result<TLOptionsInfo, TonError> {
        let req = TLRequest::Init { options };
        unwrap_tl_rsp!(self.exec_impl(&req).await?, TLOptionsInfo)
    }
}

impl Inner {
    pub async fn exec_impl(&self, req: &TLRequest) -> Result<TLResponse, TonError> {
        let _permit = self.semaphore.acquire().await?;
        let req_id = self.next_request_id.fetch_add(1, Ordering::Relaxed);
        let tag = self.tonlibjson_wrapper.tag();

        let (sender, receiver) = oneshot::channel();
        let req_ctx = TLRequestCtx {
            id: req_id,
            method: req.into(),
            send_time: Instant::now(),
            sender,
        };

        self.callbacks.before_send(tag, &req_ctx, req);
        self.active_requests.lock().await.insert(req_id, req_ctx);

        let extra = req_id.to_string();
        if let Err(err) = self.tonlibjson_wrapper.send(req, &extra) {
            let req_ctx = self.active_requests.lock().await.remove(&req_id).unwrap();
            self.callbacks.on_send_error(tag, &req_ctx, &err);
            return Err(err);
        }
        receiver.await?
    }
}

// receiving updates from tonlibjson
fn run_loop(tag: String, weak_inner: Weak<Inner>, callbacks: TLCallbacksStore) {
    callbacks.on_loop_enter(&tag);
    while let Some(inner) = weak_inner.upgrade() {
        let tag = inner.tonlibjson_wrapper.tag();
        let result = match inner.tonlibjson_wrapper.receive(1.0) {
            Some(res) => res,
            None => {
                callbacks.on_idle(tag);
                continue;
            }
        };
        callbacks.on_result(tag, &result);
        let (response, maybe_extra) = match result {
            Ok(res) => res,
            Err(_) => continue,
        };

        let maybe_req_ctx = maybe_extra
            .and_then(|extra| extra.parse::<u64>().ok())
            .and_then(|req_id| inner.active_requests.blocking_lock().remove(&req_id));

        callbacks.on_response(tag, &response, maybe_req_ctx.as_ref());

        if let Some(req_ctx) = maybe_req_ctx {
            if let Err(val) = req_ctx.sender.send(Ok(response)) {
                callbacks.on_notify_error(tag, &val);
            }
        }
    }
    callbacks.on_loop_exit(&tag);
}

async fn new_connection_checked(config: &TLClientConfig, semaphore: Arc<Semaphore>) -> Result<TLConnection, TonError> {
    let conn = loop {
        let conn = new_connection(config, semaphore.clone()).await?;
        match config.connection_check {
            LiteNodeFilter::Healthy => match conn.get_mc_info().await {
                Ok(info) => match conn.get_block_header(info.last).await {
                    Ok(_) => break conn,
                    Err(err) => log::info!("Dropping connection to unhealthy node: {err:?}"),
                },
                Err(err) => log::info!("Dropping connection to unhealthy node: {err:?}"),
            },
            LiteNodeFilter::Archive => {
                let block_id = TLBlockId {
                    workchain: TON_MASTERCHAIN,
                    shard: TON_SHARD_FULL as i64,
                    seqno: 1,
                };
                conn.sync().await?;
                match conn.lookup_block(1, block_id, 0, 0).await {
                    Ok(_) => break conn,
                    Err(err) => log::info!("Dropping connection to non-archive node: {err:?}"),
                }
            }
        };
    };
    log::debug!("Connection {} established", conn.inner.tonlibjson_wrapper.tag());
    sys_tonlib_set_verbosity_level(config.tonlib_verbosity_level);
    Ok(conn)
}

async fn new_connection(config: &TLClientConfig, semaphore: Arc<Semaphore>) -> Result<TLConnection, TonError> {
    let conn_id = CONNECTION_COUNTER.fetch_add(1, Ordering::Relaxed);
    let tag = format!("ton-conn-{conn_id}");

    let inner = Arc::new(Inner {
        tonlibjson_wrapper: TonLibJsonWrapper::new(tag.clone())?,
        active_requests: Mutex::new(HashMap::new()),
        semaphore,
        next_request_id: AtomicU64::new(0),
        callbacks: config.callbacks.clone(),
    });
    let init_log_level = match config.tonlib_verbosity_level {
        4 => 1,
        _ => 0,
    };
    sys_tonlib_set_verbosity_level(init_log_level);

    let inner_weak = Arc::downgrade(&inner);
    let callbacks = config.callbacks.clone();
    let _join_handle = thread::Builder::new().name(tag.clone()).spawn(|| run_loop(tag, inner_weak, callbacks))?;

    let conn = TLConnection { inner };
    let _info = conn.init(config.init_opts.clone()).await?;
    Ok(conn)
}
