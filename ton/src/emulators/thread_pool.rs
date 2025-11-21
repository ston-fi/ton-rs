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

        let num_threads = obj_arr.len();
        let mut senders = Vec::with_capacity(num_threads);
        let mut thread_handles = Vec::with_capacity(num_threads);
        let mut cnt_jobs_in_queue = Vec::with_capacity(num_threads);

        for _ in 0..num_threads {
            let (tx, rx): CommandChannel<Task, TonResult<Retval>> = mpsc::channel();
            let obj = obj_arr.pop().ok_or(TonError::Custom("Not enough pooled objects for threads".to_string()))?;

            let handle = thread::spawn(move || {
                if let Some(init_fn) = th_init {
                    init_fn();
                }
                Self::worker_loop(obj, rx)
            });
            senders.push(tx);
            thread_handles.push(handle);
            cnt_jobs_in_queue.push(AtomicUsize::new(0));
        }
        Ok(Self {
            senders,
            thread_handles,
            cnt_jobs_in_queue,
            cnt_current_jobs: AtomicUsize::new(0),
            timeout_emulation,
            max_tasks_in_queue,
            _phantom: std::marker::PhantomData,
        })
    }
    async fn increment_id_and_get_index(&self, deadline: u128) -> TonResult<usize> {
        loop {
            //  find thread with minimum queue size
            let mut min_queue_value = self.max_tasks_in_queue as usize + 1;
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
                if get_now_ms() > deadline {
                    return Err(TonError::EmulatorPoolTimeout {
                        deadline_time: deadline,
                    });
                }
                continue;
            }
            // do  increment asap
            self.cnt_jobs_in_queue[target_queue_index].fetch_add(1, Ordering::Relaxed);

            return Ok(target_queue_index);
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
        let _guard = DecrementOnDestructor::new(&self.cnt_jobs_in_queue[idx]);
        self.cnt_current_jobs.fetch_add(1, Ordering::Relaxed);

        self.senders[idx].send(command).map_err(|e| TonError::Custom(format!("send task error: {e}")))?;
        let res = rx.await.map_err(|e| TonError::Custom(format!("receive task error: {e}")))??;

        self.cnt_current_jobs.fetch_sub(1, Ordering::Relaxed);

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
        result.push_str(&format!("{:<10} {:<15} {:<15}\n", "Thread", "In Queue", "Queue Size"));
        result.push_str(&"-".repeat(80));
        result.push('\n');

        // Collect per-thread statistics
        let mut total_in_queue = 0;

        for idx in 0..self.senders.len() {
            let queue_size = self.cnt_jobs_in_queue[idx].load(Ordering::Relaxed);

            total_in_queue += queue_size;

            result.push_str(&format!("{:<10} {:<15} {:<15}\n", idx, queue_size, queue_size));
        }

        // Total row
        result.push_str(&"-".repeat(80));
        result.push('\n');
        let current_jobs = self.cnt_current_jobs.load(Ordering::Relaxed);
        let total_queue_size: usize = self.cnt_jobs_in_queue.iter().map(|cnt| cnt.load(Ordering::Relaxed)).sum();

        result.push_str(&format!("{:<10} {:<15} {:<15}\n", "TOTAL", total_in_queue, total_queue_size));
        result.push_str(&format!("{:<10} {:<15} {:<15}\n", "(current)", current_jobs, ""));
        result.push_str(&"=".repeat(80));
        result.push('\n');

        result
    }

    /// Get total number of tasks completed - not tracked anymore, returns 0
    pub fn get_total_tasks_done(&self) -> usize { 0 }
}

struct DecrementOnDestructor<'a> {
    cnt: &'a AtomicUsize,
}

impl<'a> DecrementOnDestructor<'a> {
    fn new(cnt: &'a AtomicUsize) -> Self { DecrementOnDestructor { cnt } }
}

impl<'a> Drop for DecrementOnDestructor<'a> {
    fn drop(&mut self) {
        // Only decrement if task failed (set_success was not called)

        self.cnt.fetch_sub(1, Ordering::Relaxed);
    }
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
}
