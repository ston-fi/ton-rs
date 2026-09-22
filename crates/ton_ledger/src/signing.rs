//! Explicit permission for content the device cannot fully display.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum SigningPolicy {
    /// Only recognized payloads whose nested content is also displayed.
    #[default]
    ClearOnly,
    /// Permit unknown payloads, state init and opaque nested references by hash.
    AllowOpaque,
}
