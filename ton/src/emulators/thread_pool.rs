use crate::bail_ton;
use crate::errors::{TonError, TonResult};
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::OnceLock;
use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::oneshot;
use tokio::time::sleep;

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
    Execute(T, oneshot::Sender<R>, u128), // task, sender, timeout
}

struct ThreadItem<Task, Retval>
where
    Task: Send + 'static,
    Retval: Send + 'static,
{
    sender: Sender<Command<Task, TonResult<Retval>>>,
    #[allow(dead_code)]
    thread: JoinHandle<TonResult<u64>>,

    cnt_in_queue_jobs: AtomicUsize,
    cnt_done_jobs: AtomicUsize,
}
impl<Task, Retval> ThreadItem<Task, Retval>
where
    Task: Send + 'static,
    Retval: Send + 'static,
{
    fn get_queue_size(&self) -> usize {
        let in_queue = self.cnt_in_queue_jobs.load(Ordering::Relaxed);
        let done_jbs = self.cnt_done_jobs.load(Ordering::Relaxed);
        if in_queue < done_jbs {
            0
        } else {
            in_queue - done_jbs
        }
    }
}
fn get_now_ms() -> u128 { SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() }

impl<Task, Retval> ThreadItem<Task, Retval>
where
    Task: Send + 'static,
    Retval: Send + 'static,
{
    fn new(sender: Sender<Command<Task, TonResult<Retval>>>, thread: JoinHandle<TonResult<u64>>) -> Self {
        Self {
            sender,
            thread,
            cnt_in_queue_jobs: AtomicUsize::new(0),
            cnt_done_jobs: AtomicUsize::new(0),
        }
    }
}

pub struct ThreadPool<Obj, Task, Retval>
where
    Obj: PooledObject<Task, Retval> + Send + 'static,
    Task: Send + 'static,
    Retval: Send + 'static,
{
    items: Vec<ThreadItem<Task, Retval>>,
    cnt_current_jobs: AtomicUsize,
    timeout_emulation: u64,
    max_tasks_in_queue: u32,
    _phantom: std::marker::PhantomData<Obj>,
}

impl<Obj, Task, Retval> ThreadPool<Obj, Task, Retval>
where
    Obj: PooledObject<Task, Retval> + Send + 'static,
    Task: Send + 'static,
    Retval: Send + 'static,
{
    pub fn new(
        mut obj_arr: Vec<Obj>,
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
            cnt_current_jobs: AtomicUsize::new(0),
            timeout_emulation,
            max_tasks_in_queue,
            _phantom: std::marker::PhantomData,
        })
    }
    async fn increment_id_and_get_index(&self, deadline: u128) -> TonResult<usize> {
        loop {
            let target_queue_index = {
                //  find thread with minimum queue size
                let mut min_queue_value = self.max_tasks_in_queue as usize + 1;
                let mut target_queue_index = self.items.len(); // set bad index
                for i in 0..self.items.len() {
                    let current_queue_size = self.items[i].get_queue_size();
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

                if target_queue_index == self.items.len() {
                    sleep(Duration::from_millis(1));
                    if (get_now_ms() > deadline) {}
                    continue;
                }
                // do  increment asap
                self.items[target_queue_index].cnt_in_queue_jobs.fetch_add(1, Ordering::Relaxed);

                return Ok(target_queue_index);
            };
        }
    }

    pub async fn execute_task(&self, task: Task, maybe_custom_timeout: Option<u64>) -> TonResult<Retval> {
        let deadline_time = if let Some(timeout) = maybe_custom_timeout {
            get_now_ms() + timeout as u128
        } else {
            get_now_ms() + self.timeout_emulation as u128
        };

        let (tx, rx) = oneshot::channel();
        let command = Command::Execute(task, tx, deadline_time);
        let idx = self.increment_id_and_get_index(deadline_time).await?;

        self.cnt_current_jobs.fetch_add(1, Ordering::Relaxed);

        self.items[idx].sender.send(command).map_err(|e| TonError::Custom(format!("send task error: {e}")))?;
        let res = rx.await.map_err(|e| TonError::Custom(format!("receive task error: {e}")))??;
        self.cnt_current_jobs.fetch_sub(1, Ordering::SeqCst);
        self.items[idx].cnt_done_jobs.fetch_add(1, Ordering::Relaxed);
        Ok(res)
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
        result.push_str(&"=".repeat(80));
        result.push('\n');
        result.push_str(&format!("{:<10} {:<15} {:<15} {:<15}\n", "Thread", "In Queue", "Done Jobs", "Queue Size"));
        result.push_str(&"-".repeat(80));
        result.push('\n');

        // Collect per-thread statistics
        let mut total_in_queue = 0;
        let mut total_done = 0;

        for (idx, item) in self.items.iter().enumerate() {
            let in_queue = item.cnt_in_queue_jobs.load(Ordering::Relaxed);
            let done_jobs = item.cnt_done_jobs.load(Ordering::Relaxed);
            let queue_size = item.get_queue_size();

            total_in_queue += in_queue;
            total_done += done_jobs;

            result.push_str(&format!("{:<10} {:<15} {:<15} {:<15}\n", idx, in_queue, done_jobs, queue_size));
        }

        // Total row
        result.push_str(&"-".repeat(80));
        result.push('\n');
        let current_jobs = self.cnt_current_jobs.load(Ordering::Relaxed);
        let total_queue_size: usize = self.items.iter().map(|item| item.get_queue_size()).sum();
        // total_done is already calculated above by summing all per-core cnt_done_jobs counters
        let total_tasks_done = self.get_total_tasks_done();

        result.push_str(&format!(
            "{:<10} {:<15} {:<15} {:<15}\n",
            "TOTAL", total_in_queue, total_tasks_done, total_queue_size
        ));
        result.push_str(&format!("{:<10} {:<15} {:<15} {:<15}\n", "(current)", current_jobs, "", ""));
        result.push_str(&"=".repeat(80));
        result.push('\n');

        result
    }

    /// Get total number of tasks completed by summing all per-core done counters
    pub fn get_total_tasks_done(&self) -> usize {
        self.items.iter().map(|item| item.cnt_done_jobs.load(Ordering::Relaxed)).sum()
    }
}
