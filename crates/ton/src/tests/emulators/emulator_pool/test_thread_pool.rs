use super::*;

struct TestObject(usize);

impl PoolObject for TestObject {
    type Task = usize;
    type Retval = usize;
    fn process<T: Into<Self::Task>>(&mut self, task: T) -> Result<usize, TonError> {
        Ok(self.0 * 1000 + task.into())
    }
    fn descriptor(&self) -> &str {
        "TestObject"
    }
}

#[tokio::test]
async fn test_thread_pool_basic() -> anyhow::Result<()> {
    let objects = vec![TestObject(1), TestObject(2)];

    let pool = ThreadPool::builder(objects)?.build()?;

    let result = pool.exec(42usize, None).await?;

    // Emulator ordering is not guaranteed, so check both possibilities
    assert!(result == 1042 || result == 2042);

    let counter = pool.get_counters_aggregated();
    println!("{:?}", counter);
    assert_eq!(counter.in_progress.load(Ordering::Relaxed), 0);
    assert_eq!(counter.done.load(Ordering::Relaxed), 1);
    assert_eq!(counter.failed.load(Ordering::Relaxed), 0);
    Ok(())
}
