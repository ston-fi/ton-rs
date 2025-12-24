use crate::emulators::emulator_pool::thread_pool::ThreadPool;
use crate::emulators::emulator_pool::{EmulatorPool, PoolEmulationWorker};
use crate::emulators::tx_emulator::TXEmulator;
use crate::errors::{TonError, TonResult};
use derive_setters::Setters;
use std::cmp::max;
use std::thread;
use std::time::Duration;

#[derive(Setters)]
#[setters(prefix = "with_", strip_option)]
pub struct Builder {
    threads_count: usize,
    default_exec_timeout: Duration,
    max_thread_queue_len: usize,
    emulator_log_level: u32,
    emulator_debug_enabled: bool,
}

impl Builder {
    pub fn new() -> TonResult<Self> {
        let cores_count = thread::available_parallelism().map_err(TonError::system)?.get();
        let builder = Self {
            threads_count: max(1, cores_count - 1), // leave one core for the rest of the system
            default_exec_timeout: Duration::from_secs(1),
            max_thread_queue_len: 10,
            emulator_log_level: 0,
            emulator_debug_enabled: false,
        };
        Ok(builder)
    }

    pub fn build(self) -> TonResult<EmulatorPool> {
        if self.threads_count == 0 {
            log::warn!("EmulationPool is configured to use 0 threads (it won't do any emulations)");
        }
        let mut workers = vec![];
        for th in 0..self.threads_count {
            let worker = PoolEmulationWorker {
                description: format!("EmulationWorker_{th}"),
                tx_emulator: TXEmulator::new(self.emulator_log_level, self.emulator_debug_enabled)?,
            };
            workers.push(worker)
        }
        let thread_pool = ThreadPool::builder(workers)?
            .with_default_exec_timeout(self.default_exec_timeout)
            .with_max_thread_queue_len(self.max_thread_queue_len)
            .build()?;
        Ok(EmulatorPool(thread_pool))
    }
}
