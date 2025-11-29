mod builder;
mod task_counter;

use crate::errors::{TonError, TonResult};
use crate::thread_pool::builder::Builder;
use crate::thread_pool::task_counter::TaskCounter;
use std::ops::Add;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::mpsc::Sender;
use std::time::{Duration, SystemTime};
use tokio::sync::oneshot;

const SLEEP_ON_FULL_QUEUE: Duration = Duration::from_millis(3);
const MIN_QUEUE_LEN_TO_ACCEPT_TASKS: usize = 2;

pub trait PoolObject: Send + Sync + 'static {
    type Task: Send + Sync;
    type Retval: Send + Sync;
    fn process<T: Into<Self::Task>>(&mut self, task: T) -> TonResult<Self::Retval>;
    /// any human-readable value for logging purposes
    fn descriptor(&self) -> &str { "undefined" }
}

/// Depends on number of objects provided, run one thread per object
#[derive(Clone)]
pub struct ThreadPool<T: PoolObject>(Arc<Inner<T>>);

impl<Obj: PoolObject> ThreadPool<Obj> {
    pub fn builder(objects: Vec<Obj>) -> TonResult<Builder<Obj>> { Builder::new(objects) }

    pub async fn exec<T: Into<Obj::Task>>(&self, task: T, timeout: Option<Duration>) -> TonResult<Obj::Retval> {
        let exec_timeout = timeout.unwrap_or(self.0.default_exec_timeout);

        match tokio::time::timeout(exec_timeout, self.0.exec_impl(task.into(), exec_timeout)).await {
            Ok(res) => res,
            Err(_) => Err(TonError::EmulatorPoolTimeout(exec_timeout)),
        }
    }

    pub fn get_counters(&self) -> &Vec<TaskCounter> { &self.0.counters }

    pub fn get_counters_aggregated(&self) -> TaskCounter {
        let in_progress = self.0.counters.iter().map(|c| c.in_progress.load(Ordering::Relaxed)).sum();
        let done = self.0.counters.iter().map(|c| c.done.load(Ordering::Relaxed)).sum();
        let failed = self.0.counters.iter().map(|c| c.failed.load(Ordering::Relaxed)).sum();

        TaskCounter {
            in_progress: AtomicUsize::new(in_progress),
            done: AtomicUsize::new(done),
            failed: AtomicUsize::new(failed),
        }
    }

    pub fn print_stats(&self) -> String {
        let mut result = String::new();

        // Table header
        result.push_str("ThreadPool Statistics\n");
        result.push_str(&"=".repeat(100));
        result.push('\n');
        result
            .push_str(&format!("{:<10} {:<15} {:<15} {:<15}\n", "Thread", "InProgress", "Done Tasks", "Failed Tasks"));
        result.push_str(&"-".repeat(100));
        result.push('\n');

        // Collect per-thread statistics
        let mut total_in_progress = 0;
        let mut total_done = 0;
        let mut total_failed = 0;

        for idx in 0..self.0.senders.len() {
            let in_progress = self.0.counters[idx].in_progress.load(Ordering::Relaxed);
            let done_tasks = self.0.counters[idx].done.load(Ordering::Relaxed);
            let failed = self.0.counters[idx].failed.load(Ordering::Relaxed);

            total_in_progress += in_progress;
            total_done += done_tasks;
            total_failed += failed;

            result.push_str(&format!(
                "{:<10} {:<15} {:<15} {:<15} {:<15}\n",
                idx, in_progress, done_tasks, failed, in_progress
            ));
        }

        // Total row
        result.push_str(&"-".repeat(100));
        result.push('\n');

        result
            .push_str(&format!("{:<10} {:<15} {:<15} {:<15}\n", "TOTAL", total_in_progress, total_done, total_failed));
        result.push_str(&"=".repeat(100));
        result.push('\n');

        result
    }
}

struct PoolTask<Obj: PoolObject> {
    task: Obj::Task,
    rsp_sender: oneshot::Sender<TonResult<Obj::Retval>>,
    timeout: Duration,
    deadline: SystemTime,
}

struct Inner<Obj: PoolObject> {
    senders: Vec<Sender<PoolTask<Obj>>>,
    counters: Vec<TaskCounter>,
    default_exec_timeout: Duration,
    max_thread_queue_len: usize,
}

impl<Obj: PoolObject> Inner<Obj> {
    async fn find_free_thread(&self) -> usize {
        let mut chosen_thread_pos = self.senders.len(); // invalid index
        let mut chosen_queue_len = self.max_thread_queue_len + 1; // invalid length

        loop {
            for pos in 0..self.senders.len() {
                let cur_queue_len = self.counters[pos].in_progress.load(Ordering::Relaxed);
                if cur_queue_len <= MIN_QUEUE_LEN_TO_ACCEPT_TASKS {
                    return pos;
                }
                if cur_queue_len <= self.max_thread_queue_len && cur_queue_len < chosen_queue_len {
                    chosen_queue_len = cur_queue_len;
                    chosen_thread_pos = pos;
                }
            }
            if chosen_thread_pos < self.senders.len() {
                return chosen_thread_pos;
            }
            tokio::time::sleep(SLEEP_ON_FULL_QUEUE).await;
        }
    }

    async fn exec_impl(&self, task: Obj::Task, timeout: Duration) -> TonResult<Obj::Retval> {
        let (tx, rx) = oneshot::channel();
        let pool_task = PoolTask {
            task,
            rsp_sender: tx,
            timeout,
            deadline: SystemTime::now().add(timeout),
        };

        let thread_idx = self.find_free_thread().await;
        let counter_updater = self.counters[thread_idx].task_added();

        self.senders[thread_idx].send(pool_task).map_err(TonError::system)?;
        let emul_result = rx.await.map_err(TonError::system)?;
        counter_updater.task_done();
        emul_result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestObject(usize);

    impl PoolObject for TestObject {
        type Task = usize;
        type Retval = usize;
        fn process<T: Into<Self::Task>>(&mut self, task: T) -> Result<usize, TonError> {
            Ok(self.0 * 1000 + task.into())
        }
        fn descriptor(&self) -> &str { "TestObject" }
    }

    #[tokio::test]
    async fn test_thread_pool_basic() -> anyhow::Result<()> {
        let objects = vec![TestObject(1), TestObject(2)];

        let pool = ThreadPool::builder(objects)?.build()?;

        let result = pool.exec(42usize, None).await?;

        // Emulator ordering is not guaranteed, so check both possibilities
        assert!(result == 1042 || result == 2042);

        let counter = pool.get_counters_aggregated();
        println!("{:?}", counter);
        assert_eq!(counter.in_progress.load(Ordering::Relaxed), 0);
        assert_eq!(counter.done.load(Ordering::Relaxed), 1);
        assert_eq!(counter.failed.load(Ordering::Relaxed), 0);
        Ok(())
    }
}
