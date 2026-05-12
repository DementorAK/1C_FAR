# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

### Planned

- Phase 4: Cross-platform testing on Linux (far2l) and persistent settings
- Phase 5: `.cf` / `.cfe` — full metadata hierarchy navigation and cascading rebuild
- Phase 6: Protected modules — bytecode disassembler
- Phase 7: `.1cd` — file database navigation and configuration replacement
- Phase 8: Lazy loading for large files (> 100 MB)

---

## [0.3.3] — 2026-05-12

### Added

- `LICENSE` (MIT)
- `CONTRIBUTING.md` — contributor guidelines
- `README.ru.md` — Russian version of README
- `CHANGELOG.md` — project changelog in Keep a Changelog format
- `rust-toolchain.toml` for reproducible builds (stable, rustfmt, clippy, Windows + Linux targets)
- `rust-version = "1.88"` in `Cargo.toml` (MSRV verified by `cargo-msrv`)
- Cargo.toml metadata: `repository`, `license`, `keywords`, `categories`

### Changed

- README.md fully rewritten in English with badges, language file install instructions, architecture overview, and phase status table
- `docs/project/plan.md`: corrected module architecture to match actual source tree, added Phase 3 (Documentation & GitHub) and Phase 4 (Cross-platform testing), renumbered subsequent phases to 5–8
- `docs/project/srs.md`: updated ADR-002 (actual module structure), fixed broken doc links, documented timestamped backup format, added missing crates (`log`, `chrono`, etc.), bumped to v2.1
- `docs/project/scope.md`: replaced outdated Lua mention with Rust, fixed broken link to processing.md
- `.gitignore`: extended with IDE and OS artifact patterns

### Fixed

- Broken documentation links (`cf_processing.md` → `processing.md`, `1c_bytecode.md` → `bytecode.md`)
- Missing `progress` argument in `writer.write()` call in integration tests

---

## [0.2.11] — 2026-05-12

### Added

- Plugin settings dialog (F11 → Configure): backup toggle, unpack style selection
- Localization support: RU/EN via `.lng` files (`far1c_ru.lng`, `far1c_en.lng`)
- Progress bar during container repacking operations

---

## [0.2.10] — 2026-05-12

### Added

- Timestamped backup naming (`[stem].[YYYYMMDD-HHMMSS].[ext]`) replacing the old `.bak` suffix

---

## [0.2.9] — 2026-05-12

### Fixed

- Plugin activation from F11 menu — current panel item is correctly resolved

---

## [0.2.8] — 2026-05-11

### Added

- CF container writer (`v8/writer.rs`) with triplet pointer tables, page alignment, and configurable compression
- Repacking infrastructure: `commit_changes()` in `PluginPanel`, atomic write via temp file + rename

---

## [0.2.7] — 2026-05-11

### Added

- F4 edit infrastructure: temporary file extraction, editor invocation, modification tracking via `is_modified` flag

---

## [0.2.6] — 2026-05-09

### Added

- Full Far Plugin SDK 3.0 ABI coverage (`src/far/api.rs`, 87 KB)
- `windebug_logger` for Windows debug output, `simple_logger` for Linux

---

## [0.2.5] — 2026-05-06

### Added

- Hierarchical VFS tree builder (`v8/vfs_builder.rs`) with UUID-based type recognition
- F3 viewer support — extract `.bsl` content to temp file, open FAR viewer

---

## [0.2.4] — 2026-05-04

### Added

- UUID-to-type mapping for 1C metadata objects (`v8/uuids.rs`)
- Virtual directory grouping: `Forms/`, `Templates/`, `Commands/`

---

## [0.2.3] — 2026-05-04

### Fixed

- Panel navigation stability — `.` and `..` entries handled correctly

---

## [0.2.2] — 2026-05-01

### Added

- CF container reader (`v8/container.rs`): ImageHeader, PageHeader, RowHeader, chain resolution, DEFLATE via `flate2`
- Integration tests for EPF/ERF/CF/CFE parsing and repacking

---

## [0.2.1] — 2026-04-30

### Added

- Three-layer project structure: `far/`, `v8/`, `base/`
- Build script (`build.rs`) with auto-incrementing build number and version embedding

---

## [0.2.0] — 2026-04-27

### Changed

- **Full migration from Lua to Rust.** Plugin is now a native `.dll`/`.so` built with Rust and C ABI.
- Far Plugin API exported via `extern "C"`: `SetStartupInfoW`, `GetPluginInfoW`, `GetGlobalInfoW`, `OpenW`, `AnalyseW`, `CloseAnalyseW`, `GetFindDataW`, `FreeFindDataW`, `SetDirectoryW`, `GetFilesW`, `PutFilesW`, `ClosePanelW`, `ProcessPanelEventW`, `ConfigureW`

---

## [0.1.0] — 2026-04-17

### Added

- Initial Lua-based prototype (superseded by v0.2.0)
- Project concept, SRS v1.0, scope statement

[Unreleased]: https://github.com/DementorAK/far1c/compare/v0.3.3...HEAD
[0.3.3]: https://github.com/DementorAK/far1c/compare/v0.2.11...v0.3.3
[0.2.11]: https://github.com/DementorAK/far1c/compare/v0.2.10...v0.2.11
[0.2.10]: https://github.com/DementorAK/far1c/compare/v0.2.9...v0.2.10
[0.2.9]: https://github.com/DementorAK/far1c/compare/v0.2.8...v0.2.9
[0.2.8]: https://github.com/DementorAK/far1c/compare/v0.2.7...v0.2.8
[0.2.7]: https://github.com/DementorAK/far1c/compare/v0.2.6...v0.2.7
[0.2.6]: https://github.com/DementorAK/far1c/compare/v0.2.5...v0.2.6
[0.2.5]: https://github.com/DementorAK/far1c/compare/v0.2.4...v0.2.5
[0.2.4]: https://github.com/DementorAK/far1c/compare/v0.2.3...v0.2.4
[0.2.3]: https://github.com/DementorAK/far1c/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/DementorAK/far1c/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/DementorAK/far1c/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/DementorAK/far1c/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/DementorAK/far1c/releases/tag/v0.1.0
