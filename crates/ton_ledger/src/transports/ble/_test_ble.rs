use super::*;
use futures_util::stream;

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
    let mut pending: Notifications = Box::pin(stream::pending());
    assert!(
        tokio::time::timeout(Duration::from_millis(1), receive_packet_size(&mut pending, Uuid::nil())).await.is_err()
    );
}
