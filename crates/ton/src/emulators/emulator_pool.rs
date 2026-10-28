mod builder;
mod pool_emulation_response;
mod pool_emulation_task;
mod pool_emulation_worker;
mod thread_pool;

pub use pool_emulation_response::*;
pub use pool_emulation_task::*;

use crate::emulators::emulator_pool::builder::Builder;
use crate::emulators::emulator_pool::pool_emulation_worker::PoolEmulationWorker;
use crate::emulators::emulator_pool::thread_pool::{TaskCounter, ThreadPool};
use crate::emulators::tvm_emulator::{TVMGetMethodResponse, TVMSendMsgResponse};
use crate::emulators::tx_emulator::{TXEmulOrdArgs, TXEmulTickTockArgs, TXEmulationResponse};
use crate::errors::TonResult;
use std::time::Duration;

#[derive(Clone)]
pub struct EmulatorPool(ThreadPool<PoolEmulationWorker>);

impl EmulatorPool {
    pub fn builder() -> TonResult<Builder> {
        Builder::new()
    }
    /// timeout == None means default timeout will be used
    pub async fn emul_get_method(
        &self,
        task: TVMGetMethodTask,
        timeout: Option<Duration>,
    ) -> TonResult<TVMGetMethodResponse> {
        self.0.exec(task, timeout).await?.try_into()
    }
    pub async fn emul_send_ext_msg(
        &self,
        task: TVMSendExtMsgTask,
        timeout: Option<Duration>,
    ) -> TonResult<TVMSendMsgResponse> {
        self.0.exec(task, timeout).await?.try_into()
    }
    pub async fn emul_send_int_msg(
        &self,
        task: TVMSendIntMsgTask,
        timeout: Option<Duration>,
    ) -> TonResult<TVMSendMsgResponse> {
        self.0.exec(task, timeout).await?.try_into()
    }
    pub async fn emul_ord_tx(&self, task: TXEmulOrdArgs, timeout: Option<Duration>) -> TonResult<TXEmulationResponse> {
        self.0.exec(task, timeout).await?.try_into()
    }
    pub async fn emul_ticktock_tx(
        &self,
        task: TXEmulTickTockArgs,
        timeout: Option<Duration>,
    ) -> TonResult<TXEmulationResponse> {
        self.0.exec(task, timeout).await?.try_into()
    }
    pub fn get_counters(&self) -> &Vec<TaskCounter> {
        self.0.get_counters()
    }
    pub fn get_counters_aggregated(&self) -> TaskCounter {
        self.0.get_counters_aggregated()
    }
    pub fn display_stats(&self) -> String {
        self.0.display_stats()
    }
}

#[cfg(test)]
#[path = "../tests/emulators/test_emulator_pool.rs"]
mod tests;
