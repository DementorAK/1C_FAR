#![allow(clippy::missing_safety_doc)]

use crate::far::far2::api::*;

use std::ffi::c_void;
use std::panic;

/// far2l-specific: called when the .so is loaded, before SetStartupInfoW.
/// Receives the filesystem path to the plugin module.
#[no_mangle]
pub unsafe extern "C" fn PluginModuleOpen(_path: *const std::ffi::c_char) {
    // far2l calls this at dlopen time to inform the plugin of its own path.
    // Can be used for module-relative resource loading in the future.
}

/// Returns the minimum FAR API version required by this plugin.
/// MAKEFARVERSION(major, minor) = (major << 16) | minor
#[no_mangle]
pub unsafe extern "C" fn GetMinFarVersionW() -> i32 {
    (2 << 16) | 6 // FAR 2.6
}

#[no_mangle]
pub unsafe extern "C" fn SetStartupInfoW(info: *const PluginStartupInfo) {
    if !info.is_null() {
        crate::far::STARTUP_INFO = Some(*info);
    }
}

#[no_mangle]
pub unsafe extern "C" fn GetPluginInfoW(info: *mut PluginInfo) {
    if let Some(info) = info.as_mut() {
        // Just empty stubs for now
        info.Flags = 0;
    }
}

#[no_mangle]
pub unsafe extern "C" fn OpenFilePluginW(
    _name: *const u32,
    _data: *const u8,
    _data_size: i32,
    _op_mode: i32,
) -> HANDLE {
    std::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn OpenPluginW(_open_from: i32, _item: isize) -> HANDLE {
    std::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn GetOpenPluginInfoW(_h_plugin: HANDLE, _info: *mut OpenPluginInfo) {
    // Fill out open plugin info for Far2
}

#[no_mangle]
pub unsafe extern "C" fn GetFindDataW(
    h_plugin: HANDLE,
    panel_item: *mut *mut PluginPanelItem,
    items_number: *mut i32,
    _op_mode: i32,
) -> i32 {
    panic::catch_unwind(|| {
        if h_plugin.is_null() {
            return 0;
        }
        let panel = &*(h_plugin as *const crate::far::panels::PluginPanel);

        let entries = panel.resolve_current_dir();
        if entries.is_empty() {
            *items_number = 0;
            *panel_item = std::ptr::null_mut();
            return 1;
        }

        let (items, _leaked) = vfs_to_panel_items(entries);
        let items_boxed = items.into_boxed_slice();
        let len = items_boxed.len();
        let ptr_val = items_boxed.as_ptr();
        std::mem::forget(items_boxed);

        *panel_item = ptr_val as *mut PluginPanelItem;
        *items_number = len as i32;
        1
    })
    .unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "C" fn FreeFindDataW(
    _h_plugin: HANDLE,
    _panel_item: *mut PluginPanelItem,
    _items_number: i32,
) {
}

#[no_mangle]
pub unsafe extern "C" fn SetDirectoryW(_h_plugin: HANDLE, _dir: *const u32, _op_mode: i32) -> i32 {
    0
}

#[no_mangle]
pub unsafe extern "C" fn GetFilesW(
    _h_plugin: HANDLE,
    _panel_item: *mut PluginPanelItem,
    _items_number: i32,
    _move_files: i32,
    _dest_path: *mut *mut u32,
    _op_mode: i32,
) -> i32 {
    0
}

#[no_mangle]
pub unsafe extern "C" fn PutFilesW(
    _h_plugin: HANDLE,
    _panel_item: *mut PluginPanelItem,
    _items_number: i32,
    _move_files: i32,
    _src_path: *const u32,
    _op_mode: i32,
) -> i32 {
    0
}

#[no_mangle]
pub unsafe extern "C" fn ClosePluginW(_h_plugin: HANDLE) {}

#[no_mangle]
pub unsafe extern "C" fn ProcessEventW(_h_plugin: HANDLE, _event: i32, _param: *mut c_void) -> i32 {
    0
}

#[no_mangle]
pub unsafe extern "C" fn ConfigureW(_item_number: i32) -> i32 {
    0
}

#[no_mangle]
pub unsafe extern "C" fn ExitFARW() {}

fn vfs_to_panel_items(
    entries: &[crate::v8::vfs_builder::VfsEntry],
) -> (Vec<PluginPanelItem>, Vec<*const u32>) {
    let mut items = Vec::new();
    let mut leaked_ptrs = Vec::new();

    for entry in entries {
        let mut item = PluginPanelItem::default();
        let wide = crate::far::string_utils::to_wide(entry.name());
        let ptr = Box::leak(wide.into_boxed_slice()).as_ptr();
        item.FindData.lpwszFileName = ptr;
        leaked_ptrs.push(ptr);

        if entry.is_dir() {
            item.FindData.dwFileAttributes = 0x10;
            item.FindData.nFileSize = 0;
        } else {
            item.FindData.dwFileAttributes = 0x20;
            item.FindData.nFileSize = entry.file_data().map(|d| d.len() as u64).unwrap_or(0);
        }
        items.push(item);
    }

    (items, leaked_ptrs)
}
