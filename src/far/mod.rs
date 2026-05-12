// Layer 1: Interaction with Far Manager API

pub mod api;
pub mod panels;
pub mod settings;
pub mod ui;
pub mod lang;

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
