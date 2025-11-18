use crate::bail_ton;
use crate::errors::{TonError, TonResult};
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::OnceLock;
use std::thread;
use std::thread::JoinHandle;
use tokio::sync::oneshot;

const MAX_QUEUE_SIZE: usize = 10;
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
    Execute(T, oneshot::Sender<R>),
    #[allow(dead_code)]
    Stop,
}

struct Params {
    timeout_emulation: u64,  //
    max_tasks_in_queue: u32, //
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
        self.cnd_in_queue_jobs.load(Ordering::Relaxed) - self.cnd_done_jobs.load(Ordering::Relaxed)
    }
}

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
    _phantom: std::marker::PhantomData<Obj>,
}

impl<Obj, Task, Retval> Inner<Obj, Task, Retval>
where
    Obj: PooledObject<Task, Retval> + Send + 'static,
    Task: Send + 'static,
    Retval: Send + 'static,
{
    fn increment_id_and_get_index(&self) -> usize {
        let rv = match self.mode {
            PoolPickStrategy::OneByOne => {
                let curr_id = self.cnt_sended.fetch_add(1, Ordering::Relaxed);
                curr_id % self.items.len()
            }
            PoolPickStrategy::MinQueue => {
                let mut rv = MAX_QUEUE_SIZE;
                let mut answer_id = self.items.len();
                for i in 0..self.items.len() {
                    let qs = self.items[i].get_queue_size();
                    if qs == 0 {
                        rv = 0;
                        answer_id = i;
                        break;
                    } else {
                        panic!("WOOW");
                        if rv < qs {
                            rv = qs;
                            answer_id = i;
                        }
                    }
                }
                if answer_id == self.items.len() {
                    panic!("NO FREE QUEUE")
                }
                answer_id
            }
        };

        if rv >= self.items.len() {
            panic!("wrond logic");
        }

        rv
    }

    fn new(mut obj_arr: Vec<Obj>, pick_strategy: PoolPickStrategy, th_init: Option<fn()>) -> TonResult<Self> {
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
            _phantom: std::marker::PhantomData,
        })
    }
    async fn execute_task(&self, task: Task) -> TonResult<Retval> {
        let (tx, rx) = oneshot::channel();

        let idx = self.increment_id_and_get_index();

        self.items[idx].cnd_in_queue_jobs.fetch_add(1, Ordering::Relaxed);
        self.items[idx]
            .sender
            .send(Command::Execute(task, tx))
            .map_err(|e| TonError::Custom(format!("send task error: {e}")))?;
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
                Ok(Command::Execute(task, resp_sender)) => {
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
    pub fn new(obj_arr: Vec<Obj>, pick_strategy: PoolPickStrategy, th_init: Option<fn()>) -> TonResult<Self> {
        let inner = Inner::new(obj_arr, pick_strategy, th_init)?;
        let pool = Self { inner: OnceLock::new() };
        pool.inner.set(inner).map_err(|_| TonError::Custom("Failed to initialize ThreadPool".to_string()))?;
        Ok(pool)
    }

    pub async fn execute_task(&self, task: Task) -> TonResult<Retval> {
        let inner = self.inner.get().ok_or(TonError::Custom("Inner ThreadPool not initialized".to_string()))?;
        inner.execute_task(task).await
    }
}
