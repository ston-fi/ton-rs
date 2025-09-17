# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.8](https://github.com/Sild/ton_lib_rs/compare/ton_lib_core-v0.0.7...ton_lib_core-v0.0.8) - 2025-08-20

### Fixed

- fix clippy

### Other

- add TVMResult trait
- make build works without tonlibjson feature & speedup build

## [0.0.7](https://github.com/Sild/ton_lib_rs/compare/ton_lib_core-v0.0.6...ton_lib_core-v0.0.7) - 2025-08-15

### Other

- support Eq for SimpleLib

## [0.0.6](https://github.com/Sild/ton_lib_rs/compare/ton_lib_core-v0.0.5...ton_lib_core-v0.0.6) - 2025-07-23

### Fixed

- fixed readme, added transaction link to example

### Other

- rewrite contract_client
- Upgraded readme
- up readme

## [0.0.5](https://github.com/Sild/ton_lib_rs/compare/ton_lib_core-v0.0.4...ton_lib_core-v0.0.5) - 2025-07-07

### Other

- calc hash using queue
- remove unused deps
- up deps
- add cache for emulate_get_method

## [0.0.4](https://github.com/Sild/ton_lib_rs/compare/ton_lib_core-v0.0.3...ton_lib_core-v0.0.4) - 2025-07-01

### Other

- add TonAddress::ZERO serial test
- review fixes
- lazy_load for cell_hash

## [0.0.3](https://github.com/Sild/ton_lib_rs/compare/ton_lib_core-v0.0.2...ton_lib_core-v0.0.3) - 2025-06-30

### Fixed

- fix rust 1.80 formatting
- fix FutureSplitMerge TLB impl
- fix build
- fix build
- fix Cargo.toml
- fix CI bage
- fix releases

### Other

- up ton_lib_core
- support jetton, nft, sbt, cleanup deps ([#66](https://github.com/Sild/ton_lib_rs/pull/66))
- try fix build
- add include for ton_lib_core
- tonlib -> ton_lib
- Dev ([#64](https://github.com/Sild/ton_lib_rs/pull/64))
- cleanup tvm_emulator
- tmp
- support serde_scylla feature flag, up adapter naming & vm -> tvm
- hide Builder & Parser consturctors behind TonCell methods
- up readme
- up build bage link
- add coverage & bages
- restruct tvm_emulator mod, prepare struct for tx_emulator
- rename tlj->tl client
- support TVMStack (as TLBType), TVMEmulator (run_get_method, send_int_msg, send_ext_msg)
- disable integration tests in github
- restruct project, move ton_lib to root folder
- up readme
- ton_lib_proc_macro, TLBDerive ([#39](https://github.com/Sild/ton_lib_rs/pull/39))
- init tonlib ([#35](https://github.com/Sild/ton_lib_rs/pull/35))
- init ton_lib ([#32](https://github.com/Sild/ton_lib_rs/pull/32))
- move crate to root ([#30](https://github.com/Sild/ton_lib_rs/pull/30))
- up readme
- rename autoreturn-pool -> auto_pool (dashes are evil)
- return dash back =( ([#17](https://github.com/Sild/ton_lib_rs/pull/17))
- merge dev ([#11](https://github.com/Sild/ton_lib_rs/pull/11))
- Init
- Initial commit
