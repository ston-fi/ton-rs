use super::*;

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
