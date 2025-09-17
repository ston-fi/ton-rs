use ton_lib_core::TLB;

/// ```raw
/// destroy#1f04537a
///   query_id:uint64
/// = InternalMsgBody;
/// ```
#[derive(Clone, Debug, PartialEq, TLB)]
#[tlb(prefix = 0x1f04537a, bits_len = 32, ensure_empty = true)]
pub struct SbtDestroyMsg {
    pub query_id: u64,
}
