//! Native Bluetooth through OS pairing. Discovery uses service UUIDs, never names.
use super::framing::{Reassembler, frames};
use crate::{error::TransportError, traits::Transport};
use async_trait::async_trait;
use btleplug::{
    api::{
        Central, CharPropFlags, Characteristic, Manager as _, Peripheral as _, ScanFilter, ValueNotification, WriteType,
    },
    platform::{Manager, Peripheral},
};
use futures_util::{Stream, StreamExt};
use std::{pin::Pin, time::Duration};
use tokio::{
    sync::{mpsc, oneshot},
    time::{Instant, timeout_at},
};
use uuid::Uuid;
const SERVICES: [Uuid; 4] = [
    Uuid::from_u128(0x13d634002c97000400004c6564676572),
    Uuid::from_u128(0x13d634002c97600400004c6564676572),
    Uuid::from_u128(0x13d634002c97300400004c6564676572),
    Uuid::from_u128(0x13d634002c97800400004c6564676572),
];
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct BleDeviceInfo {
    pub id: String,
    pub name: Option<String>,
    peripheral: Peripheral,
    service: Uuid,
}
struct Request {
    command: Vec<u8>,
    deadline: Instant,
    reply: oneshot::Sender<Result<Vec<u8>, TransportError>>,
}
/// A single worker owns notifications and disconnects when this session is dropped.
pub struct BleTransport {
    sender: mpsc::Sender<Request>,
}
fn backend(e: btleplug::Error) -> TransportError {
    match e {
        btleplug::Error::PermissionDenied => TransportError::Permission(Box::new(e)),
        btleplug::Error::NotConnected | btleplug::Error::DeviceNotFound => TransportError::Disconnected,
        btleplug::Error::TimedOut(_) => TransportError::Timeout,
        _ => TransportError::Backend(Box::new(e)),
    }
}
type Notifications = Pin<Box<dyn Stream<Item = ValueNotification> + Send>>;
impl BleTransport {
    /// Scans all adapters for normal-mode Ledger services within a total budget.
    pub async fn discover(timeout: Duration) -> Result<Vec<BleDeviceInfo>, TransportError> {
        if timeout.is_zero() {
            return Err(TransportError::Timeout);
        }
        let deadline = Instant::now().checked_add(timeout).ok_or(TransportError::Timeout)?;
        let (mut send, recv) = oneshot::channel();
        tokio::spawn(async move {
            let mut scanning = vec![];
            let work = async {
                let manager = Manager::new().await.map_err(backend)?;
                let adapters = manager.adapters().await.map_err(backend)?;
                for adapter in adapters {
                    scanning.push(adapter);
                    if let Some(a) = scanning.last() {
                        a.start_scan(ScanFilter {
                            services: SERVICES.to_vec(),
                        })
                        .await
                        .map_err(backend)?;
                    }
                }
                let remaining = deadline.saturating_duration_since(Instant::now());
                tokio::time::sleep(remaining.mul_f32(0.7)).await;
                let mut found = vec![];
                for a in &scanning {
                    for p in a.peripherals().await.map_err(backend)? {
                        if let Some(props) = p.properties().await.map_err(backend)?
                            && let Some(service) = props.services.iter().find(|s| SERVICES.contains(s))
                        {
                            found.push(BleDeviceInfo {
                                id: p.id().to_string(),
                                name: props.local_name,
                                peripheral: p,
                                service: *service,
                            });
                        }
                    }
                }
                Ok(found)
            };
            let result = tokio::select! {_ = send.closed()=>None,r=timeout_at(deadline,work)=>Some(r.map_err(|_|TransportError::Timeout).and_then(|x|x))};
            let _ = tokio::time::timeout(Duration::from_secs(2), async {
                for a in &scanning {
                    let _ = a.stop_scan().await;
                }
            })
            .await;
            if let Some(r) = result {
                let _ = send.send(r);
            }
        });
        recv.await.map_err(|_| TransportError::Disconnected)?
    }
    /// Connects and negotiates packet size within 20 seconds. Pair through the OS.
    pub async fn connect(device: BleDeviceInfo) -> Result<Self, TransportError> {
        let (sender, mut receiver) = mpsc::channel::<Request>(1);
        let (mut ready, opened) = oneshot::channel();
        tokio::spawn(async move {
            let p = device.peripheral;
            let setup = async {
                p.connect().await.map_err(backend)?;
                p.discover_services().await.map_err(backend)?;
                let chars = p.characteristics();
                let notify_id = Uuid::from_u128(device.service.as_u128() | (1u128 << 48));
                let write_id = Uuid::from_u128(device.service.as_u128() | (2u128 << 48));
                let notify = chars
                    .iter()
                    .find(|c| {
                        c.service_uuid == device.service
                            && c.uuid == notify_id
                            && c.properties.contains(CharPropFlags::NOTIFY)
                    })
                    .cloned()
                    .ok_or(TransportError::Frame("missing Ledger notification characteristic"))?;
                let write = chars
                    .iter()
                    .find(|c| {
                        c.service_uuid == device.service
                            && c.uuid == write_id
                            && c.properties.contains(CharPropFlags::WRITE)
                    })
                    .cloned()
                    .ok_or(TransportError::Frame("missing Ledger write characteristic"))?;
                let mut stream = p.notifications().await.map_err(backend)?;
                p.subscribe(&notify).await.map_err(backend)?;
                p.write(&write, &[8, 0, 0, 0, 0], WriteType::WithResponse).await.map_err(backend)?;
                let n = next(&mut stream, notify.uuid).await?;
                if n.len() != 6 || n[..5] != [8, 0, 0, 0, 1] || n[5] < 20 {
                    return Err(TransportError::Frame("invalid Ledger packet-size negotiation"));
                }
                Ok((stream, notify, write, n[5] as usize))
            };
            let result = tokio::select! {_ = ready.closed()=>Err(TransportError::Disconnected),r=tokio::time::timeout(Duration::from_secs(20),setup)=>r.map_err(|_|TransportError::Timeout).and_then(|x|x)};
            match result {
                Err(e) => {
                    let _ = ready.send(Err(e));
                },
                Ok((mut stream, notify, write, size)) => {
                    if ready.send(Ok(())).is_ok() {
                        while let Some(mut req) = receiver.recv().await {
                            let result = tokio::select! {
                                _=req.reply.closed()=>Err(TransportError::Disconnected),
                                r=timeout_at(req.deadline,exchange(&p,&write,&mut stream,notify.uuid,size,&req.command))=>r.map_err(|_|TransportError::Timeout).and_then(|x|x),
                            };
                            let failed = result.is_err();
                            let _ = req.reply.send(result);
                            if failed {
                                break;
                            }
                        }
                    }
                    let _ = tokio::time::timeout(Duration::from_secs(2), p.unsubscribe(&notify)).await;
                },
            }
            let _ = tokio::time::timeout(Duration::from_secs(2), p.disconnect()).await;
        });
        opened.await.map_err(|_| TransportError::Disconnected)??;
        Ok(Self { sender })
    }
}
async fn next(stream: &mut Notifications, id: Uuid) -> Result<Vec<u8>, TransportError> {
    loop {
        let n = stream.next().await.ok_or(TransportError::Disconnected)?;
        if n.uuid == id {
            return Ok(n.value);
        }
    }
}
async fn exchange(
    p: &Peripheral,
    write: &Characteristic,
    stream: &mut Notifications,
    notify: Uuid,
    size: usize,
    command: &[u8],
) -> Result<Vec<u8>, TransportError> {
    for frame in frames(command, size, false)? {
        p.write(write, &frame, WriteType::WithResponse).await.map_err(backend)?;
    }
    let mut parser = Reassembler::new(false);
    loop {
        if let Some(data) = parser.push(&next(stream, notify).await?)? {
            return Ok(data);
        }
    }
}
#[async_trait]
impl Transport for BleTransport {
    async fn exchange(&mut self, command: &[u8], timeout: Duration) -> Result<Vec<u8>, TransportError> {
        let deadline = Instant::now().checked_add(timeout).ok_or(TransportError::Timeout)?;
        let (reply, rx) = oneshot::channel();
        self.sender
            .try_send(Request {
                command: command.to_vec(),
                deadline,
                reply,
            })
            .map_err(|_| TransportError::Disconnected)?;
        timeout_at(deadline, rx)
            .await
            .map_err(|_| TransportError::Timeout)?
            .map_err(|_| TransportError::Disconnected)?
    }
}
