use std::sync::atomic::{AtomicUsize, Ordering};

pub(crate) struct JobsCounter<'a>(&'a AtomicUsize);

impl<'a> JobsCounter<'a> {
    pub(crate) fn new(counter: &'a AtomicUsize) -> Self {
        counter.fetch_add(1, Ordering::Relaxed);
        JobsCounter(counter)
    }
}

impl<'a> Drop for JobsCounter<'a> {
    fn drop(&mut self) { self.0.fetch_sub(1, Ordering::Relaxed); }
}
