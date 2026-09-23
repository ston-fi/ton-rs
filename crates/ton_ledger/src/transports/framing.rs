use crate::error::TransportError;
/// Ledger framing with an optional HID channel prefix. BLE frames have no padding.
pub(super) fn frames(data: &[u8], size: usize, hid: bool) -> Result<Vec<Vec<u8>>, TransportError> {
    let prefix = if hid { 2 } else { 0 };
    if size < prefix + 6 || data.len() > 260 {
        return Err(TransportError::Frame("invalid packet or command size"));
    }
    let len = u16::try_from(data.len()).map_err(|_| TransportError::Frame("APDU length"))?;
    let mut offset = 0;
    let mut seq = 0u16;
    let mut frames = vec![];
    while offset < data.len() {
        let mut f = vec![];
        if hid {
            f.extend([1, 1]);
        }
        f.push(5);
        f.extend(seq.to_be_bytes());
        if seq == 0 {
            f.extend(len.to_be_bytes());
        }
        let n = (size - f.len()).min(data.len() - offset);
        f.extend(&data[offset..offset + n]);
        offset += n;
        if hid {
            f.resize(size, 0);
        }
        frames.push(f);
        seq += 1;
    }
    Ok(frames)
}
/// Reassembles one bounded APDU response and rejects reordered fragments.
/// Discard after returning a complete response; parsers are not reused across exchanges.
pub(super) struct Reassembler {
    hid: bool,
    seq: u16,
    len: Option<usize>,
    data: Vec<u8>,
}
impl Reassembler {
    pub(super) fn new(hid: bool) -> Self {
        Self {
            hid,
            seq: 0,
            len: None,
            data: vec![],
        }
    }
    pub(super) fn push(&mut self, frame: &[u8]) -> Result<Option<Vec<u8>>, TransportError> {
        let prefix = if self.hid { 2 } else { 0 };
        let mut header = prefix + 3;
        if frame.len() < header
            || (self.hid && frame.get(..2) != Some(&[1, 1]))
            || frame[prefix] != 5
            || u16::from_be_bytes([frame[prefix + 1], frame[prefix + 2]]) != self.seq
        {
            return Err(TransportError::Frame("channel, tag or sequence"));
        }
        if self.seq == 0 {
            if frame.len() < header + 2 {
                return Err(TransportError::Frame("missing response length"));
            }
            let n = u16::from_be_bytes([frame[header], frame[header + 1]]) as usize;
            if !(2..=260).contains(&n) {
                return Err(TransportError::Frame("response size"));
            }
            self.len = Some(n);
            header += 2;
        }
        let total = self.len.ok_or(TransportError::Frame("missing first frame"))?;
        let remaining = total - self.data.len();
        let n = remaining.min(frame.len() - header);
        if n == 0 || (!self.hid && frame.len() - header > remaining) {
            return Err(TransportError::Frame("empty or trailing fragment"));
        }
        self.data.extend(&frame[header..header + n]);
        self.seq = self.seq.checked_add(1).ok_or(TransportError::Frame("sequence overflow"))?;
        if self.data.len() == total { Ok(Some(std::mem::take(&mut self.data))) } else { Ok(None) }
    }
}
