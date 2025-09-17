use crc::{Crc, CRC_32_ISO_HDLC};
use std::borrow::Cow;
use std::fmt::{Debug, Display, Formatter};

const CRC_16_XMODEM: Crc<u16> = Crc::<u16>::new(&crc::CRC_16_XMODEM);

#[derive(Clone, Eq, PartialEq, Hash)]
pub enum TVMGetMethodID {
    Number(i32),
    Name(Cow<'static, str>),
}

impl TVMGetMethodID {
    pub fn from_prototype(prototype: &str) -> TVMGetMethodID { Self::Number(calc_opcode(prototype)) }

    pub fn as_str(&self) -> Cow<'static, str> {
        match self {
            TVMGetMethodID::Number(num) => Cow::Owned(num.to_string()), // Dynamically allocate for number
            TVMGetMethodID::Name(cow) => match cow {
                Cow::Borrowed(s) => Cow::Borrowed(*s),  // Safe only if already 'static
                Cow::Owned(s) => Cow::Owned(s.clone()), // Clone the owned String
            },
        }
    }

    pub fn to_id(&self) -> i32 {
        match self {
            TVMGetMethodID::Name(name) => CRC_16_XMODEM.checksum(name.as_bytes()) as i32 | 0x10000,
            TVMGetMethodID::Number(id) => *id,
        }
    }
}

impl From<&'static str> for TVMGetMethodID {
    fn from(value: &'static str) -> Self { TVMGetMethodID::Name(Cow::Borrowed(value)) }
}

impl From<Cow<'_, str>> for TVMGetMethodID {
    fn from(value: Cow<'_, str>) -> Self { TVMGetMethodID::Name(Cow::Owned(value.into_owned())) }
}

impl From<String> for TVMGetMethodID {
    fn from(value: String) -> Self { TVMGetMethodID::Name(Cow::Owned(value)) }
}

impl From<i32> for TVMGetMethodID {
    fn from(value: i32) -> Self { TVMGetMethodID::Number(value) }
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
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { Display::fmt(self, f) }
}

fn calc_opcode(command: &str) -> i32 {
    let crc = Crc::<u32>::new(&CRC_32_ISO_HDLC);
    let checksum = crc.checksum(command.as_bytes());
    (checksum & 0x7fffffff) as i32
}

#[cfg(test)]
mod tests {
    use crate::emulators::tvm::tvm_method_id::TVMGetMethodID;

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
}
