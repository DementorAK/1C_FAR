// Layer 1: Interaction with Far Manager API

pub mod api;
pub mod panels;

pub static mut STARTUP_INFO: Option<api::PluginStartupInfo> = None;
