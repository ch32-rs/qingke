# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.8.2] - 2026-10-02

### Fixed

- **`qingke-rt`:** do not vendor or `include!` PAC `cfgs` logic; startup uses **Cargo features** aligned with `qingke-rt/Cargo.toml`.
- **`qingke` build:** replace `cfgs.inc.rs` + `include!` with `build/cfgs.rs` (`#[path]` module) for crates.io-safe packaging.

### Changed

- **`qingke-rt` build:** enforce exactly one leaf feature using names from this crate’s `[features]` (forwarders to `qingke/`), not a duplicated constant list.

## [0.8.1] - 2026-10-02

### Fixed

- **`qingke-rt` crates.io packaging:** ship leaf cfg build input inside the published crate (0.8.0 used a workspace-only path). **0.8.0 is yanked** on crates.io; use 0.8.1+ or `version = "0.8"`.

## [0.8.0] - 2026-05-21

### Added

- QingKe 0.8: H4 dual-core, leaf core features (`v2a`, `v3f`, …), PFIC/VTF fixes ([#23](https://github.com/ch32-rs/qingke/pull/23)).
- `#[interrupt(lowcode)]` opts a single handler out of `.highcode` (RAM) placement when the `highcode` feature is enabled, keeping its body in flash. Composes with `core`. No-op when the `highcode` feature is off.

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
[0.8.1]: https://github.com/ch32-rs/qingke/compare/v0.8.0...v0.8.1
[0.8.0]: https://github.com/ch32-rs/qingke/compare/v0.7.0...v0.8.0
[0.7.0]: https://github.com/ch32-rs/qingke/compare/v0.6.1...v0.7.0
[0.6.1]: https://github.com/ch32-rs/qingke/releases/tag/v0.6.1
