/// FAR Manager 3 Plugin API implementation (Windows, wchar_t = u16).
///
/// This module contains:
/// - `api`     — full FAR 3 SDK type bindings (structs, enums, function pointers)
/// - `exports` — the `#[no_mangle]` exported functions FAR 3 calls into the plugin

pub mod api;
pub mod exports;
