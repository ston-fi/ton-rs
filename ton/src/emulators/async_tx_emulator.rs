use super::tx_emulator::TXEmulator;
use crate::emulators::tx_emulator::TXEmulOrdArgs;
use crate::emulators::tx_emulator::TXEmulTickTockArgs;
use crate::emulators::tx_emulator::TXEmulationSuccess;
use crate::errors::{TonError, TonResult};
use nacl::compare;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use tokio::sync::oneshot;

pub type EmulatorResult = TonResult<TXEmulationSuccess>;
enum Task {
    EmulateOrd(TXEmulOrdArgs),
    EmulateTiktock(TXEmulTickTockArgs),
}

struct InternalCmd {
    tx: oneshot::Sender<EmulatorResult>,
    task: Task,
}

type CmdChannel = (Sender<InternalCmd>, Receiver<InternalCmd>);

struct AsyncTxEmulator {
    // Fields for the threaded transaction emulator
    tx: Sender<InternalCmd>,
    handler: thread::JoinHandle<TonResult<u64>>,
}

impl AsyncTxEmulator {
    fn new(log_level: u32, debug_enabled: bool) -> TonResult<Self> {
        let obj = TXEmulator::new(log_level, debug_enabled)?;

        let (tx, rx): CmdChannel = mpsc::channel();

        let handler = thread::spawn(move || Self::worker_loop(obj, rx));
        Ok(Self { tx, handler })
    }
    pub async fn emulate_ord(&mut self, args: &TXEmulOrdArgs) -> EmulatorResult {
        self.execute_task(Task::EmulateOrd(args.clone())).await
    }

    pub async fn emulate_ticktock(&mut self, args: &TXEmulTickTockArgs) -> EmulatorResult {
        self.execute_task(Task::EmulateTiktock(args.clone())).await
    }
    async fn execute_task(&self, task: Task) -> EmulatorResult {
        let (tx, rx) = oneshot::channel();
        self.tx.send(InternalCmd { tx, task }).map_err(|e| TonError::Custom(format!("send task error: {e}")))?;
        Ok(rx.await.map_err(|e| TonError::Custom(format!("receive task error: {e}")))??)
    }

    fn worker_loop(obj: TXEmulator, receiver: Receiver<InternalCmd>) -> TonResult<u64> {
        let mut counter = 0;
        loop {
            let command = receiver.recv();
            counter += 1;
            let rv = match command.task {
                Task::EmulateOrd(args) => obj.emulate_ord(args),
                Task::EmulateTiktock(args) => obj.emulate_tiktock(args),
            };
            let _ = command.tx.send(rv);
        }
        Ok(counter)
    }
}
