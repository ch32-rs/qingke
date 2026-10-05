# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.8.2] - 2026-10-05

### Fixed

- **`qingke-rt`:** do not vendor or `include!` PAC `cfgs` logic; startup uses **Cargo features** aligned with `qingke-rt/Cargo.toml`.
- **`qingke` build:** replace `cfgs.inc.rs` + `include!` with `build/cfgs.rs` (`#[path]` module) for crates.io-safe packaging.
- **`qingke-rt` startup:** the `u-mode` feature now applies to `v3f` and `v5f` as well. Both wrote `mstatus = 0x6088` (`MPP = 0b00`, User mode) unconditionally, so `u-mode` had no effect on CH32H417 and Machine mode was unreachable. The default is now Machine mode (`0x7888`), matching `riscv-rt`, and `u-mode` keeps mirroring WCH's `startup_ch32h417_*.S`.
- **`qingke-rt` startup:** write `mstatus.MPP` explicitly on the V3A/V3B and V4 paths instead of relying on its reset value.

### Changed

- **`qingke-rt` build:** enforce exactly one leaf feature using names from this crate’s `[features]` (forwarders to `qingke/`), not a duplicated constant list.
- **`qingke-rt` build:** derive the core-family cfg (`qingke_v2` … `qingke_v5`) from the leaf feature name, so `lib.rs` branches per family instead of enumerating leaf features. Leaf detection is restricted to `v<family><variant>` names — `unsafe-trust-wch-atomics`, which also forwards to `qingke/`, was previously collected as a leaf.

## [0.8.1] - 2026-10-02

### Added

- QingKe 0.8: H4 dual-core, leaf core features (`v2a`, `v3f`, …), PFIC/VTF fixes ([#23](https://github.com/ch32-rs/qingke/pull/23)).
- `#[interrupt(lowcode)]` opts a single handler out of `.highcode` (RAM) placement when the `highcode` feature is enabled, keeping its body in flash. Composes with `core`. No-op when the `highcode` feature is off.

### Fixed

- **`qingke-rt` crates.io packaging:** ship leaf cfg build input inside the published crate (`build.rs` had referenced `../build/cfgs.inc.rs`, which exists only in the workspace).

## [0.7.0] - 2026-05-04

### Added

- Add `v3a` and `v3b` feature flags (previously a single internal `_v3` flag).
- Add `.uninit` section support in linker scripts (`link-highcode.x`, `link-no-highcode.x`).

### Fixed

- Fix interrupt handling for V3A-based chips (CH32V103, CH565, CH569, CH571, CH573): implement Direct mode + software-dispatch interrupt handling.
- Fix critical section implementation for V3A-based chips.

## [0.6.1] - 2025-12-08

### Fixed

- Fix linking macro in `qingke-rt-macros`.

[Unreleased]: https://github.com/ch32-rs/qingke/compare/v0.8.2...HEAD
[0.8.2]: https://github.com/ch32-rs/qingke/compare/v0.8.1...v0.8.2
[0.8.1]: https://github.com/ch32-rs/qingke/compare/v0.7.0...v0.8.1
[0.7.0]: https://github.com/ch32-rs/qingke/compare/v0.6.1...v0.7.0
[0.6.1]: https://github.com/ch32-rs/qingke/releases/tag/v0.6.1
