# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.23](https://github.com/Sild/ton_lib_rs/compare/ton_lib_macros-v0.0.22...ton_lib_macros-v0.0.23) - 2025-08-20

### Other

- make build works without tonlibjson feature & speedup build

## [0.0.22](https://github.com/Sild/ton_lib_rs/compare/ton_lib_macros-v0.0.21...ton_lib_macros-v0.0.22) - 2025-06-30

### Fixed

- fix rust 1.80 formatting
- fix FutureSplitMerge TLB impl

## [0.0.17](https://github.com/Sild/ton_lib_rs/compare/ton_lib_macros-v0.0.16...ton_lib_macros-v0.0.17) - 2025-06-02

### Other

- from_slize -> from_slice_sized, as_mut_mut -> as_smth_mut
- support ConfigParams
- cleanup tvm_emulator

## [0.0.16](https://github.com/Sild/ton_lib_rs/compare/ton_lib_macros-v0.0.15...ton_lib_macros-v0.0.16) - 2025-05-21

### Other

- rename TLBType -> TLB

## [0.0.15](https://github.com/Sild/ton_lib_rs/compare/ton_lib_macros-v0.0.14...ton_lib_macros-v0.0.15) - 2025-05-21

### Other

- TLBDerive for enum produces as_X, as_mut_X, is_X, From<variant> impls

## [0.0.14](https://github.com/Sild/ton_lib_rs/compare/ton_lib_macros-v0.0.13...ton_lib_macros-v0.0.14) - 2025-05-16

### Other

- support BlockInfo tlb
- tmp
- support serde_scylla feature flag, up adapter naming & vm -> tvm

## [0.0.13](https://github.com/Sild/ton_lib_rs/compare/ton_lib_macros-v0.0.12...ton_lib_macros-v0.0.13) - 2025-05-06

### Other

- hide Builder & Parser consturctors behind TonCell methods
- add zero_block_id, zero_config as constants
- cleanup macros, init ton_wallet support

## [0.0.12](https://github.com/Sild/ton_lib_rs/compare/ton_lib_macros-v0.0.11...ton_lib_macros-v0.0.12) - 2025-05-03

### Other

- update contract signature, handle no-libs in tl-response properly

## [0.0.11](https://github.com/Sild/ton_lib_rs/compare/ton_lib_macros-v0.0.10...ton_lib_macros-v0.0.11) - 2025-05-03

### Other

- add jetton_master, jetton_wallet contract support

## [0.0.10](https://github.com/Sild/ton_lib_rs/compare/ton_lib_macros-v0.0.9...ton_lib_macros-v0.0.10) - 2025-05-01

### Other

- don't run coverage on MR

## [0.0.9](https://github.com/Sild/ton_lib_rs/compare/ton_lib_macros-v0.0.8...ton_lib_macros-v0.0.9) - 2025-04-30

### Other

- Merge pull request #9 from Sild/into-coint-impl

## [0.0.8](https://github.com/Sild/ton_lib_rs/compare/ton_lib_macros-v0.0.7...ton_lib_macros-v0.0.8) - 2025-04-27

### Other

- Merge pull request #7 from Sild/add-libs-dict

## [0.0.7](https://github.com/Sild/ton_lib_rs/compare/ton_lib_macros-v0.0.6...ton_lib_macros-v0.0.7) - 2025-04-26

### Fixed

- fix package description
- fix releases
- fix projects description

## [0.0.6](https://github.com/Sild/libs_rs/compare/ton_lib_proc_macro-v0.0.5...ton_lib_proc_macro-v0.0.6) - 2025-04-20

### Fixed

- fix formatting

### Other

- add TLBRef alias for derive
- use bits_len as alias for adapter=ConstLen

## [0.0.5](https://github.com/Sild/libs_rs/compare/ton_lib_proc_macro-v0.0.4...ton_lib_proc_macro-v0.0.5) - 2025-04-19

### Other

- add TonAddress converters + TLBType, fix TLBDerive for Enums

## [0.0.4](https://github.com/Sild/libs_rs/compare/ton_lib_proc_macro-v0.0.3...ton_lib_proc_macro-v0.0.4) - 2025-04-18

### Other

- impl Account TLB
- generalize adapters
- support adapter="TLBRef"
- Update adapter interfaces ([#46](https://github.com/Sild/libs_rs/pull/46))

## [0.0.3](https://github.com/Sild/libs_rs/compare/ton_lib_proc_macro-v0.0.2...ton_lib_proc_macro-v0.0.3) - 2025-04-14

### Other

- Support dict ([#45](https://github.com/Sild/libs_rs/pull/45))

## [0.0.2](https://github.com/Sild/libs_rs/compare/ton_lib_proc_macro-v0.0.1...ton_lib_proc_macro-v0.0.2) - 2025-04-05

### Other

- Update export ([#42](https://github.com/Sild/libs_rs/pull/42))
- Add msg address ([#41](https://github.com/Sild/libs_rs/pull/41))

## [0.0.1](https://github.com/Sild/libs_rs/releases/tag/ton_lib_proc_macro-v0.0.1) - 2025-04-01

### Other

- ton_lib_proc_macro, TLBDerive ([#39](https://github.com/Sild/libs_rs/pull/39))
