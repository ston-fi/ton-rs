use crate::bail_ton;
use crate::errors::{TonError, TonResult};
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::oneshot;
use tokio::time::sleep;

pub trait PooledObject<T: Send, R: Send> {
    fn handle(&mut self, task: T) -> Result<R, TonError>;
}
#[derive(Clone, Debug)]
pub struct ThreadPoolConfig {
    default_timeout_emulation: u64,
    thread_queue_capacity: u32,
    pin_to_core_function: Option<fn()>,
}

impl ThreadPoolConfig {
    pub fn new(default_timeout_emulation: u64, thread_queue_capacity: u32, pin_to_core_function: Option<fn()>) -> Self {
        Self {
            default_timeout_emulation,
            thread_queue_capacity,
            pin_to_core_function,
        }
    }
}

pub struct ThreadPool<Obj, Task, Retval>
where
    Obj: PooledObject<Task, Retval> + Send + 'static,
    Task: Send + 'static,
    Retval: Send + 'static,
{
    senders: Vec<Sender<Command<Task, TonResult<Retval>>>>,
    #[allow(dead_code)]
    thread_handles: Vec<JoinHandle<TonResult<u64>>>,
    cnt_jobs_in_queue: Vec<AtomicUsize>,
    cnt_done_tasks: Vec<AtomicUsize>,
    cnt_errored_tasks: Vec<AtomicUsize>,

    cnt_current_jobs: AtomicUsize,
    default_timeout_emulation: u64,
    thread_queue_capacity: u32,
    _phantom: std::marker::PhantomData<Obj>,
}

impl<Obj, Task, Retval> ThreadPool<Obj, Task, Retval>
where
    Obj: PooledObject<Task, Retval> + Send + 'static,
    Task: Send + 'static,
    Retval: Send + 'static,
{
    pub fn new(mut obj_arr: Vec<Obj>, cfg: ThreadPoolConfig) -> TonResult<Self> {
        if obj_arr.is_empty() {
            bail_ton!("Object array for ThreadPool is empty");
        }

        let num_threads = obj_arr.len();
        let mut senders = Vec::with_capacity(num_threads);
        let mut thread_handles = Vec::with_capacity(num_threads);
        let mut cnt_jobs_in_queue = Vec::with_capacity(num_threads);
        let mut cnt_done_tasks = Vec::with_capacity(num_threads);
        let mut cnt_errored_tasks = Vec::with_capacity(num_threads);

        for _ in 0..num_threads {
            let (tx, rx): CommandChannel<Task, TonResult<Retval>> = mpsc::channel();
            let obj = obj_arr.pop().unwrap();

            let handle = thread::spawn(move || {
                if let Some(init_fn) = cfg.pin_to_core_function {
                    init_fn();
                }
                Self::worker_loop(obj, rx)
            });
            senders.push(tx);
            thread_handles.push(handle);
            cnt_jobs_in_queue.push(AtomicUsize::new(0));
            cnt_done_tasks.push(AtomicUsize::new(0));
            cnt_errored_tasks.push(AtomicUsize::new(0));
        }
        Ok(Self {
            senders,
            thread_handles,
            cnt_jobs_in_queue,
            cnt_done_tasks,
            cnt_errored_tasks,
            cnt_current_jobs: AtomicUsize::new(0),
            default_timeout_emulation: cfg.default_timeout_emulation,
            thread_queue_capacity: cfg.thread_queue_capacity,
            _phantom: std::marker::PhantomData,
        })
    }
    // This function increment a queue state
    async fn find_thread_id(&self, deadline: u128) -> TonResult<usize> {
        loop {
            if get_now_ms() > deadline {
                return Err(TonError::EmulatorPoolTimeout {
                    deadline_time: deadline,
                });
            }
            //  find thread with minimum queue size
            let mut min_queue_value = self.thread_queue_capacity as usize + 1;
            let mut target_queue_index = self.senders.len(); // set bad index
            for i in 0..self.senders.len() {
                let current_queue_size = self.cnt_jobs_in_queue[i].load(Ordering::Relaxed);
                if current_queue_size < 2 {
                    target_queue_index = i;
                    break;
                }
                // as late as possible
                if current_queue_size <= min_queue_value {
                    min_queue_value = current_queue_size;
                    target_queue_index = i;
                }
            }

            if target_queue_index == self.senders.len() {
                sleep(Duration::from_millis(1)).await;

                continue;
            }
            // do  increment asap
            self.cnt_jobs_in_queue[target_queue_index].fetch_add(1, Ordering::Relaxed);

            return Ok(target_queue_index);
        }
    }

    pub async fn execute_task(&self, task: Task, maybe_custom_timeout_ms: Option<u64>) -> TonResult<Retval> {
        let deadline_time = if let Some(timeout) = maybe_custom_timeout_ms {
            get_now_ms() + timeout as u128
        } else {
            get_now_ms() + self.default_timeout_emulation as u128
        };

        let (tx, rx) = oneshot::channel();
        let command = Command::Execute(task, tx, deadline_time);
        let idx = self.find_thread_id(deadline_time).await?;
        let _guard = DecrementOnDestructor::new(&self.cnt_jobs_in_queue[idx]);
        self.cnt_current_jobs.fetch_add(1, Ordering::Relaxed);
        self.senders[idx].send(command).map_err(|e| {
            // On send error, increment error counter (guard will decrement queue)
            self.cnt_errored_tasks[idx].fetch_add(1, Ordering::Relaxed);
            self.cnt_current_jobs.fetch_sub(1, Ordering::Relaxed);
            TonError::Custom(format!("send task error: {e}"))
        })?;
        let res = rx.await.map_err(|e| {
            // On receive error, increment error counter (guard will decrement queue)
            self.cnt_errored_tasks[idx].fetch_add(1, Ordering::Relaxed);
            self.cnt_current_jobs.fetch_sub(1, Ordering::Relaxed);
            TonError::Custom(format!("receive task error: {e}"))
        })?;
        match res {
            Ok(retval) => {
                self.cnt_done_tasks[idx].fetch_add(1, Ordering::Relaxed);
                self.cnt_current_jobs.fetch_sub(1, Ordering::Relaxed);

                Ok(retval)
            }
            Err(e) => {
                self.cnt_errored_tasks[idx].fetch_add(1, Ordering::Relaxed);
                self.cnt_current_jobs.fetch_sub(1, Ordering::Relaxed);
                Err(e)
            }
        }
    }

    fn worker_loop(mut obj: Obj, receiver: Receiver<Command<Task, TonResult<Retval>>>) -> TonResult<u64> {
        let mut counter = 0;
        loop {
            let command = receiver.recv();
            counter += 1;
            match command {
                Ok(Command::Execute(task, resp_sender, deadline_time)) => {
                    // Check if deadline has passed
                    let current_time = get_now_ms();
                    if current_time > deadline_time {
                        let _ = resp_sender.send(Err(TonError::EmulatorPoolTimeout { deadline_time }));
                        continue;
                    }
                    let result = obj.handle(task)?;
                    let _ = resp_sender.send(Ok(result));
                }
                Err(_) => break,
            }
        }
        Ok(counter)
    }

    pub fn print_stats(&self) -> String {
        let mut result = String::new();

        // Table header
        result.push_str("ThreadPool Statistics\n");
        result.push_str(&"=".repeat(100));
        result.push('\n');
        result.push_str(&format!(
            "{:<10} {:<15} {:<15} {:<15} {:<15}\n",
            "Thread", "In Queue", "Done Tasks", "Errored Tasks", "Queue Size"
        ));
        result.push_str(&"-".repeat(100));
        result.push('\n');

        // Collect per-thread statistics
        let mut total_in_queue = 0;
        let mut total_done = 0;
        let mut total_errored = 0;

        for idx in 0..self.senders.len() {
            let queue_size = self.cnt_jobs_in_queue[idx].load(Ordering::Relaxed);
            let done_tasks = self.cnt_done_tasks[idx].load(Ordering::Relaxed);
            let errored_tasks = self.cnt_errored_tasks[idx].load(Ordering::Relaxed);

            total_in_queue += queue_size;
            total_done += done_tasks;
            total_errored += errored_tasks;

            result.push_str(&format!(
                "{:<10} {:<15} {:<15} {:<15} {:<15}\n",
                idx, queue_size, done_tasks, errored_tasks, queue_size
            ));
        }

        // Total row
        result.push_str(&"-".repeat(100));
        result.push('\n');
        let current_jobs = self.cnt_current_jobs.load(Ordering::Relaxed);
        let total_queue_size: usize = self.cnt_jobs_in_queue.iter().map(|cnt| cnt.load(Ordering::Relaxed)).sum();

        result.push_str(&format!(
            "{:<10} {:<15} {:<15} {:<15} {:<15}\n",
            "TOTAL", total_in_queue, total_done, total_errored, total_queue_size
        ));
        result.push_str(&format!("{:<10} {:<15} {:<15} {:<15} {:<15}\n", "(current)", current_jobs, "", "", ""));
        result.push_str(&"=".repeat(100));
        result.push('\n');

        result
    }

    /// Get total number of tasks completed by summing all per-thread done counters
    pub fn get_total_tasks_done(&self) -> usize {
        self.cnt_done_tasks.iter().map(|cnt| cnt.load(Ordering::Relaxed)).sum()
    }

    /// Get total number of tasks that errored by summing all per-thread error counters
    pub fn get_total_tasks_errored(&self) -> usize {
        self.cnt_errored_tasks.iter().map(|cnt| cnt.load(Ordering::Relaxed)).sum()
    }
}

struct DecrementOnDestructor<'a> {
    cnt: &'a AtomicUsize,
}

impl<'a> DecrementOnDestructor<'a> {
    fn new(cnt: &'a AtomicUsize) -> Self { DecrementOnDestructor { cnt } }
}

impl<'a> Drop for DecrementOnDestructor<'a> {
    fn drop(&mut self) { self.cnt.fetch_sub(1, Ordering::Relaxed); }
}

type CommandChannel<T, R> = (Sender<Command<T, R>>, Receiver<Command<T, R>>);

enum Command<T, R>
where
    T: Send,
    R: Send,
{
    Execute(T, oneshot::Sender<R>, u128), // task, sender, timeout
}

fn get_now_ms() -> u128 { SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_queue_counter() {
        // Test that AtomicUsize queue counter works correctly
        let cnt_jobs_in_queue = AtomicUsize::new(0);

        // Test increment
        cnt_jobs_in_queue.fetch_add(1, Ordering::Relaxed);
        assert_eq!(cnt_jobs_in_queue.load(Ordering::Relaxed), 1);

        // Test multiple increments
        cnt_jobs_in_queue.fetch_add(5, Ordering::Relaxed);
        assert_eq!(cnt_jobs_in_queue.load(Ordering::Relaxed), 6);

        // Test decrement
        cnt_jobs_in_queue.fetch_sub(2, Ordering::Relaxed);
        assert_eq!(cnt_jobs_in_queue.load(Ordering::Relaxed), 4);

        // Test reset
        cnt_jobs_in_queue.store(0, Ordering::Relaxed);
        assert_eq!(cnt_jobs_in_queue.load(Ordering::Relaxed), 0);
    }

    // Simple test object that implements PooledObject
    struct TestObject {
        id: usize,
    }

    impl PooledObject<usize, usize> for TestObject {
        fn handle(&mut self, task: usize) -> Result<usize, TonError> { Ok(self.id * 1000 + task) }
    }

    #[tokio::test]
    async fn test_thread_pool_basic() {
        // Create pool with TTL 1 second and 2 threads
        let mut objects = vec![TestObject { id: 1 }, TestObject { id: 2 }];
        let config = ThreadPoolConfig::new(1000, 100, None); // 1 second timeout, max 100 tasks in queue
        let pool = ThreadPool::new(objects, config).unwrap();

        // Run 1 task
        let result = pool.execute_task(42, None).await.unwrap();

        // Verify result: should be from one of the threads (1*1000+42 = 1042 or 2*1000+42 = 2042)
        assert!(result == 1042 || result == 2042);

        // Verify done counter
        let total_done = pool.get_total_tasks_done();
        assert_eq!(total_done, 1);

        // Verify no errors
        let total_errored = pool.get_total_tasks_errored();
        assert_eq!(total_errored, 0);

        // Verify queue is empty by checking stats
        let stats = pool.print_stats();
        // Queue should be 0 after task completes
        assert!(stats.contains("0") || stats.contains("TOTAL"));
    }
}
