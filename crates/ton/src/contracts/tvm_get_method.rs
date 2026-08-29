use crate::block_tlb::TVMStack;
use crate::errors::TonResult;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use crc::{CRC_32_ISO_HDLC, Crc};
use std::borrow::Cow;
use std::fmt::{Debug, Display, Formatter};
use ton_core::traits::tlb::TLB;

const CRC_16_XMODEM: Crc<u16> = Crc::<u16>::new(&crc::CRC_16_XMODEM);

/// TVM get-method identifier represented by a numeric ID or method name.
///
/// Serde represents [`Number`](Self::Number) as an integer and [`Name`](Self::Name) as a string.
#[derive(Clone, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
#[non_exhaustive]
pub enum TVMGetMethodID {
    /// Numeric method identifier.
    Number(i32),
    /// Method name converted to its numeric identifier before execution.
    Name(Cow<'static, str>),
}

impl TVMGetMethodID {
    /// Creates a numeric ID from a method prototype.
    pub fn from_prototype(prototype: &str) -> TVMGetMethodID {
        Self::Number(calc_opcode(prototype))
    }

    /// Returns the numeric ID or method name as text.
    pub fn as_str(&self) -> Cow<'static, str> {
        match self {
            TVMGetMethodID::Number(num) => Cow::Owned(num.to_string()),
            TVMGetMethodID::Name(cow) => match cow {
                Cow::Borrowed(s) => Cow::Borrowed(*s),
                Cow::Owned(s) => Cow::Owned(s.clone()),
            },
        }
    }

    /// Resolves this identifier to its numeric TVM ID.
    pub fn to_id(&self) -> i32 {
        match self {
            TVMGetMethodID::Name(name) => CRC_16_XMODEM.checksum(name.as_bytes()) as i32 | 0x10000,
            TVMGetMethodID::Number(id) => *id,
        }
    }
}

impl From<&'static str> for TVMGetMethodID {
    fn from(value: &'static str) -> Self {
        TVMGetMethodID::Name(Cow::Borrowed(value))
    }
}

impl From<Cow<'_, str>> for TVMGetMethodID {
    fn from(value: Cow<'_, str>) -> Self {
        TVMGetMethodID::Name(Cow::Owned(value.into_owned()))
    }
}

impl From<String> for TVMGetMethodID {
    fn from(value: String) -> Self {
        TVMGetMethodID::Name(Cow::Owned(value))
    }
}

impl From<i32> for TVMGetMethodID {
    fn from(value: i32) -> Self {
        TVMGetMethodID::Number(value)
    }
}

impl Display for TVMGetMethodID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TVMGetMethodID::Number(n) => write!(f, "#{n:08x}"),
            TVMGetMethodID::Name(m) => write!(f, "'{m}'"),
        }
    }
}

impl Debug for TVMGetMethodID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

fn calc_opcode(command: &str) -> i32 {
    let crc = Crc::<u32>::new(&CRC_32_ISO_HDLC);
    let checksum = crc.checksum(command.as_bytes());
    (checksum & 0x7fffffff) as i32
}

/// Successful TVM get-method execution.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct TVMGetMethodSuccess {
    /// TVM exit code. Codes `0` and `1` indicate success.
    pub vm_exit_code: i32,
    /// Emulator log, when provided by the implementation.
    pub vm_log: Option<String>,
    /// Result stack serialized as a base64-encoded BOC.
    pub stack_boc_base64: String,
    /// Gas units consumed by execution.
    pub gas_used: i32,
    /// Unprocessed provider response retained for diagnostics.
    pub raw_response: String,
}

impl TVMGetMethodSuccess {
    /// Parses the returned TVM stack.
    pub fn stack_parsed(&self) -> TonResult<TVMStack> {
        Ok(TVMStack::from_boc_base64(&self.stack_boc_base64)?)
    }

    /// Decodes the returned stack BOC.
    pub fn stack_boc(&self) -> TonResult<Vec<u8>> {
        Ok(BASE64_STANDARD.decode(self.stack_boc_base64.as_bytes())?)
    }

    /// Returns whether the VM exit code indicates success.
    pub fn exit_success(&self) -> bool {
        self.vm_exit_code == 0 || self.vm_exit_code == 1
    }
}

impl From<ton_core::traits::emulation_provider::EmulatorGetMethodSuccess> for TVMGetMethodSuccess {
    fn from(value: ton_core::traits::emulation_provider::EmulatorGetMethodSuccess) -> Self {
        Self {
            vm_exit_code: value.vm_exit_code,
            vm_log: value.vm_log,
            stack_boc_base64: BASE64_STANDARD.encode(value.stack_boc),
            gas_used: value.gas_used.unwrap_or_default(),
            raw_response: value.raw_response.unwrap_or_default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TVMGetMethodID;

    #[test]
    fn test_hex_format() -> anyhow::Result<()> {
        let method_id: TVMGetMethodID = 0x1234beef.into();
        let s = format!("{method_id}");
        assert_eq!(s, "#1234beef");
        Ok(())
    }

    #[test]
    fn test_opcode() -> anyhow::Result<()> {
        let p = "transfer query_id:uint64 amount:VarUInteger 16 destination:MsgAddress \
        response_destination:MsgAddress custom_payload:Maybe ^Cell forward_ton_amount:VarUInteger 16 \
        forward_payload:Either Cell ^Cell = InternalMsgBody";
        let method_id: TVMGetMethodID = TVMGetMethodID::from_prototype(p);
        assert_eq!(method_id, TVMGetMethodID::Number(0x0f8a7ea5));
        Ok(())
    }

    #[test]
    fn test_serde_contract() -> anyhow::Result<()> {
        let number = TVMGetMethodID::Number(0x1234);
        assert_eq!(serde_json::to_string(&number)?, "4660");
        assert_eq!(serde_json::from_str::<TVMGetMethodID>("4660")?, number);

        let name = TVMGetMethodID::from("get_wallet_data");
        assert_eq!(serde_json::to_string(&name)?, "\"get_wallet_data\"");
        assert_eq!(serde_json::from_str::<TVMGetMethodID>("\"get_wallet_data\"")?, name);

        Ok(())
    }
}
