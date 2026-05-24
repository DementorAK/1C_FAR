# Contributing to 1C Far Plugin

Thank you for your interest in contributing to **far1c**! This document provides guidelines and information for contributors.

## Getting Started

### Prerequisites

- [Rust toolchain](https://rustup.rs/) (1.70 or later)
- FAR Manager 3 (Windows) or far2l (Linux) for testing
- Test 1C artifacts (`.epf`, `.erf`, `.cf`, `.cfe` files) — samples are provided in the `tests/` directory

### Building

```bash
# Debug build (Windows / FAR 3 default)
cargo build

# Release build (Windows / FAR 3 default)
cargo build --release

# Release build (Linux / far2l)
cargo build --release --no-default-features --features far2

# Run tests
cargo test
```

### Installing for Testing

After building, copy the plugin files to FAR Manager's plugin directory:

**Windows:**
```cmd
copy target\release\far1c.dll "%FARHOME%\Plugins\far1c\"
copy far1c_en.lng "%FARHOME%\Plugins\far1c\"
copy far1c_ru.lng "%FARHOME%\Plugins\far1c\"
```

**Linux:**
```bash
# far2l uses the FHS standard for plugins
sudo mkdir -p /usr/lib/far2l/Plugins/far1c
sudo cp target/release/libfar1c.so /usr/lib/far2l/Plugins/far1c/far1c.far-plug-wide

sudo mkdir -p /usr/share/far2l/Plugins/far1c/plug
sudo cp far1c_en.lng far1c_ru.lng /usr/share/far2l/Plugins/far1c/plug/
```
Or you can simply use the provided `install_linux.sh` script.

## Project Structure

The codebase is organized into three layers:

| Layer | Directory | Responsibility |
|-------|-----------|----------------|
| **Layer 1** | `src/far/` | FAR API interaction. Uses Dual-API Architecture (`far3` for Windows, `far2` for Linux) with `traits.rs` abstraction. |
| **Layer 2** | `src/v8/` | 1C artifact parsing, VFS tree construction, container repacking |
| **Layer 3** | `src/base/` | Low-level I/O, bracket-format parser, DEFLATE compression |

For detailed architecture, see [SRS § ADR-002](docs/project/srs.md).

## How to Contribute

### Reporting Bugs

- Check existing issues first to avoid duplicates
- Include FAR Manager version, OS, and the artifact type that caused the issue
- If possible, attach (or describe) a minimal artifact file that reproduces the bug

### Suggesting Features

- Open an issue describing the feature and its use case
- Reference relevant requirements from the [SRS](docs/project/srs.md) if applicable

### Submitting Code

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make your changes following the coding guidelines below
4. Ensure `cargo build` and `cargo test` pass
5. Submit a pull request with a clear description

## Coding Guidelines

### General

- All source code comments should be in **English**
- Project documentation may be in Russian or English
- Follow standard Rust formatting: `cargo fmt`
- Address all compiler warnings: `cargo clippy`

### Architecture Rules

- **Layer separation**: `base/` must not depend on `far/` or `v8/`. `v8/` must not depend on `far/`.
- **FFI safety**: All `extern "C"` functions must use `panic::catch_unwind` to prevent unwinding across FFI boundaries.
- **Memory management**: Strings passed to FAR API must be leaked via `Box::leak` and properly tracked for cleanup.

### Commit Messages

Use conventional commit format:

```
feat: implement CF/CFE metadata hierarchy navigation
fix: correct page alignment in container writer
docs: update SRS with new backup naming convention
refactor: extract VFS builder into separate module
test: add integration tests for EPF repacking
```

## Documentation

Project documentation lives in `docs/project/`:

| Document | Purpose |
|----------|---------|
| [concept.md](docs/project/concept.md) | High-level architecture and rationale |
| [scope.md](docs/project/scope.md) | Project boundaries |
| [srs.md](docs/project/srs.md) | Detailed requirements and acceptance criteria |
| [plan.md](docs/project/plan.md) | Implementation phases and progress |

Technical references for 1C formats are in `docs/1C/`.

## License

By contributing, you agree that your contributions will be licensed under the [MIT License](LICENSE).
