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
fn get_now_ms() -> u128 { SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros() }

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
    cnt_sended: AtomicUsize,
    cnt_done: AtomicUsize,
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

            cnt_sended: AtomicUsize::new(0),
            cnt_done: AtomicUsize::new(0),
            timeout_emulation,
            max_tasks_in_queue,
            _phantom: std::marker::PhantomData,
        })
    }
    fn increment_id_and_get_index(&self) -> TonResult<usize> {
        let target_queue_index = {
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
                panic!("implement me")
                // return Err(TonError::EmulatorQueueIsFull {
                //     msg: format!("All queues are full, queue_sizes={:?}", q_st),
                //     queue_size: max_queue,
                // });
            }

            target_queue_index
        };

        // This check is redundant for MinQueue (already checked above) but kept for OneByOne safety
        #[allow(clippy::manual_range_contains)]
        if target_queue_index >= self.items.len() {
            return Err(TonError::Custom("Unexpected error".to_string()));
        }

        Ok(target_queue_index)
    }

    pub async fn execute_task(&self, task: Task, maybe_custom_timeout: Option<u64>) -> TonResult<Retval> {
        let deadline_time = if let Some(timeout) = maybe_custom_timeout {
            get_now_ms() + timeout as u128
        } else {
            get_now_ms() + self.timeout_emulation as u128
        };

        let (tx, rx) = oneshot::channel();
        let command = Command::Execute(task, tx, deadline_time);
        let idx = self.increment_id_and_get_index()?;




        // For MinQueue strategy, increment cnt_sended here (OneByOne already increments it in increment_id_and_get_index)

        self.cnt_sended.fetch_add(1, Ordering::Relaxed);



        // Check if queue is full before sending
        let queue_size = self.items[idx].get_queue_size();
        if queue_size >= self.max_tasks_in_queue as usize {
            unreachable!();
        }


        self.items[idx].cnt_in_queue_jobs.fetch_add(1, Ordering::Relaxed);
        self.items[idx].sender.send(command).map_err(|e| TonError::Custom(format!("send task error: {e}")))?;
        let res = rx.await.map_err(|e| TonError::Custom(format!("receive task error: {e}")))??;
        self.cnt_done.fetch_add(1, Ordering::SeqCst);
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
        let total_sended = self.cnt_sended.load(Ordering::Relaxed);
        let total_done_global = self.cnt_done.load(Ordering::Relaxed);

        result.push_str(&format!("{:<10} {:<15} {:<15} {:<15}\n", "TOTAL", total_sended, total_done_global, ""));
        result.push_str(&format!("{:<10} {:<15} {:<15} {:<15}\n", "(sum)", total_in_queue, total_done, ""));
        result.push_str(&"=".repeat(80));
        result.push('\n');

        result
    }
}
