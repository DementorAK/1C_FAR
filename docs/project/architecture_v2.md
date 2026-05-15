# Архитектура мультиплатформенного плагина v2: Dual-API (FAR 3 + far2l)

## 1. Проблема

Текущая имплементация плагина использует **исключительно FAR 3 Plugin API** (Windows):
- Все функции экспортируются в формате FAR 3 (`GetGlobalInfoW`, `OpenW`, `AnalyseW`)
- Структуры данных соответствуют FAR 3 SDK (`GlobalInfo`, `OpenInfo`, `AnalyseInfo` с GUIDs)
- Типы данных — Windows-native (`wchar_t` = 2 байта = UTF-16)

Проекты **far2l** и **far2m** используют **FAR 2 Plugin API**, который **несовместим** с FAR 3:
- Другие экспортируемые функции (`OpenPluginW`, `OpenFilePluginW` вместо `OpenW`)
- Нет `GetGlobalInfoW` (нет GUID-системы)
- Другие структуры (`OpenPluginInfo` вместо `GetOpenPanelInfo`)
- `wchar_t` = 4 байта = UTF-32 на Linux
- Другая ABI (`extern "C"` вместо `extern "system"`)

## 2. Сравнительный анализ API

### 2.1 Экспортируемые функции

| Функция (FAR 3) | Функция (far2l / FAR 2) | Различия |
|---|---|---|
| `GetGlobalInfoW(*GlobalInfo)` | ❌ Отсутствует | FAR 2 не имеет GUID-системы |
| `SetStartupInfoW(*PluginStartupInfo)` | `SetStartupInfoW(*PluginStartupInfo)` | **Разные** структуры `PluginStartupInfo` |
| `GetPluginInfoW(*PluginInfo)` | `GetPluginInfoW(*PluginInfo)` | **Разные** структуры `PluginInfo` |
| `OpenW(*OpenInfo) → HANDLE` | `OpenPluginW(OpenFrom, Item) → HANDLE` | Разная сигнатура; FAR 3 упаковывает в структуру |
| `AnalyseW(*AnalyseInfo) → HANDLE` | `OpenFilePluginW(Name, Data, DataSize, OpMode) → HANDLE` | Семантически похожи, но разная сигнатура |
| `CloseAnalyseW(*CloseAnalyseInfo)` | ❌ (нет отдельной функции) | В FAR 2 анализ и открытие совмещены |
| `GetOpenPanelInfoW(*GetOpenPanelInfo)` | `GetOpenPluginInfoW(HANDLE, *OpenPluginInfo)` | **Разные** структуры, другая передача HANDLE |
| `GetFindDataW(*GetFindDataInfo) → IntPtr` | `GetFindDataW(HANDLE, **Items, *Count, OpMode) → int` | FAR 3 упаковывает в структуру |
| `FreeFindDataW(*FreeFindDataInfo)` | `FreeFindDataW(HANDLE, *Items, Count)` | FAR 2 передает параметры напрямую |
| `SetDirectoryW(*SetDirectoryInfo) → IntPtr` | `SetDirectoryW(HANDLE, *Dir, OpMode) → int` | FAR 3 упаковывает в структуру |
| `GetFilesW(*GetFilesInfo) → IntPtr` | `GetFilesW(HANDLE, *Items, Count, Move, **DestPath, OpMode) → int` | FAR 2 передает параметры напрямую |
| `PutFilesW(*PutFilesInfo) → IntPtr` | `PutFilesW(HANDLE, *Items, Count, Move, *SrcPath, OpMode) → int` | FAR 2 передает параметры напрямую |
| `ClosePanelW(*ClosePanelInfo)` | `ClosePluginW(HANDLE)` | FAR 3 упаковывает в структуру |
| `ProcessPanelEventW(*ProcessPanelEventInfo) → IntPtr` | `ProcessEventW(HANDLE, Event, *Param) → int` | Разные сигнатуры |
| `ConfigureW(*ConfigureInfo) → IntPtr` | `ConfigureW(ItemNumber) → int` | FAR 2 передает номер элемента |
| `ExitFARW(*ExitInfo)` | `ExitFARW(void)` | FAR 3 передает структуру |

### 2.2 Ключевые структурные различия

#### PluginPanelItem

```
FAR 3:                              far2l (FAR 2):
─────────                           ─────────────
CreationTime: FILETIME              FindData.ftCreationTime: FILETIME
LastAccessTime: FILETIME            FindData.ftLastAccessTime: FILETIME
LastWriteTime: FILETIME             FindData.ftLastWriteTime: FILETIME
ChangeTime: FILETIME                (отсутствует)
FileSize: u64                       FindData.nFileSize: u64
AllocationSize: u64                 FindData.nPhysicalSize: u64
FileName: *const wchar_t            FindData.lpwszFileName: *const wchar_t
AlternateFileName: *const wchar_t   (отсутствует)
Description: *const wchar_t         Description: *const wchar_t
Owner: *const wchar_t               Owner: *const wchar_t
CustomColumnData: **wchar_t          CustomColumnData: **wchar_t
CustomColumnNumber: usize            CustomColumnNumber: int
Flags: u64                          Flags: DWORD
UserData: UserDataItem               UserData: DWORD_PTR
FileAttributes: DWORD                FindData.dwFileAttributes: DWORD
NumberOfLinks: DWORD                 NumberOfLinks: DWORD
CRC32: DWORD                        CRC32: DWORD
(нет)                                Group: *const wchar_t
(нет)                                FindData.dwUnixMode: DWORD
```

#### PluginInfo

```
FAR 3:                              far2l (FAR 2):
─────────                           ─────────────
StructSize: usize                    StructSize: int
Flags: u64                           Flags: DWORD
PluginMenu: PluginMenuItem           PluginMenuStrings: **wchar_t
  .Guids: *const GUID                PluginMenuStringsNumber: int
  .Strings: **wchar_t               (нет GUID)
  .Count: usize                     
DiskMenu: PluginMenuItem             DiskMenuStrings: **wchar_t
PluginConfig: PluginMenuItem         DiskMenuStringsNumber: int
CommandPrefix: *const wchar_t        PluginConfigStrings: **wchar_t
                                     PluginConfigStringsNumber: int
                                     CommandPrefix: *const wchar_t
                                     SysID: DWORD
```

### 2.3 Системные различия

| Аспект | Windows (FAR 3) | Linux (far2l/far2m) |
|---|---|---|
| `wchar_t` | 2 байта (UTF-16) | 4 байта (UTF-32) |
| Calling convention | `extern "system"` (= `__stdcall` x86, `__cdecl` x64) | `extern "C"` |
| Библиотека | `.dll` | `.so` |
| HANDLE | `*mut c_void` | `*mut c_void` (через WinPort) |
| Идентификация плагина | GUID | SysID (DWORD) + строковые имена |
| Настройки | SQLite через SettingsControl | INI-файлы в ~/.config/far2l/ |
| Разделитель путей | `\` | `/` |
| Message API | `Message(*GUID, *GUID, Flags, ...)` | `Message(PluginNumber, Flags, ...)` |

## 3. Предлагаемая архитектура

### 3.1 Принцип: Feature-gated dual API

Используем **Cargo features** для выбора API-версии при компиляции:

```toml
[features]
default = ["far3"]
far3 = []      # FAR Manager 3 для Windows
far2 = []      # far2l / far2m для Linux/macOS
```

### 3.2 Новая структура модулей

```
src/
├── lib.rs                          -- Условная точка входа
├── far/                            -- СЛОЙ 1: Взаимодействие с Far Manager
│   ├── mod.rs                      -- Общие GUID-ы, глобальное состояние
│   ├── traits.rs                   -- 🆕 Трейт FarHost: абстракция API
│   ├── string_utils.rs             -- 🆕 Кроссплатформенные строковые утилиты
│   ├── far3/                       -- 🆕 Имплементация для FAR 3
│   │   ├── mod.rs
│   │   ├── api.rs                  -- Биндинги FAR 3 SDK (текущий api.rs)
│   │   └── exports.rs              -- 🆕 Экспортируемые функции FAR 3
│   ├── far2/                       -- 🆕 Имплементация для far2l
│   │   ├── mod.rs
│   │   ├── api.rs                  -- 🆕 Биндинги far2l SDK  
│   │   └── exports.rs              -- 🆕 Экспортируемые функции far2l
│   ├── panels.rs                   -- Логика виртуальных панелей (ОБЩАЯ)
│   ├── ui.rs                       -- UI (адаптируется через traits)
│   ├── lang.rs                     -- Локализация (ОБЩАЯ)
│   └── settings.rs                 -- Настройки (адаптируется)
├── v8/                             -- СЛОЙ 2: Бизнес-логика 1С (БЕЗ ИЗМЕНЕНИЙ)
│   ├── mod.rs
│   ├── container.rs
│   ├── vfs_builder.rs
│   ├── writer.rs
│   ├── uuids.rs
│   └── tests.rs
└── base/                           -- СЛОЙ 3: I/O библиотека (БЕЗ ИЗМЕНЕНИЙ)
    ├── mod.rs
    ├── reader.rs
    ├── parser.rs
    └── deflate.rs
```

### 3.3 Ключевой трейт: FarHost

```rust
/// Абстракция над различиями API FAR Manager.
/// Реализуется отдельно для FAR 3 и far2l.
pub trait FarHost {
    /// Тип широкого символа (u16 для Windows, u32 для Linux)
    type WChar: Copy + Default + PartialEq;
    
    /// Конвертация Rust-строки в wide-строку
    fn to_wide(s: &str) -> Vec<Self::WChar>;
    
    /// Конвертация wide-строки обратно в Rust-строку
    fn from_wide(s: &[Self::WChar]) -> String;
    
    /// Показать сообщение пользователю
    fn message(title: &str, text: &str, buttons: &[&str]) -> isize;
    
    /// Показать прогресс-бар
    fn show_progress(title: &str, text: &str, percent: f64);
    
    /// Вызвать встроенный редактор
    fn editor(file_path: &str, title: &str) -> i32;
    
    /// Вызвать встроенный viewer
    fn viewer(file_path: &str, title: &str) -> i32;
    
    /// Обновить панель
    fn update_panel();
    
    /// Получить путь к текущему файлу на файловой панели
    fn get_current_panel_path() -> Option<String>;
    
    /// Логирование
    fn log_info(msg: &str);
}
```

### 3.4 Разделение lib.rs

```rust
// lib.rs — единая точка входа
pub mod base;
pub mod far;
pub mod v8;

// Компиляция нужной версии экспортов:
#[cfg(feature = "far3")]
pub use far::far3::exports::*;

#[cfg(feature = "far2")]
pub use far::far2::exports::*;
```

### 3.5 Экспорты для FAR 3 (текущий код, рефакторинг)

```rust
// far/far3/exports.rs
// Все текущие экспортируемые функции из lib.rs переносятся сюда:
// GetGlobalInfoW, SetStartupInfoW, GetPluginInfoW, OpenW, AnalyseW, ...
// Каждая функция:
//   1. Принимает FAR 3 структуры
//   2. Конвертирует их в общие типы
//   3. Вызывает общую бизнес-логику из panels.rs
```

### 3.6 Экспорты для far2l (новый код)

```rust
// far/far2/exports.rs
// Экспорт функций far2l API:

#[no_mangle]
pub unsafe extern "C" fn SetStartupInfoW(info: *const far2::api::PluginStartupInfo) {
    // Сохранить PluginStartupInfo far2l формата
}

#[no_mangle]
pub unsafe extern "C" fn GetPluginInfoW(info: *mut far2::api::PluginInfo) {
    // Заполнить PluginInfo формата FAR 2
    // Без GUID, со строковыми массивами
}

#[no_mangle]
pub unsafe extern "C" fn OpenFilePluginW(
    name: *const u32,        // wchar_t = u32 на Linux
    data: *const u8,
    data_size: i32,
    op_mode: i32,
) -> HANDLE {
    // Аналог AnalyseW + OpenW из FAR 3
}

#[no_mangle]
pub unsafe extern "C" fn OpenPluginW(open_from: i32, item: isize) -> HANDLE {
    // Аналог OpenW из FAR 3
}

#[no_mangle]
pub unsafe extern "C" fn GetOpenPluginInfoW(
    h_plugin: HANDLE,
    info: *mut far2::api::OpenPluginInfo,
) {
    // Аналог GetOpenPanelInfoW из FAR 3
}

#[no_mangle]
pub unsafe extern "C" fn GetFindDataW(
    h_plugin: HANDLE,
    panel_item: *mut *mut far2::api::PluginPanelItem,
    items_number: *mut i32,
    op_mode: i32,
) -> i32 {
    // Аналог GetFindDataW из FAR 3 (другая сигнатура)
}

// ... остальные функции
```

## 4. Строковые утилиты

```rust
// far/string_utils.rs

/// Конвертация &str -> Vec<u16> (для FAR 3 / Windows)
#[cfg(feature = "far3")]
pub fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Конвертация &str -> Vec<u32> (для far2l / Linux)
#[cfg(feature = "far2")]
pub fn to_wide(s: &str) -> Vec<u32> {
    s.chars().map(|c| c as u32).chain(std::iter::once(0)).collect()
}

/// Конвертация wide-строки обратно в String
#[cfg(feature = "far3")]
pub fn from_wide(ptr: *const u16) -> String {
    unsafe {
        let mut len = 0;
        while *ptr.offset(len) != 0 { len += 1; }
        String::from_utf16_lossy(std::slice::from_raw_parts(ptr, len as usize))
    }
}

#[cfg(feature = "far2")]
pub fn from_wide(ptr: *const u32) -> String {
    unsafe {
        let mut len = 0;
        while *ptr.offset(len) != 0 { len += 1; }
        let slice = std::slice::from_raw_parts(ptr, len as usize);
        slice.iter()
            .filter_map(|&c| char::from_u32(c))
            .collect()
    }
}
```

## 5. Cargo.toml (обновленный)

```toml
[package]
name = "far1c"
version = "0.4.0"
# ...

[features]
default = ["far3"]
far3 = []
far2 = []

[lib]
crate-type = ["cdylib"]

[dependencies]
flate2 = "1.0"
encoding_rs = "0.8"
uuid = { version = "1.3", features = ["v4"] }
widestring = "1.0"
log = "0.4"
chrono = "0.4.44"

[target.'cfg(windows)'.dependencies]
windebug_logger = "0.1"

[target.'cfg(not(windows))'.dependencies]
simple_logger = "4.0"
```

## 6. Стратегия сборки

```bash
# Windows — FAR Manager 3 (текущий вариант, по умолчанию)
cargo build --release

# Linux — far2l / far2m
cargo build --release --features far2 --no-default-features --target x86_64-unknown-linux-gnu
```

## 7. FAR 2 для Windows: Совместимость

> [!NOTE]
> По результатам анализа, far2l/far2m API на **99% совместим** с FAR 2 для Windows. 
> Ключевое отличие — `wchar_t` (2 байта на Windows vs 4 байта на Linux).
> Если нужна поддержка FAR 2 на Windows — потребуется третий feature (`far2-win`), 
> использующий FAR 2 API с `wchar_t` = 2 байта.

Для начала заявляем поддержку:
- ✅ FAR Manager 3 (Windows) — `--features far3` (default)
- ✅ far2l (Linux/macOS) — `--features far2`
- ✅ far2m (Linux) — `--features far2` (совместим с far2l)
- ⏳ FAR Manager 2 (Windows) — отложено, при необходимости `--features far2-win`

## 8. План реализации

### Фаза 4A: Архитектурный рефакторинг (только Windows/FAR 3)

| # | Задача | Часы | Критерий |
|---|--------|------|----------|
| 4A.1 | Создать `far/traits.rs` с общим трейтом `FarHost` | 2 | Трейт определен |
| 4A.2 | Создать `far/string_utils.rs` | 2 | `to_wide`/`from_wide` работают |
| 4A.3 | Вынести FAR 3 API в `far/far3/api.rs` (переместить `api.rs`) | 2 | Код перемещен |
| 4A.4 | Вынести FAR 3 экспорты из `lib.rs` в `far/far3/exports.rs` | 4 | `lib.rs` чистый, плагин собирается |
| 4A.5 | Обновить `Cargo.toml` с features | 1 | Features объявлены |
| 4A.6 | Тестирование FAR 3 сборки | 2 | Плагин работает без регрессий |

**Выход 4A:** Плагин на FAR 3 работает как прежде, но код подготовлен к добавлению far2l.

### Фаза 4B: Имплементация far2l API (будущая работа)

| # | Задача | Часы | Критерий |
|---|--------|------|----------|
| 4B.1 | Создать `far/far2/api.rs` — биндинги far2l SDK | 8 | Все нужные типы определены |
| 4B.2 | Создать `far/far2/exports.rs` — экспортируемые функции | 8 | Функции компилируются |
| 4B.3 | Адаптировать `panels.rs` под общий интерфейс | 4 | Панели работают через трейт |
| 4B.4 | Адаптировать `ui.rs` под общий интерфейс | 4 | Диалоги работают через трейт |
| 4B.5 | Сборка на Linux (x86_64-unknown-linux-gnu) | 2 | `.so` файл собирается |
| 4B.6 | Тестирование на far2l (Linux) | 4 | Плагин загружается, показывает .epf |
| 4B.7 | Тестирование на far2m (Linux) | 2 | Плагин работает в far2m |

**Выход 4B:** Плагин работает на Linux (far2l/far2m) с базовым функционалом EPF/ERF.

## 9. Риски и митигация

| Риск | Вероятность | Митигация |
|------|-------------|-----------|
| Несовместимость ABI WinPort с Rust-определениями | Высокая | Тщательная верификация размеров структур через `assert_eq!(size_of::<T>(), ...)` |
| Различия в поведении API между far2l и far2m | Средняя | Тестирование на обеих платформах |
| Сложности кросс-компиляции Linux из Windows | Средняя | Использовать нативную сборку на Linux-хосте |
| Деградация производительности строковых конвертаций | Низкая | Buffer pooling для hot path |

## 10. Влияние на существующий код

> [!IMPORTANT]
> **Слои `v8/` и `base/` остаются полностью без изменений.**
> Вся бизнес-логика парсинга/записи контейнеров 1С платформонезависима.

Изменения затрагивают **только** слой `far/`:
1. `api.rs` → перемещается в `far/far3/api.rs`
2. `lib.rs` → экспорты переносятся в `far/far3/exports.rs`
3. `panels.rs` → минимальная адаптация (убрать прямые ссылки на FAR 3 типы)
4. `ui.rs` → адаптация диалогов через трейт
5. `Cargo.toml` → добавление features

Остальные модули (`settings.rs`, `lang.rs`, `mod.rs`) — **минимальные изменения**.
