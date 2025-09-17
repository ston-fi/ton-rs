use ton_lib_core::TLB;

/// ```raw
/// revoke#6f89f5e3
///   query_id:uint64
/// = InternalMsgBody;
/// ```
#[derive(Clone, Debug, PartialEq, TLB)]
#[tlb(prefix = 0x6f89f5e3, bits_len = 32, ensure_empty = true)]
pub struct SbtRevokeMsg {
    pub query_id: u64,
}
