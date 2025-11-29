use crate::errors::{TonError, TonResult};
use crate::thread_pool::task_counter::TaskCounter;
use crate::thread_pool::{PoolObject, PoolTask, ThreadPool};
use derive_setters::Setters;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::{Duration, SystemTime};

#[derive(Setters)]
#[setters(prefix = "with_", strip_option)]
pub struct Builder<Obj: PoolObject> {
    #[setters(skip)]
    emulators: Vec<Obj>,
    default_emul_timeout: Duration,
    max_thread_queue_len: usize,
}

impl<Obj: PoolObject> Builder<Obj> {
    pub(crate) fn new(emulators: Vec<Obj>) -> TonResult<Self> {
        Ok(Self {
            emulators,
            default_emul_timeout: Duration::from_secs(5),
            max_thread_queue_len: 10,
        })
    }
    pub fn build(mut self) -> TonResult<ThreadPool<Obj>> {
        let threads_count = self.emulators.len();

        let mut senders = Vec::with_capacity(threads_count);
        let mut counters = Vec::with_capacity(threads_count);

        for id in 0..threads_count {
            let (tx, rx) = mpsc::channel::<PoolTask<Obj>>();
            let obj = self.emulators.pop().unwrap();
            let _ = thread::spawn(move || worker_loop(obj, rx, id));
            senders.push(tx);
            counters.push(TaskCounter::new());
        }
        Ok(ThreadPool {
            default_exec_timeout: self.default_emul_timeout,
            max_thread_queue_len: self.max_thread_queue_len,
            senders,
            counters,
        })
    }
}

fn worker_loop<Obj: PoolObject>(mut obj: Obj, receiver: Receiver<PoolTask<Obj>>, id: usize) {
    let log_prefix = format!("EmulatorPool][{}][{}", obj.descriptor(), id);
    log::debug!("[{log_prefix}] thread started");

    while let Ok(task) = receiver.recv() {
        if SystemTime::now() > task.deadline {
            let _ = task.rsp_sender.send(Err(TonError::EmulatorPoolTimeout(task.timeout)));
            continue;
        }
        let emul_result = obj.process(task.task);
        if let Err(_) = task.rsp_sender.send(emul_result) {
            log::debug!("[{log_prefix}] failed to send emul_result, seems user reached the deadline");
        }
    }
    log::debug!("[{log_prefix}] thread completed");
}
