use crate::bail_ton;
use crate::errors::{TonError, TonResult};
use std::sync::atomic::Ordering;
use std::sync::atomic::{AtomicU16, AtomicUsize};
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
        let mut cnt_in_queue_jobs = Vec::with_capacity(num_threads);
        let mut cnt_done_jobs = Vec::with_capacity(num_threads);

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
            cnt_in_queue_jobs.push(AtomicU16::new(0));
            cnt_done_jobs.push(AtomicU16::new(0));
        }
        Ok(Self {
            senders,
            thread_handles,
            cnt_in_queue_jobs,
            cnt_done_jobs,
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
        result.push_str(&format!("{:<10} {:<15} {:<15} {:<15}\n", "Thread", "In Queue", "Done Jobs", "Queue Size"));
        result.push_str(&"-".repeat(80));
        result.push('\n');

        // Collect per-thread statistics
        let mut total_in_queue = 0;


        for idx in 0..self.senders.len() {
            let in_queue = self.cnt_in_queue_jobs[idx].load(Ordering::Relaxed);
            let done_jobs = self.cnt_done_jobs[idx].load(Ordering::Relaxed);
            let queue_size = get_queue_size(&self.cnt_in_queue_jobs[idx], &self.cnt_done_jobs[idx]);

            total_in_queue += in_queue;


            result.push_str(&format!("{:<10} {:<15} {:<15} {:<15}\n", idx, in_queue, done_jobs, queue_size));
        }

        // Total row
        result.push_str(&"-".repeat(80));
        result.push('\n');
        let current_jobs = self.cnt_current_jobs.load(Ordering::Relaxed);
        let total_queue_size: usize = (0..self.senders.len())
            .map(|i| get_queue_size(&self.cnt_in_queue_jobs[i], &self.cnt_done_jobs[i]))
            .sum();

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
        self.cnt_done_jobs.iter().map(|cnt| cnt.load(Ordering::Relaxed) as usize).sum()
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
    use std::sync::mpsc::channel;
    use std::thread;

    #[test]
    fn test_get_queue_size_overflow() {
        // Create counters for testing
        let cnt_in_queue_jobs = AtomicU16::new(0);
        let cnt_done_jobs = AtomicU16::new(0);

        // Test normal case: in_queue > done_jobs
        cnt_in_queue_jobs.store(100, Ordering::Relaxed);
        cnt_done_jobs.store(50, Ordering::Relaxed);
        assert_eq!(get_queue_size(&cnt_in_queue_jobs, &cnt_done_jobs), 50);

        // Test edge case: in_queue == done_jobs
        cnt_in_queue_jobs.store(100, Ordering::Relaxed);
        cnt_done_jobs.store(100, Ordering::Relaxed);
        assert_eq!(get_queue_size(&cnt_in_queue_jobs, &cnt_done_jobs), 0);

        // Test overflow case: in_queue wraps around to 0, done_jobs is high
        // This simulates counter overflow where in_queue wrapped from 65535 to 0
        // in_queue = 10 (after wraparound), done_jobs = 65500 (before wraparound)
        // Expected: (65536 - 65500) + 10 = 36 + 10 = 46
        cnt_in_queue_jobs.store(10, Ordering::Relaxed);
        cnt_done_jobs.store(65500, Ordering::Relaxed);
        assert_eq!(get_queue_size(&cnt_in_queue_jobs, &cnt_done_jobs), 46);

        // Test another overflow case: done_jobs wraps around
        cnt_in_queue_jobs.store(65500, Ordering::Relaxed);
        cnt_done_jobs.store(10, Ordering::Relaxed);
        // Normal case: in_queue > done_jobs, so 65500 - 10 = 65490
        assert_eq!(get_queue_size(&cnt_in_queue_jobs, &cnt_done_jobs), 65490);

        // Test max values
        cnt_in_queue_jobs.store(u16::MAX, Ordering::Relaxed);
        cnt_done_jobs.store(0, Ordering::Relaxed);
        assert_eq!(get_queue_size(&cnt_in_queue_jobs, &cnt_done_jobs), u16::MAX as usize);

        // Test both at max
        cnt_in_queue_jobs.store(u16::MAX, Ordering::Relaxed);
        cnt_done_jobs.store(u16::MAX, Ordering::Relaxed);
        assert_eq!(get_queue_size(&cnt_in_queue_jobs, &cnt_done_jobs), 0);

        // Test where done exceeds in_queue after wraparound
        // in_queue = 100, done_jobs = 200
        // Expected: (65536 - 200) + 100 = 65436 + 100 = 65536
        // But this represents wraparound, so actual queue might be less
        cnt_in_queue_jobs.store(100, Ordering::Relaxed);
        cnt_done_jobs.store(200, Ordering::Relaxed);
        assert_eq!(get_queue_size(&cnt_in_queue_jobs, &cnt_done_jobs), 65436);

        // Test wraparound with specific values
        // in_queue wrapped: value is 5, done_jobs is 65530
        // Expected: (65536 - 65530) + 5 = 6 + 5 = 11
        cnt_in_queue_jobs.store(5, Ordering::Relaxed);
        cnt_done_jobs.store(65530, Ordering::Relaxed);
        assert_eq!(get_queue_size(&cnt_in_queue_jobs, &cnt_done_jobs), 11);

        // Test edge case: in_queue wrapped to 0, done_jobs is 65535
        cnt_in_queue_jobs.store(0, Ordering::Relaxed);
        cnt_done_jobs.store(65535, Ordering::Relaxed);
        // Expected: (65536 - 65535) + 0 = 1 + 0 = 1
        assert_eq!(get_queue_size(&cnt_in_queue_jobs, &cnt_done_jobs), 1);
    }
}
