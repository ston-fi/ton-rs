# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.39](https://github.com/Sild/ton_lib_rs/compare/ton_lib-v0.0.38...ton_lib-v0.0.39) - 2025-08-20

### Fixed

- fix tests without tonlibjson feature
- fix clippy

### Other

- add TVMResult trait
- make build works without tonlibjson feature & speedup build

## [0.0.38](https://github.com/Sild/ton_lib_rs/compare/ton_lib-v0.0.37...ton_lib-v0.0.38) - 2025-08-15

### Other

- derive basic traits for few structs
- init meta
- snake_data support
- support Eq for SimpleLib

## [0.0.37](https://github.com/Sild/ton_lib_rs/compare/ton_lib-v0.0.36...ton_lib-v0.0.37) - 2025-07-23

### Fixed

- fixed readme, added transaction link to example

### Other

- rewrite contract_client
- Upgraded readme
- Improved example and fixed error display
- up readme
- up default parallel requests

## [0.0.36](https://github.com/Sild/ton_lib_rs/compare/ton_lib-v0.0.35...ton_lib-v0.0.36) - 2025-07-07

### Other

- calc hash using queue
- cargo fmt
- up deps
- Revert "upd version and CHANGELOG.md"
- upd version and CHANGELOG.md
- add cache test
- add cache for emulate_get_method

## [0.0.35](https://github.com/Sild/ton_lib_rs/compare/ton_lib-v0.0.34...ton_lib-v0.0.35) - 2025-07-01

### Other

- support ShardIdent::merge()
- lazy_load for cell_hash

## [0.0.34](https://github.com/Sild/ton_lib_rs/compare/ton_lib-v0.0.33...ton_lib-v0.0.34) - 2025-06-30

### Fixed

- fix rust 1.80 formatting
- fix FutureSplitMerge TLB impl

### Other

- bump sys 2025.6

## [0.0.31](https://github.com/Sild/ton_lib_rs/compare/ton_lib-v0.0.30...ton_lib-v0.0.31) - 2025-06-07

### Other

- rename Ordinal -> Ord
- add tx_emulator
- up TonCellUtils interface

## [0.0.30](https://github.com/Sild/ton_lib_rs/compare/ton_lib-v0.0.29...ton_lib-v0.0.30) - 2025-06-06

### Other

- cleanup TonHash traits

## [0.0.29](https://github.com/Sild/ton_lib_rs/compare/ton_lib-v0.0.28...ton_lib-v0.0.29) - 2025-06-06

### Other

- add _mut getters to MaybeAccount
- :contains_addr
- add test_block_tlb_block_info_prev_block_ids
- :split, prev_block_ids()
- rename workchain -> wc
- add ShardAccount::NON_EXIST
- add test_get_wallet_data_result_from_stack_bigint_balance_int

## [0.0.28](https://github.com/Sild/ton_lib_rs/compare/ton_lib-v0.0.27...ton_lib-v0.0.28) - 2025-06-03

### Other

- Fix get wallet data tinyint ([#56](https://github.com/Sild/ton_lib_rs/pull/56))

## [0.0.27](https://github.com/Sild/ton_lib_rs/compare/ton_lib-v0.0.26...ton_lib-v0.0.27) - 2025-06-02

### Other

- cleanup TonHash tests
- tx.out_msgs is vec
- add default values for entities
- use proper primitives for VarLenBytes
- const Coins::ZERO
- created_at, created_lt, use u128 for grams
- lazy_load for ConfigParams
- from_slize -> from_slice_sized, as_mut_mut -> as_smth_mut
- support ConfigParams
- use Coins instead of Grams, add to_uint converters
- cleanup tvm_emulator
- client module structure and naming updated ([#52](https://github.com/Sild/ton_lib_rs/pull/52))
- Be 2452 2 ([#51](https://github.com/Sild/ton_lib_rs/pull/51))
- Be 2452 Review/refactor boc module ([#49](https://github.com/Sild/ton_lib_rs/pull/49))

## [0.0.26](https://github.com/Sild/ton_lib_rs/compare/ton_lib-v0.0.25...ton_lib-v0.0.26) - 2025-05-26

### Other

- use tlb types in clients, support retry in lite_client
- move tonlibjson serde to tonlibjson mod
- don't do extra copy in boc builder
- move EMPTY_CELL_HASH from TonHash to TonCell

## [0.0.25](https://github.com/Sild/ton_lib_rs/compare/ton_lib-v0.0.24...ton_lib-v0.0.25) - 2025-05-24

### Other

- partial support for MCBlockExtra
- add tests for block hash, block_info values
- support retry for tl_client

## [0.0.24](https://github.com/Sild/ton_lib_rs/compare/ton_lib-v0.0.23...ton_lib-v0.0.24) - 2025-05-22

### Fixed

- fix CurrencyCollection::other

### Other

- get nice error if from_boc failed
- rename files, add retry config options for tl_client

## [0.0.23](https://github.com/Sild/ton_lib_rs/compare/ton_lib-v0.0.22...ton_lib-v0.0.23) - 2025-05-21

### Other

- support storage_extra_info
- rename TLBType -> TLB
- update exports

## [0.0.22](https://github.com/Sild/ton_lib_rs/compare/ton_lib-v0.0.21...ton_lib-v0.0.22) - 2025-05-21

### Fixed

- fix TLB ComputeSkipReasonNoState

### Other

- hide println
- reduce default max_parallel_requests 1000 -> 200
- up ton_net_conf interface

## [0.0.21](https://github.com/Sild/ton_lib_rs/compare/ton_lib-v0.0.20...ton_lib-v0.0.21) - 2025-05-21

### Fixed

- fix write_num in builder, fix bits_len in Coins
- fix prunned_branch serialization

### Other

- Merge pull request #34 from Sild/dev
- hide useless wornings
- TLBDerive for enum produces as_X, as_mut_X, is_X, From<variant> impls
- replace entities
- TLClient can clone, to_boc doesn't write crc32 by default
- add tests for shard_account/tx with blockchain data
- tests for tick_tock tx & tick_tock account
- add mock for TxDescr, support ShardAccount

## [0.0.20](https://github.com/Sild/ton_lib_rs/compare/ton_lib-v0.0.19...ton_lib-v0.0.20) - 2025-05-16

### Other

- update emulator interface
- add code_by_version, version_by_code methods
- make VersionHelper public

## [0.0.19](https://github.com/Sild/ton_lib_rs/compare/ton_lib-v0.0.18...ton_lib-v0.0.19) - 2025-05-16

### Other

- comment out serde_scylla
- more emulator tests
- up tvm_emulator send_msg interface
- up emulator interface
- init ConfigParams support
- somehow complete block.tlb (with many TODO)
- support BlockInfo tlb
- introduce data_provider
- tmp
- support serde_scylla feature flag, up adapter naming & vm -> tvm

## [0.0.18](https://github.com/Sild/ton_lib_rs/compare/ton_lib-v0.0.17...ton_lib-v0.0.18) - 2025-05-06

### Other

- add test for new_with_creds
- cleanup ton_wallet signatures, add test for create_ext_in_msg
- don't enable sys in tests by default
- up sys dep
- cleanup feature=sys deps, finish TonWallet support
- support wallet_msg_body v1-v5 & OutList, OutAction
- moveout client_types to client
- hide Builder & Parser consturctors behind TonCell methods
- add zero_block_id, zero_config as constants
- cleanup macros, init ton_wallet support

## [0.0.17](https://github.com/Sild/ton_lib_rs/compare/ton_lib-v0.0.16...ton_lib-v0.0.17) - 2025-05-03

### Other

- add vmstack::push/pop tuple

## [0.0.16](https://github.com/Sild/ton_lib_rs/compare/ton_lib-v0.0.15...ton_lib-v0.0.16) - 2025-05-03

### Other

- add jetton_master, jetton_wallet contract support

## [0.0.15](https://github.com/Sild/ton_lib_rs/compare/ton_lib-v0.0.14...ton_lib-v0.0.15) - 2025-05-01

### Other

- don't run coverage on MR

## [0.0.14](https://github.com/Sild/ton_lib_rs/compare/ton_lib-v0.0.13...ton_lib-v0.0.14) - 2025-04-30

### Other

- Merge pull request #9 from Sild/into-coint-impl

## [0.0.13](https://github.com/Sild/ton_lib_rs/compare/ton_lib-v0.0.12...ton_lib-v0.0.13) - 2025-04-27

### Other

- Merge pull request #7 from Sild/add-libs-dict

## [0.0.12](https://github.com/Sild/ton_lib_rs/compare/ton_lib-v0.0.11...ton_lib-v0.0.12) - 2025-04-26

### Other

- add LiteClient, TonlibClient (TLClient)

## [0.0.10](https://github.com/Sild/libs_rs/compare/ton_lib-v0.0.9...ton_lib-v0.0.10) - 2025-04-20

### Other

- Build dict bench ([#51](https://github.com/Sild/libs_rs/pull/51))

## [0.0.9](https://github.com/Sild/libs_rs/compare/ton_lib-v0.0.8...ton_lib-v0.0.9) - 2025-04-20

### Other

- add TLBRef alias for derive
- use bits_len as alias for adapter=ConstLen

## [0.0.8](https://github.com/Sild/libs_rs/compare/ton_lib-v0.0.7...ton_lib-v0.0.8) - 2025-04-19

### Other

- add Coins::zero() constructor
- add TonAddress converters + TLBType, fix TLBDerive for Enums

## [0.0.7](https://github.com/Sild/libs_rs/compare/ton_lib-v0.0.6...ton_lib-v0.0.7) - 2025-04-18

### Other

- small fixes after using in external project

## [0.0.6](https://github.com/Sild/libs_rs/compare/ton_lib-v0.0.5...ton_lib-v0.0.6) - 2025-04-18

### Fixed

- fix flack test

### Other

- impl Account TLB
- generalize adapters
- support adapter="TLBRef"
- Update adapter interfaces ([#46](https://github.com/Sild/libs_rs/pull/46))

## [0.0.5](https://github.com/Sild/libs_rs/compare/ton_lib-v0.0.4...ton_lib-v0.0.5) - 2025-04-14

### Other

- Support dict ([#45](https://github.com/Sild/libs_rs/pull/45))
- cleanup naming, update imports, add TLBRef, TonCellRef now behave like TonCell in TLB context
- cleanup verify_prefix

## [0.0.4](https://github.com/Sild/libs_rs/compare/ton_lib-v0.0.3...ton_lib-v0.0.4) - 2025-04-05

### Fixed

- fix formatting

### Other

- add TLB prefix to tlb entities, add tests for dyn_len types ([#43](https://github.com/Sild/libs_rs/pull/43))
- Update export ([#42](https://github.com/Sild/libs_rs/pull/42))
- Add msg address ([#41](https://github.com/Sild/libs_rs/pull/41))

## [0.0.3](https://github.com/Sild/libs_rs/compare/ton_lib-v0.0.2...ton_lib-v0.0.3) - 2025-04-01

### Other

- ton_lib_proc_macro, TLBDerive ([#39](https://github.com/Sild/libs_rs/pull/39))
- only TonCell left ([#38](https://github.com/Sild/libs_rs/pull/38))
- implement write/read big_num ([#36](https://github.com/Sild/libs_rs/pull/36))

## [0.0.2](https://github.com/Sild/libs_rs/compare/ton_lib-v0.0.1...ton_lib-v0.0.2) - 2025-03-07

### Other

- init tonlib ([#35](https://github.com/Sild/libs_rs/pull/35))
- release v0.3.1 ([#33](https://github.com/Sild/libs_rs/pull/33))
