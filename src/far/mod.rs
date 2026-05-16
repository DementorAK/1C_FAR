// Layer 1: Interaction with Far Manager API

pub mod string_utils;
pub mod traits; // FarHost trait: abstraction over API version differences // Cross-platform wide string utilities (u16/u32)

// API-version-specific submodules (selected at compile time via Cargo features)
#[cfg(feature = "far3")]
pub mod far3; // FAR Manager 3 — bindings + exports

#[cfg(feature = "far2")]
pub mod far2; // far2l / far2m — bindings + exports (Phase 4B)

#[cfg(feature = "far2")]
pub use crate::far::far2::api;
#[cfg(feature = "far3")]
pub use crate::far::far3::api;

pub mod lang;
pub mod panels;
pub mod settings;
pub mod ui;

pub static mut STARTUP_INFO: Option<api::PluginStartupInfo> = None;

pub const PLUGIN_GUID: api::GUID = api::GUID {
    Data1: 0x1c1c1c1c,
    Data2: 0x1c1c,
    Data3: 0x1c1c,
    Data4: [0x1c, 0x1c, 0x1c, 0x1c, 0x1c, 0x1c, 0x1c, 0x1c],
};

pub const MENU_GUID: api::GUID = api::GUID {
    Data1: 0x1c1c1c1d,
    Data2: 0x1c1c,
    Data3: 0x1c1c,
    Data4: [0x1c, 0x1c, 0x1c, 0x1c, 0x1c, 0x1c, 0x1c, 0x1d],
};
