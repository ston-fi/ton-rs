use crate::emulators::tvm_emulator::{TVMGetMethodID, TVMState};
use crate::emulators::tx_emulator::{TXEmulOrdArgs, TXEmulTickTockArgs};
use std::sync::Arc;

pub enum PoolEmulationTask {
    EmulGetMethod(TVMGetMethodTask),
    EmulSendExtMsg(TVMSendExtMsgTask),
    EmulSendIntMsg(TVMSendIntMsgTask),
    EmulOrdTx(TXEmulOrdArgs),
    EmulTickTockTx(TXEmulTickTockArgs),
}

impl From<TVMGetMethodTask> for PoolEmulationTask {
    fn from(task: TVMGetMethodTask) -> Self { PoolEmulationTask::EmulGetMethod(task) }
}
impl From<TVMSendExtMsgTask> for PoolEmulationTask {
    fn from(task: TVMSendExtMsgTask) -> Self { PoolEmulationTask::EmulSendExtMsg(task) }
}
impl From<TVMSendIntMsgTask> for PoolEmulationTask {
    fn from(task: TVMSendIntMsgTask) -> Self { PoolEmulationTask::EmulSendIntMsg(task) }
}
impl From<TXEmulOrdArgs> for PoolEmulationTask {
    fn from(args: TXEmulOrdArgs) -> Self { PoolEmulationTask::EmulOrdTx(args) }
}
impl From<TXEmulTickTockArgs> for PoolEmulationTask {
    fn from(args: TXEmulTickTockArgs) -> Self { PoolEmulationTask::EmulTickTockTx(args) }
}

#[derive(Clone, Debug)]
pub struct TVMGetMethodTask {
    pub state: TVMState,
    pub method: TVMGetMethodID,
    pub stack_boc: Arc<Vec<u8>>,
}

#[derive(Clone, Debug)]
pub struct TVMSendIntMsgTask {
    pub state: TVMState,
    pub msg_boc: Arc<Vec<u8>>,
    pub amount: u64,
}

#[derive(Clone, Debug)]
pub struct TVMSendExtMsgTask {
    pub state: TVMState,
    pub msg_boc: Arc<Vec<u8>>,
}
