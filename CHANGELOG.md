# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[Unreleased]: https://github.com/ch32-rs/qingke/compare/v0.7.0...HEAD
[0.7.0]: https://github.com/ch32-rs/qingke/compare/v0.6.1...v0.7.0
[0.6.1]: https://github.com/ch32-rs/qingke/releases/tag/v0.6.1
