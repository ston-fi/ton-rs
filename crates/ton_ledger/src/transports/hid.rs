//! Native Ledger USB HID. A dedicated worker owns each blocking handle.
//! HIDAPI initialization and enumeration share a process-lifetime thread.
#[cfg(test)]
#[path = "hid/_test_hid.rs"]
mod tests;
use super::framing::{Reassembler, frames};
use crate::{error::TransportError, transports::Transport};
use async_trait::async_trait;
use hidapi::{HidApi, HidDevice};
use std::{
    collections::HashSet,
    ffi::CString,
    sync::{LazyLock, Mutex, mpsc},
    time::{Duration, Instant},
};
use tokio::sync::oneshot;
/// Discovered USB HID device; obtain through the transport discovery method.
/// Cloning this descriptor does not open another session.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct HidDeviceInfo {
    /// Display identifier; changing it does not change backend identity or ownership.
    pub id: String,
    /// Optional device name reported by the OS.
    pub product: Option<String>,
    path: CString,
}
struct Request {
    command: Vec<u8>,
    deadline: Instant,
    reply: oneshot::Sender<Result<Vec<u8>, TransportError>>,
}
/// Drop closes the worker queue. Reads poll cancellation at most every 50 ms.
pub struct HidTransport {
    sender: mpsc::SyncSender<Request>,
}
fn backend(e: hidapi::HidError) -> TransportError {
    if matches!(&e, hidapi::HidError::IoError { error } if error.kind() == std::io::ErrorKind::PermissionDenied) {
        TransportError::Permission(Box::new(e))
    } else {
        TransportError::Backend(Box::new(e))
    }
}
impl HidTransport {
    /// Enumerates application interfaces without opening a wallet or prompting.
    /// A process-lifetime worker keeps the native HID manager's thread alive.
    /// Timing out does not interrupt an enumeration already inside the OS.
    pub async fn discover(timeout: Duration) -> Result<Vec<HidDeviceInfo>, TransportError> {
        tokio::time::timeout(timeout, async {
            let api = request_api()?.await.map_err(|_| TransportError::Disconnected)??;
            Ok(api
                .device_list()
                .filter(|d| d.vendor_id() == 0x2c97 && (d.usage_page() == 0xffa0 || d.interface_number() == 0))
                .map(|d| HidDeviceInfo {
                    id: d.path().to_string_lossy().into_owned(),
                    product: d.product_string().map(str::to_owned),
                    path: d.path().to_owned(),
                })
                .collect())
        })
        .await
        .map_err(|_| TransportError::Timeout)?
    }
    pub(crate) async fn connect_default(timeout: Duration) -> Result<Self, TransportError> {
        let mut devices = Self::discover(timeout).await?;
        if devices.len() > 1 {
            return Err(TransportError::AmbiguousDevice);
        }
        Self::connect(devices.pop().ok_or(TransportError::NoDevice)?).await
    }
    /// Opens a previously discovered interface, with a 10-second connection budget.
    /// Returns `DeviceBusy` while this process already owns the backend path,
    /// including while a cancelled or dropped connection is still closing.
    pub async fn connect(device: HidDeviceInfo) -> Result<Self, TransportError> {
        let (sender, receiver) = mpsc::sync_channel::<Request>(1);
        let (ready, opened) = oneshot::channel();
        spawn_worker(device.path.clone(), move || {
            let dev = request_api()
                .and_then(|reply| reply.blocking_recv().map_err(|_| TransportError::Disconnected)?)
                .and_then(|api| api.open_path(&device.path).map_err(backend));
            match dev {
                Err(e) => {
                    let _ = ready.send(Err(e));
                },
                Ok(dev) => {
                    if ready.send(Ok(())).is_err() {
                        return;
                    }
                    while let Ok(req) = receiver.recv() {
                        if req.reply.is_closed() {
                            break;
                        }
                        let result = exchange(&dev, &req);
                        let failed = result.is_err();
                        let _ = req.reply.send(result);
                        if failed {
                            break;
                        }
                    }
                },
            }
        })?;
        tokio::time::timeout(Duration::from_secs(10), opened)
            .await
            .map_err(|_| TransportError::Timeout)?
            .map_err(|_| TransportError::Disconnected)??;
        Ok(Self { sender })
    }
}

type ApiReply = oneshot::Sender<Result<HidApi, TransportError>>;

// HIDAPI's C context is never deinitialized by its Rust wrapper. On macOS its
// IOHIDManager uses the initializing thread's CFRunLoop, so that thread must not
// be a Tokio blocking-pool thread or a short-lived device worker. The static
// sender keeps this worker alive even after all transports and runtimes drop.
static API_WORKER: LazyLock<Result<mpsc::Sender<ApiReply>, std::io::Error>> =
    LazyLock::new(|| spawn_api_worker(|| HidApi::new().map_err(backend)));

fn request_api() -> Result<oneshot::Receiver<Result<HidApi, TransportError>>, TransportError> {
    let worker = API_WORKER
        .as_ref()
        .map_err(|error| TransportError::Backend(Box::new(std::io::Error::other(error.to_string()))))?;
    let (reply, result) = oneshot::channel();
    worker.send(reply).map_err(|_| TransportError::Disconnected)?;
    Ok(result)
}

fn spawn_api_worker(
    mut create_api: impl FnMut() -> Result<HidApi, TransportError> + Send + 'static,
) -> std::io::Result<mpsc::Sender<ApiReply>> {
    let (sender, receiver) = mpsc::channel::<ApiReply>();
    std::thread::Builder::new().name("ton-ledger-hid-api".into()).spawn(move || {
        for reply in receiver {
            // Cancelled queued discoveries need no OS work. An in-flight call
            // must finish, but its timeout must never retire the owning thread.
            if !reply.is_closed() {
                let _ = reply.send(create_api());
            }
        }
    })?;
    Ok(sender)
}

fn exchange(dev: &HidDevice, req: &Request) -> Result<Vec<u8>, TransportError> {
    for frame in frames(&req.command, 64, true)? {
        if req.reply.is_closed() {
            return Err(TransportError::Disconnected);
        }
        if Instant::now() >= req.deadline {
            return Err(TransportError::Timeout);
        }
        let mut report = vec![0];
        report.extend(frame);
        if dev.write(&report).map_err(backend)? != report.len() {
            return Err(TransportError::Frame("short HID write"));
        }
    }
    let mut parser = Reassembler::new(true);
    let mut frame = [0; 64];
    loop {
        if req.reply.is_closed() {
            return Err(TransportError::Disconnected);
        }
        let left = req.deadline.saturating_duration_since(Instant::now());
        if left.is_zero() {
            return Err(TransportError::Timeout);
        }
        let ms = left.as_millis().clamp(1, 50) as i32;
        let n = dev.read_timeout(&mut frame, ms).map_err(backend)?;
        if n == 0 {
            continue;
        }
        if let Some(data) = parser.push(&frame[..n])? {
            return Ok(data);
        }
    }
}
#[async_trait]
impl Transport for HidTransport {
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
        tokio::time::timeout(timeout, rx)
            .await
            .map_err(|_| TransportError::Timeout)?
            .map_err(|_| TransportError::Disconnected)?
    }
}

// Use the private OS path, never the editable or lossy display ID.
static CONNECTED_DEVICES: LazyLock<Mutex<HashSet<CString>>> = LazyLock::new(|| Mutex::new(HashSet::new()));

/// Worker-owned OS-path exclusion, released only after device-handle destruction.
struct DeviceLease(CString);
impl DeviceLease {
    fn acquire(path: CString) -> Result<Self, TransportError> {
        let mut devices = CONNECTED_DEVICES
            .lock()
            .map_err(|_| TransportError::Backend(Box::new(std::io::Error::other("HID session registry poisoned"))))?;
        if !devices.insert(path.clone()) {
            return Err(TransportError::DeviceBusy);
        }
        Ok(Self(path))
    }
}
impl Drop for DeviceLease {
    fn drop(&mut self) {
        let mut devices = CONNECTED_DEVICES.lock().unwrap_or_else(|error| error.into_inner());
        devices.remove(&self.0);
    }
}

fn spawn_worker(
    path: CString,
    work: impl FnOnce() + Send + 'static,
) -> Result<std::thread::JoinHandle<()>, TransportError> {
    let lease = DeviceLease::acquire(path)?;
    std::thread::Builder::new()
        .name("ton-ledger-hid".into())
        .spawn(move || {
            // Hold ownership through setup, all I/O, and handle destruction.
            // Dropping the caller cannot release a worker blocked in an OS call.
            let _lease = lease;
            work();
        })
        .map_err(|error| TransportError::Backend(Box::new(error)))
}
