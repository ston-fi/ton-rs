use crate::emulators::thread_pool::{PooledObject, ThreadPool};
use crate::emulators::tx_emulator::TXEmulOrdArgs;
use crate::emulators::tx_emulator::TXEmulTickTockArgs;
use crate::emulators::tx_emulator::TXEmulationSuccess;
use crate::errors::TonResult;

pub type TxEmulatorPool = ThreadPool<crate::emulators::tx_emulator::TXEmulator, TxEmulatorTask, TXEmulationSuccess>;

#[derive(Clone, Debug)]
pub enum TxEmulatorTask {
    TXOrd(TXEmulOrdArgs),
    TXTicktock(TXEmulTickTockArgs),
}

impl PooledObject<TxEmulatorTask, TXEmulationSuccess> for crate::emulators::tx_emulator::TXEmulator {
    fn handle(&mut self, task: TxEmulatorTask) -> TonResult<TXEmulationSuccess> {
        match task {
            TxEmulatorTask::TXOrd(args) => self.emulate_ord(&args),
            TxEmulatorTask::TXTicktock(args) => self.emulate_ticktock(&args),
        }
    }
}
