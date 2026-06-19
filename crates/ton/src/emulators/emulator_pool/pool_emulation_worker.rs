use crate::emulators::emulator_pool::thread_pool::PoolObject;
use crate::emulators::emulator_pool::{PoolEmulationResponse, PoolEmulationTask};
use crate::emulators::tvm_emulator::TVMEmulator;
use crate::emulators::tx_emulator::TXEmulator;
use crate::errors::TonResult;

pub(super) struct PoolEmulationWorker {
    pub(super) description: String,
    pub(super) tx_emulator: TXEmulator,
}

impl PoolObject for PoolEmulationWorker {
    type Task = PoolEmulationTask;
    type Retval = PoolEmulationResponse;

    #[rustfmt::skip]
    fn process<T: Into<Self::Task>>(&mut self, task: T) -> TonResult<Self::Retval> {
        match task.into() {
            PoolEmulationTask::EmulGetMethod(args) => {
                TVMEmulator::from_state(&args.state)?
                    .emul_get_method(args.method, &args.stack_boc)
                    .map(PoolEmulationResponse::EmulGetMethod)
            },
            PoolEmulationTask::EmulSendExtMsg(args) =>  {
                TVMEmulator::from_state(&args.state)?
                    .emul_send_ext_msg(&args.msg_boc)
                    .map(PoolEmulationResponse::EmulSendExtMsg)
            },
            PoolEmulationTask::EmulSendIntMsg(args) => {
                TVMEmulator::from_state(&args.state)?
                    .emul_send_int_msg(&args.msg_boc, args.amount)
                    .map(PoolEmulationResponse::EmulSendIntMsg)
            },
            PoolEmulationTask::EmulOrdTx(args) => {
                self.tx_emulator
                    .emulate_ord(&args)
                    .map(PoolEmulationResponse::EmulOrdTx)
            },
            PoolEmulationTask::EmulTickTockTx(args) => {
                self.tx_emulator
                    .emulate_ticktock(&args)
                    .map(PoolEmulationResponse::EmulTickTockTx)
            },
        }
    }
    fn descriptor(&self) -> &str { &self.description }
}
