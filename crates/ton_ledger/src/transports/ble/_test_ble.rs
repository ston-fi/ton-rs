use super::*;
use futures_util::stream;

#[test]
fn test_device_lease_excludes_duplicates_until_owner_drops() -> anyhow::Result<()> {
    let first_id = "lease-test-first".to_owned();
    let second_id = "lease-test-second".to_owned();
    let first = DeviceLease::acquire(first_id.clone())?;
    let _second = DeviceLease::acquire(second_id.clone())?;
    assert!(matches!(DeviceLease::acquire(first_id.clone()), Err(TransportError::DeviceBusy)));
    // A rejected acquisition must not release the current owner's lease.
    assert!(matches!(DeviceLease::acquire(first_id.clone()), Err(TransportError::DeviceBusy)));
    drop(first);
    let _reconnected = DeviceLease::acquire(first_id)?;
    assert!(matches!(DeviceLease::acquire(second_id), Err(TransportError::DeviceBusy)));
    Ok(())
}

fn notifications(frames: Vec<Vec<u8>>) -> Notifications {
    Box::pin(stream::iter(frames.into_iter().map(|value| ValueNotification {
        uuid: Uuid::nil(),
        service_uuid: Uuid::nil(),
        value,
    })))
}

#[tokio::test]
async fn test_packet_size_legacy_and_current_firmware() -> anyhow::Result<()> {
    // SDK e9470456, protocol/src/ledger_protocol.c TAG_MTU writes the size
    // both at offsets 2..4 and at offset 5 after the BLE channel is removed.
    for (response, expected) in [
        (vec![8, 0, 0, 0, 1, 20], 20),
        (vec![8, 0, 0, 244, 1, 244], 244),
        (vec![8, 0, 0, 255, 1, 255], 255),
    ] {
        assert_eq!(receive_packet_size(&mut notifications(vec![response]), Uuid::nil()).await?, expected);
    }
    Ok(())
}

#[tokio::test]
async fn test_packet_size_waits_for_negotiation_reply() -> anyhow::Result<()> {
    let mut stream = notifications(vec![vec![], vec![5, 0, 0, 0, 2, 0x90, 0], vec![8, 0, 0, 0, 1, 20]]);
    assert_eq!(receive_packet_size(&mut stream, Uuid::nil()).await?, 20);
    Ok(())
}

#[tokio::test]
async fn test_packet_size_rejects_truncated_invalid_and_disconnected_replies() {
    for response in [
        vec![8],
        vec![8, 0, 0, 0, 1],
        vec![8, 0, 0, 0, 1, 0],
        vec![8, 0, 0, 0, 1, 19],
    ] {
        assert!(matches!(
            receive_packet_size(&mut notifications(vec![response]), Uuid::nil()).await,
            Err(TransportError::Frame(_))
        ));
    }
    assert!(matches!(
        receive_packet_size(&mut notifications(vec![]), Uuid::nil()).await,
        Err(TransportError::Disconnected)
    ));
}

#[tokio::test]
async fn test_disconnect_releases_only_after_backend_confirmation() -> anyhow::Result<()> {
    let id = "ble-confirmed-disconnect".to_owned();
    let mut lease = DeviceLease::acquire(id.clone())?;
    lease.begin_session();
    let (finish, pending) = oneshot::channel();
    let cleanup = tokio::spawn(lease.disconnect(async { pending.await.map_err(|_| TransportError::Disconnected)? }));
    assert!(matches!(DeviceLease::acquire(id.clone()), Err(TransportError::DeviceBusy)));
    finish.send(Ok(())).map_err(|_| anyhow::anyhow!("cleanup stopped"))?;
    cleanup.await?;
    let _reconnected = DeviceLease::acquire(id)?;
    Ok(())
}

#[tokio::test]
async fn test_disconnect_failure_quarantines_device() -> anyhow::Result<()> {
    let id = "ble-failed-disconnect".to_owned();
    let mut lease = DeviceLease::acquire(id.clone())?;
    lease.begin_session();
    lease.disconnect(async { Err(TransportError::Disconnected) }).await;
    assert!(matches!(DeviceLease::acquire(id), Err(TransportError::DeviceBusy)));
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn test_disconnect_timeout_quarantines_device_after_late_completion() -> anyhow::Result<()> {
    let id = "ble-timed-out-disconnect".to_owned();
    let mut lease = DeviceLease::acquire(id.clone())?;
    lease.begin_session();
    let (late_completion, pending) = oneshot::channel::<()>();
    lease.disconnect(async { pending.await.map_err(|_| TransportError::Disconnected) }).await;
    assert!(matches!(DeviceLease::acquire(id.clone()), Err(TransportError::DeviceBusy)));
    // The OS may still complete the queued disconnect after the waiter is gone.
    assert!(late_completion.send(()).is_err());
    assert!(matches!(DeviceLease::acquire(id), Err(TransportError::DeviceBusy)));
    Ok(())
}

#[tokio::test]
async fn test_cancelled_cleanup_keeps_device_quarantined() -> anyhow::Result<()> {
    let id = "ble-cancelled-cleanup".to_owned();
    let mut lease = DeviceLease::acquire(id.clone())?;
    lease.begin_session();
    let cleanup = tokio::spawn(lease.disconnect(std::future::pending()));
    cleanup.abort();
    assert!(cleanup.await.is_err());
    assert!(matches!(DeviceLease::acquire(id), Err(TransportError::DeviceBusy)));
    Ok(())
}
