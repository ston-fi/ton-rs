# `ton_macros` crate guide

`ton_macros` is the public procedural-macro crate used by `ton_core` and `ton`.
It provides `TLB`, `FromTVMStack`, and `ton_methods`.

Use `rust-library-review` for public macro syntax or generated API changes.
Generated paths must resolve when crates are renamed. Preserve existing
attributes and diagnostics unless a breaking change is documented. Keep parsing
strict by default; tolerance such as `allow_extra` must remain opt-in.

Add focused expansion/behavior tests for macro-owned rules. Validate with
`cargo test -p ton_macros`, downstream `ton_core`/`ton` tests, strict Rustdoc and
Clippy, and `cargo package --list -p ton_macros`.
