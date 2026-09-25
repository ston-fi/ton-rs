use super::*;

struct WorkerLifetime(mpsc::Sender<()>);
impl Drop for WorkerLifetime {
    fn drop(&mut self) {
        let _ = self.0.send(());
    }
}

#[test]
fn test_api_worker_survives_runtime_shutdown() -> anyhow::Result<()> {
    let (stopped, shutdown) = mpsc::channel();
    let lifetime = WorkerLifetime(stopped);
    let (thread, threads) = mpsc::channel();
    let worker = spawn_api_worker(move || {
        let _keep_alive = &lifetime;
        let _ = thread.send(std::thread::current().id());
        // Exercise a backend failure without initializing the process-global
        // native HIDAPI context on this disposable test worker.
        Err(TransportError::NoDevice)
    })?;
    let mut ids = Vec::new();
    for _ in 0..2 {
        let runtime = tokio::runtime::Builder::new_current_thread().enable_time().build()?;
        let (reply, result) = oneshot::channel();
        worker.send(reply)?;
        let result = runtime.block_on(async { tokio::time::timeout(Duration::from_secs(5), result).await })??;
        assert!(matches!(result, Err(TransportError::NoDevice)));
        ids.push(threads.recv_timeout(Duration::from_secs(5))?);
        // Runtime shutdown is stronger than waiting for its blocking threads
        // to retire: the HID initialization thread must survive both.
        drop(runtime);
        assert!(matches!(shutdown.try_recv(), Err(mpsc::TryRecvError::Empty)));
    }
    assert_eq!(ids[0], ids[1]);
    assert_ne!(ids[0], std::thread::current().id());
    drop(worker);
    shutdown.recv_timeout(Duration::from_secs(5))?;
    Ok(())
}

#[test]
fn test_api_worker_skips_cancelled_requests_and_survives_inflight_timeout() -> anyhow::Result<()> {
    let (entered, started) = mpsc::channel();
    let (release, resume) = mpsc::channel();
    let mut calls = 0;
    let worker = spawn_api_worker(move || {
        calls += 1;
        let _ = entered.send(calls);
        if calls == 1 {
            let _ = resume.recv_timeout(Duration::from_secs(5));
        }
        Err(TransportError::NoDevice)
    })?;
    let runtime = tokio::runtime::Builder::new_current_thread().enable_time().build()?;
    let (reply, result) = oneshot::channel();
    worker.send(reply)?;
    assert_eq!(started.recv_timeout(Duration::from_secs(5))?, 1);
    assert!(runtime.block_on(async { tokio::time::timeout(Duration::ZERO, result).await }).is_err());

    let (cancelled, result) = oneshot::channel();
    drop(result);
    worker.send(cancelled)?;
    let (reply, result) = oneshot::channel();
    worker.send(reply)?;
    release.send(())?;
    let result = runtime.block_on(async { tokio::time::timeout(Duration::from_secs(5), result).await })??;
    assert!(matches!(result, Err(TransportError::NoDevice)));
    assert_eq!(started.recv_timeout(Duration::from_secs(5))?, 2);
    assert!(matches!(started.try_recv(), Err(mpsc::TryRecvError::Empty)));
    Ok(())
}

#[tokio::test]
async fn test_connect_rejects_duplicate_backend_path_despite_changed_display_id() -> anyhow::Result<()> {
    let device = HidDeviceInfo {
        id: "original label".into(),
        product: None,
        path: CString::new("hid-duplicate-test")?,
    };
    let lease = DeviceLease::acquire(device.path.clone())?;
    let mut rediscovered = device.clone();
    rediscovered.id = "different label".into();
    for descriptor in [device.clone(), rediscovered] {
        assert!(matches!(HidTransport::connect(descriptor).await, Err(TransportError::DeviceBusy)));
    }
    // Rejected connections must not release the first owner's lease.
    assert!(matches!(DeviceLease::acquire(device.path.clone()), Err(TransportError::DeviceBusy)));
    drop(lease);
    let _reconnected = DeviceLease::acquire(device.path)?;
    Ok(())
}

#[test]
fn test_worker_holds_lease_through_handle_cleanup() -> anyhow::Result<()> {
    struct BlockingClose {
        started: mpsc::Sender<()>,
        finish: mpsc::Receiver<()>,
    }
    impl Drop for BlockingClose {
        fn drop(&mut self) {
            let _ = self.started.send(());
            let _ = self.finish.recv_timeout(Duration::from_secs(5));
        }
    }
    let path = CString::new("hid-worker-cleanup-test")?;
    let (started, closing) = mpsc::channel();
    let (finish, release) = mpsc::channel();
    let handle = BlockingClose {
        started,
        finish: release,
    };
    let worker = spawn_worker(path.clone(), move || drop(handle))?;
    closing.recv_timeout(Duration::from_secs(5))?;
    let duplicate = DeviceLease::acquire(path.clone());
    // Always unblock and join before asserting, even if ownership regresses.
    finish.send(())?;
    worker.join().map_err(|_| anyhow::anyhow!("HID worker panicked"))?;
    assert!(matches!(duplicate, Err(TransportError::DeviceBusy)));
    let _reconnected = DeviceLease::acquire(path)?;
    Ok(())
}

#[test]
fn test_worker_releases_lease_after_setup_failure() -> anyhow::Result<()> {
    let path = CString::new("hid-worker-failure-test")?;
    let (result, opened) = oneshot::channel::<Result<(), TransportError>>();
    // A cancelled connection no longer receives the worker's setup result.
    drop(opened);
    let worker = spawn_worker(path.clone(), move || {
        let _ = result.send(Err(TransportError::Disconnected));
    })?;
    worker.join().map_err(|_| anyhow::anyhow!("HID worker panicked"))?;
    let _reconnected = DeviceLease::acquire(path)?;
    Ok(())
}
