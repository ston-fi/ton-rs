use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

#[derive(Default, Debug)]
pub struct TaskCounter {
    pub in_progress: AtomicUsize,
    pub done: AtomicUsize,
    pub failed: AtomicUsize,
}

impl TaskCounter {
    pub(crate) fn new() -> Self { Self::default() }

    pub(crate) fn task_added(&'_ self) -> TaskCounterUpdater<'_> { TaskCounterUpdater::new(self) }
}

/// mark_done() must be called to mark task as successfully completed
/// Otherwise, Drop will mark task as failed
pub(crate) struct TaskCounterUpdater<'a> {
    counter: &'a TaskCounter,
    is_done: AtomicBool,
}

impl<'a> TaskCounterUpdater<'a> {
    pub(crate) fn new(counter: &'a TaskCounter) -> Self {
        counter.in_progress.fetch_add(1, Ordering::Relaxed);
        Self {
            counter,
            is_done: AtomicBool::new(false),
        }
    }

    pub(crate) fn task_done(self) {
        self.counter.in_progress.fetch_sub(1, Ordering::Relaxed);
        self.counter.done.fetch_add(1, Ordering::Relaxed);
        self.is_done.store(true, Ordering::Relaxed);
        // std::mem::forget(self); // TODO research how it works
    }
}

impl<'a> Drop for TaskCounterUpdater<'a> {
    fn drop(&mut self) {
        if self.is_done.load(Ordering::Relaxed) {
            return;
        }
        self.counter.in_progress.fetch_sub(1, Ordering::Relaxed);
        self.counter.failed.fetch_add(1, Ordering::Relaxed);
    }
}
