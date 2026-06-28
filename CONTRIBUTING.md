# Contributing to 1C Far Plugin

Thank you for your interest in contributing to **far1c**! This document provides guidelines and information for contributors.

## Getting Started

### Prerequisites

- [Rust toolchain](https://rustup.rs/) (1.70 or later)
- FAR Manager 3 (Windows) or far2l (Linux) for testing
- Test 1C artifacts (`.epf`, `.erf`, `.cf`, `.cfe` files) — samples are provided in the `tests/` directory

### Checking Code

When working on different features, use `cargo check` with the respective target platform:

```bash
# Windows (far3 - default)
cargo check --target x86_64-pc-windows-msvc

# Linux (far2l)
cargo check --no-default-features --features far2l --target x86_64-unknown-linux-gnu

# Linux (far2m)
cargo check --no-default-features --features far2m --target x86_64-unknown-linux-gnu

# Run tests
cargo test
```

### Building and Installing

The project provides automation scripts to build the release version and deploy it.

**Windows (FAR 3)**
Run the PowerShell script:

```powershell
.\build_release.ps1
```

This script will:

- Run `cargo build --release`
- Assemble `far1c.dll` and `.lng` files into `target\release\far3\`
- Attempt to create a junction point in `C:\Program Files\Far Manager\Plugins\Far1C` (requires Administrator privileges) so you don't need to manually copy files after each build.

**Linux (far2l / far2m)**
Run the Shell script:

```bash
./build_release.sh
```

This script will:

- Build the `far2l` feature, rename the binary to `far1c.far-plug-wide`, and assemble it with language files into `target/release/far2l/`.
- Build the `far2m` feature (keeping the same build number), rename the binary, and assemble it into `target/release/far2m/`.
- For `far2l`, you can then run `target/release/far2l/copy_to_far2l.sh` to install the plugin into standard system paths (`/usr/lib/far2l/Plugins/far1c/`, `/usr/share/far2l/Plugins/far1c/`).

## Project Structure

The codebase is organized into three layers:

| Layer | Directory | Responsibility |
|-------|-----------|----------------|
| **Layer 1** | `src/far/` | FAR API interaction. Uses Static Multi-Feature Architecture (`far3` for Windows, `far2l` and `far2m` for Linux) with `traits.rs` design abstraction. |
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
