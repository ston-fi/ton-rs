use crate::clients::tl_client::tl::request::TLRequest;
use crate::clients::tl_client::tl::{request_context::TLRequestCtx, response::TLResponse};
use crate::errors::TonError;
use std::sync::Arc;

/// Check tl_conn_default (mostly `run_loop` method) for method execution flow
#[allow(unused)]
pub trait TLCallback: Send + Sync {
    fn on_loop_enter(&self, tag: &str);
    fn on_loop_exit(&self, tag: &str);

    fn before_send(&self, tag: &str, req_ctx: &TLRequestCtx, req: &TLRequest);
    fn on_send_error(&self, tag: &str, req_ctx: &TLRequestCtx, err: &TonError);
    fn on_idle(&self, tag: &str);
    fn on_result(&self, tag: &str, result: &Result<(TLResponse, Option<String>), TonError>);
    fn on_response(&self, tag: &str, rrsp: &TLResponse, eq_ctx: Option<&TLRequestCtx>);
    fn on_notify_error(&self, tag: &str, rsp: &Result<TLResponse, TonError>);
}

#[derive(Clone, Default)]
pub struct TLCallbacksStore {
    pub callbacks: Arc<Vec<Box<dyn TLCallback>>>,
}

impl TLCallback for TLCallbacksStore {
    fn on_loop_enter(&self, tag: &str) { self.callbacks.iter().for_each(|cb| cb.on_loop_enter(tag)); }
    fn on_loop_exit(&self, tag: &str) { self.callbacks.iter().for_each(|cb| cb.on_loop_exit(tag)); }
    fn before_send(&self, tag: &str, req_ctx: &TLRequestCtx, req: &TLRequest) {
        self.callbacks.iter().for_each(|cb| cb.before_send(tag, req_ctx, req));
    }
    fn on_send_error(&self, tag: &str, req_ctx: &TLRequestCtx, err: &TonError) {
        self.callbacks.iter().for_each(|cb| cb.on_send_error(tag, req_ctx, err));
    }
    fn on_idle(&self, tag: &str) { self.callbacks.iter().for_each(|cb| cb.on_idle(tag)); }
    fn on_result(&self, tag: &str, result: &Result<(TLResponse, Option<String>), TonError>) {
        self.callbacks.iter().for_each(|cb| cb.on_result(tag, result));
    }
    fn on_response(&self, tag: &str, rsp: &TLResponse, req_ctx: Option<&TLRequestCtx>) {
        self.callbacks.iter().for_each(|cb| cb.on_response(tag, rsp, req_ctx));
    }
    fn on_notify_error(&self, tag: &str, rsp: &Result<TLResponse, TonError>) {
        self.callbacks.iter().for_each(|cb| cb.on_notify_error(tag, rsp));
    }
}

pub struct TLCallbackLogTrace {}
impl TLCallback for TLCallbackLogTrace {
    fn on_loop_enter(&self, tag: &str) {
        log::info!("[{tag}] Starting event loop");
    }
    fn on_loop_exit(&self, tag: &str) {
        log::info!("[{tag}] Exiting event loop");
    }
    fn before_send(&self, tag: &str, req_ctx: &TLRequestCtx, req: &TLRequest) {
        log::trace!("[{tag}] Sending request: {req_ctx}, req: {req:?}");
    }
    fn on_send_error(&self, tag: &str, req_ctx: &TLRequestCtx, err: &TonError) {
        log::error!("[{tag}] Fail to send request: {req_ctx}, error: {err:?}");
    }
    fn on_idle(&self, _tag: &str) {}
    fn on_result(&self, tag: &str, result: &Result<(TLResponse, Option<String>), TonError>) {
        match result {
            Ok((rsp, _)) => log::trace!("[{tag}] Received response: {rsp:?}"),
            Err(err) => log::error!("[{tag}] Error receiving response: {err:?}"),
        }
    }

    fn on_response(&self, tag: &str, rsp: &TLResponse, req_ctx: Option<&TLRequestCtx>) {
        match req_ctx {
            Some(ctx) => log::trace!("[{tag}] Received response: {rsp:?}, {ctx}"),
            None => log::trace!("[{tag}] Received response: {rsp:?}"),
        }
    }

    fn on_notify_error(&self, tag: &str, rsp: &Result<TLResponse, TonError>) {
        log::error!("[{tag}] Fail to send notify: {rsp:?}");
    }
}
