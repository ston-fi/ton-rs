use crate::bail_ton_core_data;
use crate::cell::TonHash;
use crate::errors::TonCoreError;
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TxLTHash {
    pub lt: i64,
    pub hash: TonHash,
}

impl TxLTHash {
    pub const ZERO: Self = Self::new(0, TonHash::ZERO);
    pub const fn new(lt: i64, hash: TonHash) -> Self { Self { lt, hash } }
}

// Expects format "lt:hash", where lt is a number and hash is a hex string
impl FromStr for TxLTHash {
    type Err = TonCoreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (lt_str, hash_str) = match s.split_once(":") {
            Some(x) => x,
            None => bail_ton_core_data!("Expecting 'lt:hash' format, got '{s}'"),
        };
        let lt = match lt_str.parse::<i64>() {
            Ok(x) => x,
            Err(err) => bail_ton_core_data!("Failed to parse lt from '{lt_str}': {err}"),
        };
        let hash = TonHash::from_str(hash_str)?;
        Ok(TxLTHash::new(lt, hash))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use tokio_test::assert_err;

    #[test]
    fn test_tx_lt_hash_from_str() -> anyhow::Result<()> {
        let tx_lt_hash = TxLTHash::from_str("12345:abcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcd")?;
        assert_eq!(tx_lt_hash.lt, 12345);
        assert_eq!(
            tx_lt_hash.hash,
            TonHash::from_str("abcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcd")?
        );

        assert_err!(TxLTHash::from_str("123"));
        assert_err!(TxLTHash::from_str("xxx:123"));
        assert_err!(TxLTHash::from_str("123:zzz"));
        Ok(())
    }
}
