use crate::bail_ton;
use crate::errors::{TonError, TonResult};
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::OnceLock;
use std::thread;
use std::thread::JoinHandle;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::oneshot;

/// A command sent to worker threads.
pub trait PooledObject<T: Send, R: Send> {
    fn handle(&mut self, task: T) -> Result<R, TonError>;
}

type CommandChannel<T, R> = (Sender<Command<T, R>>, Receiver<Command<T, R>>);

enum Command<T, R>
where
    T: Send,
    R: Send,
{
    Execute(T, oneshot::Sender<R>, u128, u64), // task, sender, deadline_time, timeout
    #[allow(dead_code)]
    Stop,
}

struct ThreadItem<Task, Retval>
where
    Task: Send + 'static,
    Retval: Send + 'static,
{
    sender: Sender<Command<Task, TonResult<Retval>>>,
    #[allow(dead_code)]
    thread: JoinHandle<TonResult<u64>>,

    cnd_in_queue_jobs: AtomicUsize,
    cnd_done_jobs: AtomicUsize,
}
impl<Task, Retval> ThreadItem<Task, Retval>
where
    Task: Send + 'static,
    Retval: Send + 'static,
{
    fn get_queue_size(&self) -> usize {
        let in_queue = self.cnd_in_queue_jobs.load(Ordering::Relaxed);
        let done_jbs = self.cnd_done_jobs.load(Ordering::Relaxed);
        if in_queue < done_jbs {
            0
        } else {
            in_queue - done_jbs
        }
    }
}
fn get_now_ms() -> u128 { SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros() }

pub enum PoolPickStrategy {
    OneByOne,
    MinQueue,
}

impl<Task, Retval> ThreadItem<Task, Retval>
where
    Task: Send + 'static,
    Retval: Send + 'static,
{
    fn new(sender: Sender<Command<Task, TonResult<Retval>>>, thread: JoinHandle<TonResult<u64>>) -> Self {
        Self {
            sender,
            thread,
            cnd_in_queue_jobs: AtomicUsize::new(0),
            cnd_done_jobs: AtomicUsize::new(0),
        }
    }
}

struct Inner<Obj, Task, Retval>
where
    Obj: PooledObject<Task, Retval> + Send + 'static,
    Task: Send + 'static,
    Retval: Send + 'static,
{
    items: Vec<ThreadItem<Task, Retval>>,
    cnt_sended: AtomicUsize,
    cnt_done: AtomicUsize,
    mode: PoolPickStrategy,
    timeout_emulation: u64,
    max_tasks_in_queue: u32,
    _phantom: std::marker::PhantomData<Obj>,
}

impl<Obj, Task, Retval> Inner<Obj, Task, Retval>
where
    Obj: PooledObject<Task, Retval> + Send + 'static,
    Task: Send + 'static,
    Retval: Send + 'static,
{
    fn increment_id_and_get_index(&self) -> TonResult<usize> {
        let target_queue_index = match self.mode {
            PoolPickStrategy::OneByOne => {
                // For OneByOne, increment atomically to get unique sequence number for round-robin
                let curr_id = self.cnt_sended.fetch_add(1, Ordering::Relaxed);
                curr_id % self.items.len()
            }
            PoolPickStrategy::MinQueue => {
                // For MinQueue, find thread with minimum queue size
                let mut min_queue_value = self.max_tasks_in_queue as usize;
                let mut target_queue_index = self.items.len(); // set bad index
                for i in 0..self.items.len() {
                    let current_queue_size = self.items[i].get_queue_size();
                    if current_queue_size == 0 {
                        target_queue_index = i;
                        break;
                    }
                    if current_queue_size < min_queue_value {
                        min_queue_value = current_queue_size;
                        target_queue_index = i;
                    }
                }
                if target_queue_index == self.items.len() {
                    let q_st: Vec<usize> = (0..self.items.len()).map(|i| self.items[i].get_queue_size()).collect();
                    let max_queue = q_st.iter().max().copied().unwrap_or(0);
                    return Err(TonError::EmulatorQueueIsFull {
                        msg: format!("All queues are full, queue_sizes={:?}", q_st),
                        queue_size: max_queue,
                    });
                }

                target_queue_index
            }
        };

        // This check is redundant for MinQueue (already checked above) but kept for OneByOne safety
        #[allow(clippy::manual_range_contains)]
        if target_queue_index >= self.items.len() {
            return Err(TonError::Custom("Unexpected error".to_string()));
        }

        Ok(target_queue_index)
    }

    fn new(
        mut obj_arr: Vec<Obj>,
        pick_strategy: PoolPickStrategy,
        th_init: Option<fn()>,
        timeout_emulation: u64,
        max_tasks_in_queue: u32,
    ) -> TonResult<Self> {
        if obj_arr.is_empty() {
            bail_ton!("Object array for ThreadPool is empty");
        }

        let mut items = Vec::new();

        let num_threads = obj_arr.len();

        for _ in 0..num_threads {
            let (tx, rx): CommandChannel<Task, TonResult<Retval>> = mpsc::channel();
            let obj = obj_arr.pop().ok_or(TonError::Custom("Not enough pooled objects for threads".to_string()))?;

            let handle = thread::spawn(move || {
                if let Some(init_fn) = th_init {
                    init_fn();
                }
                Self::worker_loop(obj, rx)
            });
            items.push(ThreadItem::new(tx, handle));
        }
        Ok(Self {
            items,

            cnt_sended: AtomicUsize::new(0),
            cnt_done: AtomicUsize::new(0),
            mode: pick_strategy,
            timeout_emulation,
            max_tasks_in_queue,
            _phantom: std::marker::PhantomData,
        })
    }
    async fn execute_task(&self, task: Task) -> TonResult<Retval> {
        let (tx, rx) = oneshot::channel();

        let idx = self.increment_id_and_get_index()?;

        // For MinQueue strategy, increment cnt_sended here (OneByOne already increments it in increment_id_and_get_index)
        if matches!(self.mode, PoolPickStrategy::MinQueue) {
            self.cnt_sended.fetch_add(1, Ordering::Relaxed);
        }
        let deadline_time = get_now_ms() + self.timeout_emulation as u128;

        // Check if queue is full before sending
        let queue_size = self.items[idx].get_queue_size();
        if queue_size >= self.max_tasks_in_queue as usize {
            return Err(TonError::EmulatorQueueIsFull {
                msg: format!("Thread {} queue is full", idx),
                queue_size,
            });
        }

        let command = Command::Execute(task, tx, deadline_time, self.timeout_emulation);
        self.items[idx].cnd_in_queue_jobs.fetch_add(1, Ordering::Relaxed);
        self.items[idx].sender.send(command).map_err(|e| TonError::Custom(format!("send task error: {e}")))?;
        let res = rx.await.map_err(|e| TonError::Custom(format!("receive task error: {e}")))??;
        self.cnt_done.fetch_add(1, Ordering::SeqCst);
        self.items[idx].cnd_done_jobs.fetch_add(1, Ordering::Relaxed);
        Ok(res)
    }

    fn worker_loop(mut obj: Obj, receiver: Receiver<Command<Task, TonResult<Retval>>>) -> TonResult<u64> {
        let mut counter = 0;
        loop {
            let command = receiver.recv();
            counter += 1;
            match command {
                Ok(Command::Execute(task, resp_sender, deadline_time, timeout)) => {
                    // Check if deadline has passed
                    let current_time = get_now_ms();
                    if current_time > deadline_time {
                        let _ = resp_sender.send(Err(TonError::EmulatorTimeout { current_time, timeout }));
                        continue;
                    }
                    let result = obj.handle(task)?;
                    let _ = resp_sender.send(Ok(result));
                }
                Ok(Command::Stop) => break,
                Err(_) => break,
            }
        }
        Ok(counter)
    }
}

pub struct ThreadPool<Obj, Task, Retval>
where
    Obj: PooledObject<Task, Retval> + Send + 'static,
    Task: Send + 'static,
    Retval: Send + 'static,
{
    inner: OnceLock<Inner<Obj, Task, Retval>>,
}

impl<Obj, Task, Retval> ThreadPool<Obj, Task, Retval>
where
    Obj: PooledObject<Task, Retval> + Send + 'static,
    Task: Send + 'static,
    Retval: Send + 'static,
{
    pub fn new(
        obj_arr: Vec<Obj>,
        pick_strategy: PoolPickStrategy,
        th_init: Option<fn()>,
        timeout_emulation: u64,
        max_tasks_in_queue: u32,
    ) -> TonResult<Self> {
        let inner = Inner::new(obj_arr, pick_strategy, th_init, timeout_emulation, max_tasks_in_queue)?;
        let pool = Self { inner: OnceLock::new() };
        pool.inner.set(inner).map_err(|_| TonError::Custom("Failed to initialize ThreadPool".to_string()))?;
        Ok(pool)
    }

    pub async fn execute_task(&self, task: Task) -> TonResult<Retval> {
        let inner = self.inner.get().ok_or(TonError::Custom("Inner ThreadPool not initialized".to_string()))?;
        inner.execute_task(task).await
    }

    pub fn print_stat(&self) -> String {
        let inner = match self.inner.get() {
            Some(inner) => inner,
            None => return "ThreadPool not initialized".to_string(),
        };

        let mut result = String::new();

        // Table header
        result.push_str("ThreadPool Statistics\n");
        result.push_str(&"=".repeat(80));
        result.push('\n');
        result.push_str(&format!("{:<10} {:<15} {:<15} {:<15}\n", "Thread", "In Queue", "Done Jobs", "Queue Size"));
        result.push_str(&"-".repeat(80));
        result.push('\n');

        // Collect per-thread statistics
        let mut total_in_queue = 0;
        let mut total_done = 0;

        for (idx, item) in inner.items.iter().enumerate() {
            let in_queue = item.cnd_in_queue_jobs.load(Ordering::Relaxed);
            let done_jobs = item.cnd_done_jobs.load(Ordering::Relaxed);
            let queue_size = item.get_queue_size();

            total_in_queue += in_queue;
            total_done += done_jobs;

            result.push_str(&format!("{:<10} {:<15} {:<15} {:<15}\n", idx, in_queue, done_jobs, queue_size));
        }

        // Total row
        result.push_str(&"-".repeat(80));
        result.push('\n');
        let total_sended = inner.cnt_sended.load(Ordering::Relaxed);
        let total_done_global = inner.cnt_done.load(Ordering::Relaxed);

        result.push_str(&format!("{:<10} {:<15} {:<15} {:<15}\n", "TOTAL", total_sended, total_done_global, ""));
        result.push_str(&format!("{:<10} {:<15} {:<15} {:<15}\n", "(sum)", total_in_queue, total_done, ""));
        result.push_str(&"=".repeat(80));
        result.push('\n');

        result
    }
}
