# 1C Far Plugin (`far1c`)

[![Rust](https://img.shields.io/badge/Rust-1.88%2B-orange?logo=rust)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform: Windows](https://img.shields.io/badge/Platform-Windows-0078D6?logo=windows)](https://github.com/APertsev/farmanager)
[![Platform: Linux](https://img.shields.io/badge/Platform-Linux-FCC624?logo=linux&logoColor=black)](https://github.com/elfmz/far2l)

🇷🇺 [Версия на русском языке](README.ru.md)

A cross-platform plugin for **FAR Manager** / **far2l** that provides transparent navigation and modification of **1C:Enterprise 8** binary artifacts — just like working with archives.

## Supported Formats

| Format | Description | Read | Write |
|--------|-------------|------|-------|
| `.epf` | External data processor | ✓ | ✓ |
| `.erf` | External report | ✓ | ✓ |
| `.cf`  | Configuration | ✓ | ✓ |
| `.cfe` | Configuration extension | ✓ | ✓ |
| `.1cd` | File database | 🔜 | 🔜 |

## Installation

### Supported Platforms

- **Windows**: Far Manager 3.x (`far3`)
- **Linux/macOS**: far2l (`far2l`)
- **macOS/Linux**: far2m (`far2m`)

A separate plugin binary is built for each platform.

### Building and Installation

**Prerequisites:** [Rust toolchain](https://rustup.rs/) (1.88+)

The project provides automated scripts to build the release version and deploy it.

#### Windows (FAR Manager 3)

Run the PowerShell script:

```powershell
.\build_release.ps1
```

This script runs `cargo build --release`, assembles the plugin and language files in `target\release\far3\`, and creates a junction point in `%FARHOME%\Plugins\Far1C` so the plugin is automatically available in FAR (need to restart FAR).

#### Linux (far2l / far2m)

Run the Shell script:

```bash
./build_release.sh
```

This script cross-compiles both the `far2l` and `far2m` features and assembles them in `target/release/far2l/` and `target/release/far2m/` respectively.
For `far2l`, you can run `target/release/far2l/copy_to_far2l.sh` (requires `sudo`) to install the plugin into standard system paths (`/usr/lib/far2l/Plugins/far1c/`, `/usr/share/far2l/Plugins/far1c/`).

> **Note:** Language files (`*.lng`) are required for proper localization of the plugin UI. Without them, the plugin will display raw message IDs instead of translated strings.

## Usage

### Opening an Artifact

```
Enter / Ctrl+PgDn on .epf/.cf/.cfe  →  Opens virtual panel inside the artifact
Backspace / Ctrl+PgUp               →  Exit virtual panel
```

The plugin can also be invoked from the `F11` Plugin Menu — it will open the currently selected file.

### File Operations Inside Virtual Panel

| Key | Action |
|-----|--------|
| `F3` | View file (viewer) |
| `F4` | Edit file (editor) — changes are saved back to the container |
| `F5` | Copy from artifact to disk |
| `F6` | Copy from disk into artifact |

### Settings

Access plugin settings via `F11` → Plugin Configuration, or press `F9` → Options → Plugins configuration.

Available options:

- **Create backup before saving** — creates a timestamped backup (e.g., `test.20260512-143022.epf`)
- **Unpacking style** — Raw / Full-parse / V8Unpack-style / Json-style / EDT-style / Configurator-style

## Architecture

The project follows a three-layer architecture:

```
src/
├── lib.rs                       # Entry point, conditional Far API export
├── far/                         # LAYER 1: FAR Manager interaction (Static Multi-Feature)
│   ├── mod.rs                   # GUIDs/globals, API exports mount
│   ├── far3/                    # Implementation for FAR Manager 3 (Windows)
│   │   ├── mod.rs
│   │   ├── api.rs               # Far Plugin SDK 3.0 bindings
│   │   ├── exports.rs           # Exported C ABI functions
│   │   └── ui.rs                # UI dialogs for FAR 3
│   ├── far2l/                   # Implementation for far2l (Linux/macOS)
│   │   ├── mod.rs
│   │   ├── api.rs               # far2l Plugin API bindings
│   │   ├── exports.rs           # Exported C ABI functions
│   │   └── ui.rs                # UI dialogs for far2l
│   ├── far2m/                   # Implementation for far2m (Linux/macOS/BSD)
│   │   ├── mod.rs
│   │   ├── api.rs               # far2m Plugin API bindings
│   │   ├── exports.rs           # Exported C ABI functions
│   │   └── ui.rs                # UI dialogs for far2m
│   ├── string_utils.rs          # Cross-platform string handling (u16/u32)
│   ├── traits.rs                # FarHost design abstraction
│   ├── panels.rs                # Virtual file panel logic (VFS, navigation, commit)
│   ├── ui.rs                    # UI mount point (abstraction over platform-specific ui.rs)
│   ├── lang.rs                  # Localization via .lng files
│   └── settings.rs              # Plugin settings (unpack style, backup)
├── v8/                          # LAYER 2: 1C artifact semantics
│   ├── mod.rs                   # Entry point for V8 module
│   ├── container.rs             # CF container reader (ImageHeader, rows, pages)
│   ├── vfs_builder.rs           # VFS tree builder from container rows
│   ├── writer.rs                # CF container writer (repacking)
│   ├── uuids.rs                 # 1C metadata object type UUIDs
│   ├── styles/                  # Unpacking and presentation styles
│   │   ├── mod.rs               # PresentationStyle trait and dispatching
│   │   ├── raw.rs               # Raw binary extraction style
│   │   ├── full_parse.rs        # Full-parse style (structured metadata files)
│   │   ├── v8unpack.rs          # V8Unpack style (header + data files)
│   │   ├── json.rs              # JSON style (saby v8unpack format)
│   │   ├── edt.rs               # EDT style (1C:EDT project format)
│   │   ├── metadata_parser.rs   # Shared parsing helper for object metadata structures
│   │   └── configurator/        # Configurator dump style
│   │       ├── mod.rs           # Configurator style entry point and directory layout
│   │       └── schema/          # Schema parsers for configurator presentation
│   │           ├── mod.rs       # Entry and helper structs for schemas
│   │           ├── form_layout.rs  # Parser for 1C form layout XML files
│   │           ├── metadata_xml.rs # Parser for metadata XML files (objects/configs)
│   │           └── synonyms.rs     # Parser for synonyms/multilanguage strings
│   └── tests.rs                 # Integration tests
└── base/                        # LAYER 3: Low-level primitives (I/O, parsing)
    ├── mod.rs                   # Entry point for base primitives
    ├── reader.rs                # Abstract reader (FileReader, StringReader)
    ├── parser.rs                # Bracket-format parser for 1C metadata
    ├── bracket_json.rs          # Converts 1C bracket-format strings to JSON
    └── deflate.rs               # DEFLATE via flate2
```

## Current Status

| Phase | Description | Status |
|-------|-------------|--------|
| Phase 0 | Infrastructure setup | ✅ Complete |
| Phase 1 | Plugin skeleton + CF parser | ✅ Complete |
| Phase 2 | EPF/ERF (MVP): VFS tree, F3/F4, CF-writer, settings, localization | ✅ Complete |
| Phase 3 | Documentation & GitHub primary publication | ✅ Complete |
| Phase 4 | Dual-API refactoring (FAR 3) and far2l basic implementation | ✅ Complete |
| Phase 5 | Documentation update V2 | ✅ Complete |
| Phase 6 | Linux version (build, implementation) | ✅ Complete |
| Phase 7 | Cross-platform stability (Static Multi-Feature) | ✅ Complete |
| Phase 8 | Presentation styles implementation (Raw, Full-parse, V8Unpack, Json, EDT, Configurator) | ✅ Complete |
| Phase 9 | Artifact parsing fixes: discrepancy reports, display fixes, repacking tests, composition management | 🔄 In progress |
| Phase 10 | File operations with artifacts: unpack to folder, build from folder, round-trip testing | 🔜 Planned |
| Phase 11 | Protected modules: bytecode disassembler | 🔜 Planned |
| Phase 12 | 1CD: file database navigation | 🔜 Planned |

## Documentation

- [Project Concept](docs/project/concept.md) — architectural overview and rationale
- [Project Scope](docs/project/scope.md) — boundaries and constraints
- [Software Requirements Specification](docs/project/srs.md) — detailed functional requirements
- [Execution Plan](docs/project/plan.md) — phases, tasks, and progress

### Technical References

- [CF Container Processing](docs/1C/processing.md) — parsing algorithms
- [1C File Formats](docs/1C/formats.md) — binary format specifications
- [1C Bytecode Reference](docs/1C/bytecode.md) — VM opcode reference

## Contributing

Contributions are welcome! Please refer to the [CONTRIBUTING.md](CONTRIBUTING.md) for details.

See [CHANGELOG.md](CHANGELOG.md) for the full history of changes.

## Author

### Dmitry Kinash

- 📧 E-mail: [dv.kinash@gmail.com](mailto:dv.kinash@gmail.com)
- 💼 LinkedIn: [dv-kinash](https://www.linkedin.com/in/dv-kinash/)
- 🐙 GitHub: [@DementorAK](https://github.com/DementorAK)

## 📝 License

[MIT License](LICENSE)
