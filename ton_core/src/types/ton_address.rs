use crate::cell::CellBuilder;
use crate::cell::CellParser;
use crate::cell::TonHash;

use crate::bits_utils::BitsUtils;
use crate::errors::TonCoreError;
use crate::traits::tlb::TLB;
use crate::types::tlb_core::*;
use crate::{bail_ton_core, bail_ton_core_data};
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use base64::Engine;
use crc::Crc;
use std::cmp::Ordering;
use std::fmt::{Debug, Display};
use std::str::FromStr;

const CRC_16_XMODEM: Crc<u16> = Crc::<u16>::new(&crc::CRC_16_XMODEM);

/// User-friendly representation of TON address
///
/// Can be converted to/from TLB MsgAddress (can represent MsgAddressInt and MsgAddressNone)
/// Default string representation is URL-safe base64 with bounceable mainnet tag
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TonAddress {
    pub workchain: i32,
    pub hash: TonHash,
}

impl TonAddress {
    pub const ZERO: Self = TonAddress::new(0, TonHash::ZERO);

    pub const fn new(workchain: i32, hash: TonHash) -> Self { Self { workchain, hash } }

    pub fn from_msg_address<T: Into<MsgAddress>>(msg_address: T) -> Result<Self, TonCoreError> {
        match msg_address.into() {
            MsgAddress::Ext(MsgAddressExt::None(_)) => Ok(TonAddress::ZERO),
            MsgAddress::Int(int) => from_msg_address_int(&int),
            other => {
                raise_address_error(&format!("{other:?}"), "can't make TonAddress from specified MsgAddress")?;
                unreachable!()
            }
        }
    }

    pub fn to_hex(&self) -> String { format!("{}:{}", self.workchain, hex::encode(self.hash.as_slice())) }

    pub fn to_base64(&self, mainnet: bool, bounce: bool, urlsafe: bool) -> String {
        let mut buf = [0; 36];
        let tag: u8 = match (mainnet, bounce) {
            (true, true) => 0x11,
            (true, false) => 0x51,
            (false, true) => 0x91,
            (false, false) => 0xD1,
        };
        buf[0] = tag;
        buf[1] = (self.workchain & 0xff) as u8;
        buf[2..34].clone_from_slice(self.hash.as_slice());
        let crc = CRC_16_XMODEM.checksum(&buf[0..34]);
        buf[34] = ((crc >> 8) & 0xff) as u8;
        buf[35] = (crc & 0xff) as u8;
        if urlsafe {
            URL_SAFE_NO_PAD.encode(buf)
        } else {
            STANDARD.encode(buf)
        }
    }

    pub fn to_msg_address_none(&self) -> Result<MsgAddressNone, TonCoreError> {
        if self != &TonAddress::ZERO {
            bail_ton_core!("Can't convert non-zero address={self} to MsgAddressNone");
        }
        Ok(MsgAddressNone {})
    }

    pub fn to_msg_address_int(&self) -> MsgAddressInt {
        MsgAddressIntStd {
            anycast: None,
            workchain: self.workchain as i8,
            address: self.hash.clone(),
        }
        .into()
    }

    pub fn to_msg_address(&self) -> MsgAddress {
        if self == &TonAddress::ZERO {
            return MsgAddress::NONE;
        }
        MsgAddress::Int(self.to_msg_address_int())
    }
}

impl FromStr for TonAddress {
    type Err = TonCoreError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() == 48 {
            return from_base64(s);
        }
        from_hex(s)
    }
}

impl Display for TonAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_base64(true, true, true))
    }
}

impl Debug for TonAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(&format!("TonAddress(\"{self}\")")).finish()
    }
}

impl PartialOrd for TonAddress {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self == other {
            return Some(Ordering::Equal);
        }
        let self_hash = self.to_cell().and_then(|c| c.cell_hash());
        let other_hash = other.to_cell().and_then(|c| c.cell_hash());
        match (self_hash, other_hash) {
            (Ok(hash0), Ok(hash1)) => Some(hash0.cmp(&hash1)),
            _ => {
                log::error!("Failed to calc hash for addresses: {self:?} and {other:?}");
                None
            }
        }
    }
}

impl TLB for TonAddress {
    fn read_definition(parser: &mut CellParser) -> Result<Self, TonCoreError> {
        TonAddress::from_msg_address(MsgAddress::read(parser)?)
    }

    fn write_definition(&self, builder: &mut CellBuilder) -> Result<(), TonCoreError> {
        match self.to_msg_address_none() {
            Ok(none) => none.write(builder),
            Err(_) => self.to_msg_address_int().write(builder),
        }
    }
}

fn from_base64<T: AsRef<str>>(addr: T) -> Result<TonAddress, TonCoreError> {
    let addr_str = addr.as_ref();
    if addr_str.chars().any(|c| c == '-' || c == '_') {
        from_bytes(&URL_SAFE_NO_PAD.decode(addr_str)?, addr_str)
    } else {
        from_bytes(&STANDARD.decode(addr_str)?, addr_str)
    }
}

fn from_hex<T: AsRef<str>>(addr: T) -> Result<TonAddress, TonCoreError> {
    let addr_str = addr.as_ref();
    let parts: Vec<&str> = addr_str.split(':').collect();

    if parts.len() != 2 {
        raise_address_error(addr_str, "expecting 2 parts divided by ':'")?;
    }

    let wc = parts[0].parse::<i32>()?;

    let hash = TonHash::from_vec(hex::decode(parts[1])?)?;
    Ok(TonAddress::new(wc, hash))
}

fn from_bytes(bytes: &[u8], addr_str: &str) -> Result<TonAddress, TonCoreError> {
    if bytes.len() != 36 {
        raise_address_error(addr_str, format!("expecting 36 bytes, got {}", bytes.len()))?;
    }

    let addr_crc = ((bytes[34] as u16) << 8) | bytes[35] as u16;
    if addr_crc != CRC_16_XMODEM.checksum(&bytes[0..34]) {
        raise_address_error(addr_str, "crc32 mismatch")?;
    }

    let address = TonAddress {
        workchain: bytes[1] as i8 as i32,
        hash: TonHash::from_slice(&bytes[2..34])?,
    };
    Ok(address)
}

fn from_msg_address_int(msg_address: &MsgAddressInt) -> Result<TonAddress, TonCoreError> {
    let (wc, addr, bits_len, anycast) = match msg_address {
        MsgAddressInt::Std(MsgAddressIntStd {
            workchain,
            address,
            anycast,
        }) => (*workchain as i32, address.as_slice(), 256, anycast.as_ref()),
        MsgAddressInt::Var(MsgAddressIntVar {
            workchain,
            address,
            addr_bits_len,
            anycast,
        }) => (*workchain, address.as_slice(), *addr_bits_len, anycast.as_ref()),
    };

    let anycast = match anycast {
        Some(anycast) => anycast,
        None => return Ok(TonAddress::new(wc, TonHash::from_slice(addr)?)),
    };

    if bits_len < anycast.rewrite_pfx.bits_len as u32 {
        let err_msg =
            format!("rewrite_pfx has {} bits, but address has only {bits_len} bits", anycast.rewrite_pfx.bits_len);
        let ext_addr_str = format!("address: {msg_address:?}, anycast: {anycast:?}");
        raise_address_error(&ext_addr_str, err_msg)?
    }

    let new_prefix = anycast.rewrite_pfx.as_slice();

    let bits = anycast.rewrite_pfx.bits_len;
    let mut addr_mutable = addr.to_vec();

    if !BitsUtils::rewrite(new_prefix, 0, addr_mutable.as_mut_slice(), 0, bits) {
        let err_msg = format!(
            "Failed to rewrite address prefix with new_prefix={new_prefix:?}, address={msg_address:?}, bits={bits}"
        );
        let ext_addr_str = format!("address: {msg_address:?}, anycast: {anycast:?}");
        raise_address_error(&ext_addr_str, err_msg)?
    }

    Ok(TonAddress::new(wc, TonHash::from_vec(addr_mutable)?))
}

fn raise_address_error<T: AsRef<str>>(address: &str, msg: T) -> Result<(), TonCoreError> {
    bail_ton_core_data!("Can't parse {address}, err: {}", msg.as_ref())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test::{assert_err, assert_ok};

    #[test]
    fn test_ton_address_to_string() -> anyhow::Result<()> {
        let bytes = TonHash::from_str("e4d954ef9f4e1250a26b5bbad76a1cdd17cfd08babad6f4c23e372270aef6f76")?;
        let addr = TonAddress::new(0, bytes);
        assert_eq!(addr.to_hex(), "0:e4d954ef9f4e1250a26b5bbad76a1cdd17cfd08babad6f4c23e372270aef6f76");
        assert_eq!(addr.to_base64(true, true, true), "EQDk2VTvn04SUKJrW7rXahzdF8_Qi6utb0wj43InCu9vdjrR");
        assert_eq!(addr.to_base64(true, true, false), "EQDk2VTvn04SUKJrW7rXahzdF8/Qi6utb0wj43InCu9vdjrR");
        assert_eq!(addr.to_base64(true, true, true), addr.to_string());
        Ok(())
    }

    #[test]
    fn test_ton_address_from_str() -> anyhow::Result<()> {
        let bytes = TonHash::from_str("e4d954ef9f4e1250a26b5bbad76a1cdd17cfd08babad6f4c23e372270aef6f76")?;
        let addr = TonAddress::new(0, bytes);
        assert_eq!(TonAddress::from_str("0:e4d954ef9f4e1250a26b5bbad76a1cdd17cfd08babad6f4c23e372270aef6f76")?, addr);
        assert_eq!(TonAddress::from_str("EQDk2VTvn04SUKJrW7rXahzdF8_Qi6utb0wj43InCu9vdjrR")?, addr);
        assert_eq!(TonAddress::from_str("EQDk2VTvn04SUKJrW7rXahzdF8/Qi6utb0wj43InCu9vdjrR")?, addr);
        Ok(())
    }

    #[test]
    fn test_ton_address_crc_error() -> anyhow::Result<()> {
        assert_err!(TonAddress::from_str("EQDk2VTvn04SUKJrW7rXahzdF8_Qi6utb0wj43InCu9vdjra"));
        Ok(())
    }

    #[test]
    fn test_ton_address_ord() -> anyhow::Result<()> {
        let address0 = TonAddress::from_str("EQBKwtMZSZurMxGp7FLZ_lM9t54_ECEsS46NLR3qfIwwTnKW")?;
        let address1 = TonAddress::from_str("EQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAM9c")?;
        assert_eq!(address0.partial_cmp(&address1), Some(Ordering::Less));
        Ok(())
    }

    #[test]
    fn test_ton_address_to_msg_addr_int() -> anyhow::Result<()> {
        let address = TonAddress::from_str("EQDk2VTvn04SUKJrW7rXahzdF8_Qi6utb0wj43InCu9vdjrR")?;
        let msg_addr = address.to_msg_address_int();
        let expected = MsgAddressIntStd {
            anycast: None,
            workchain: 0,
            address: TonHash::from_str("e4d954ef9f4e1250a26b5bbad76a1cdd17cfd08babad6f4c23e372270aef6f76")?,
        }
        .into();
        assert_eq!(msg_addr, expected);
        Ok(())
    }

    #[test]
    fn test_ton_address_to_msg_addr_none() -> anyhow::Result<()> {
        let address = TonAddress::from_str("EQDk2VTvn04SUKJrW7rXahzdF8_Qi6utb0wj43InCu9vdjrR")?;
        assert_err!(address.to_msg_address_none());
        let address = TonAddress::ZERO;
        assert_ok!(address.to_msg_address_none());
        Ok(())
    }

    #[test]
    fn test_ton_address_tlb_type() -> anyhow::Result<()> {
        for addr in [
            "EQDk2VTvn04SUKJrW7rXahzdF8_Qi6utb0wj43InCu9vdjrR",
            "EQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAM9c",
        ] {
            let address = TonAddress::from_str(addr)?;
            let cell = address.to_cell()?;
            let parsed_address = TonAddress::from_cell(&cell)?;
            assert_eq!(address, parsed_address);
        }
        Ok(())
    }

    #[test]
    fn test_ton_address_zero_to_string() -> anyhow::Result<()> {
        assert_eq!(TonAddress::ZERO.to_string(), "EQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAM9c");
        Ok(())
    }
}
