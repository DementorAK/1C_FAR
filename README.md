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

### Building from Source

**Prerequisites:** [Rust toolchain](https://rustup.rs/) (1.88+)

```bash
cargo build --release
```
*(Note: Windows/FAR 3 is the default build target)*

### Windows (FAR Manager 3)

1. Build the plugin (see above)
2. Create plugin directory: `%FARHOME%\Plugins\far1c\`
3. Copy `target\release\far1c.dll` to the plugin directory
4. Copy language files `far1c_en.lng` and `far1c_ru.lng` to the same directory
5. Restart FAR Manager
6. The plugin will appear in the `F11` menu

### Linux (far2l)

The easiest way to install on Linux is using the provided installation script, which builds the plugin and copies it to the system FHS directories.

1. Make the script executable and run it:
   ```bash
   chmod +x install_linux.sh
   ./install_linux.sh
   ```
2. Restart far2l

> **Note:** The script uses `sudo` to install the plugin binary to `/usr/lib/far2l/Plugins/far1c/far1c.far-plug-wide` and language files to `/usr/share/far2l/Plugins/far1c/plug/` in accordance with the far2l FHS standard.

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
- **Unpacking style** — Raw / Full-parse / V8Unpack-style / Saby-style

## Architecture

The project follows a three-layer architecture:

```
src/
├── lib.rs                       # Entry point, conditional Far API export
├── far/                         # LAYER 1: FAR Manager interaction (Dual-API)
│   ├── far3/                    # Implementation for FAR 3 (Windows)
│   │   ├── api.rs               # Far Plugin SDK 3.0 bindings
│   │   └── exports.rs           # Exported C ABI functions
│   ├── far2/                    # Implementation for far2l/far2m (Linux/macOS)
│   │   ├── api.rs               # far2l Plugin API bindings
│   │   └── exports.rs           # Exported C ABI functions
│   ├── string_utils.rs          # Cross-platform string handling (u16/u32)
│   ├── traits.rs                # FarHost trait for API abstraction
│   ├── panels.rs                # Virtual file panel logic (VFS, navigation, commit)
│   ├── ui.rs                    # UI elements (dialogs, progress bars, menus)
│   ├── lang.rs                  # Localization via .lng files
│   └── settings.rs              # Plugin settings (unpack style, backup)
├── v8/                          # LAYER 2: 1C artifact semantics
│   ├── container.rs             # CF container reader (ImageHeader, rows, pages)
│   ├── vfs_builder.rs           # VFS tree builder from container rows
│   ├── writer.rs                # CF container writer (repacking)
│   ├── uuids.rs                 # 1C metadata object type UUIDs
│   └── tests.rs                 # Integration tests
└── base/                        # LAYER 3: Low-level primitives (I/O, parsing)
    ├── reader.rs                # Abstract reader (FileReader, StringReader)
    ├── parser.rs                # Bracket-format parser for 1C metadata
    └── deflate.rs               # DEFLATE via flate2
```

## Current Status

| Phase | Description | Status |
|-------|-------------|--------|
| Phase 0 | Infrastructure setup | ✅ Complete |
| Phase 1 | Plugin skeleton + CF parser | ✅ Complete |
| Phase 2 | EPF/ERF: VFS tree, F3/F4, CF-writer, settings, localization | ✅ Complete |
| Phase 3 | Documentation & GitHub primary publication | ✅ Complete |
| Phase 4A| Dual-API refactoring (FAR 3) | ✅ Complete |
| Phase 4B| far2l basic integration | ✅ Complete |
| Phase 5 | Documentation update V2 & CI | 🔄 In progress |
| Phase 6 | Linux version (build, stubs, testing) | 🔜 Planned |
| Phase 7 | CF/CFE: metadata hierarchy, cascading rebuild | 🔜 Planned |
| Phase 8 | Protected modules: bytecode disassembler | 🔜 Planned |
| Phase 9 | 1CD: file database navigation | 🔜 Planned |
| Phase 10| Polish: lazy loading, large files | 🔜 Planned |

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
