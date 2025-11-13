use crate::bail_ton;
use crate::errors::{TonError, TonResult};
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::OnceLock;
use std::thread;
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
    Execute(T, oneshot::Sender<R>),
    #[allow(dead_code)]
    Stop,
}

struct Inner<Obj, Task, Retval>
where
    Obj: PooledObject<Task, Retval> + Send + 'static,
    Task: Send + 'static,
    Retval: Send + 'static,
{
    senders: Vec<Sender<Command<Task, TonResult<Retval>>>>,
    #[allow(dead_code)]
    workers: Vec<thread::JoinHandle<TonResult<u64>>>,
    cnt_sended: AtomicUsize,
    _phantom: std::marker::PhantomData<Obj>,
}

impl<Obj, Task, Retval> Inner<Obj, Task, Retval>
where
    Obj: PooledObject<Task, Retval> + Send + 'static,
    Task: Send + 'static,
    Retval: Send + 'static,
{
    fn new(mut obj_arr: Vec<Obj>, th_init: Option<fn()>) -> TonResult<Self> {
        if obj_arr.is_empty() {
            bail_ton!("Object array for ThreadPool is empty");
        }

        let mut senders = Vec::new();
        let mut workers = Vec::new();
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

            senders.push(tx);
            workers.push(handle);
        }
        Ok(Self {
            senders,
            workers,
            cnt_sended: AtomicUsize::new(0),
            _phantom: std::marker::PhantomData,
        })
    }
    async fn execute_task(&self, task: Task) -> TonResult<Retval> {
        let (tx, rx) = oneshot::channel();
        let idx = self.cnt_sended.fetch_add(1, Ordering::Relaxed) % self.senders.len();
        self.senders[idx]
            .send(Command::Execute(task, tx))
            .map_err(|e| TonError::Custom(format!("send task error: {e}")))?;
        let res = rx.await.map_err(|e| TonError::Custom(format!("receive task error: {e}")))??;
        // self.c_completed.fetch_add(1, Ordering::SeqCst);
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
    pub fn new(obj_arr: Vec<Obj>, th_init: Option<fn()>) -> TonResult<Self> {
        let inner = Inner::new(obj_arr, th_init)?;
        let pool = Self { inner: OnceLock::new() };
        pool.inner.set(inner).map_err(|_| TonError::Custom("Failed to initialize ThreadPool".to_string()))?;
        Ok(pool)
    }

    pub async fn execute_task(&self, task: Task) -> TonResult<Retval> {
        let inner = self.inner.get().ok_or(TonError::Custom("Inner ThreadPool not initialized".to_string()))?;
        inner.execute_task(task).await
    }
}
