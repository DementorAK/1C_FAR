use crate::far::far2::api::*;

use crate::far::panels;
use std::ffi::c_void;
use std::panic;

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
    name: *const u32,
    data: *const u8,
    data_size: i32,
    op_mode: i32,
) -> HANDLE {
    std::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn OpenPluginW(open_from: i32, item: isize) -> HANDLE {
    std::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn GetOpenPluginInfoW(h_plugin: HANDLE, info: *mut OpenPluginInfo) {
    // Fill out open plugin info for Far2
}

#[no_mangle]
pub unsafe extern "C" fn GetFindDataW(
    h_plugin: HANDLE,
    panel_item: *mut *mut PluginPanelItem,
    items_number: *mut i32,
    op_mode: i32,
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
    h_plugin: HANDLE,
    panel_item: *mut PluginPanelItem,
    items_number: i32,
) {
}

#[no_mangle]
pub unsafe extern "C" fn SetDirectoryW(h_plugin: HANDLE, dir: *const u32, op_mode: i32) -> i32 {
    0
}

#[no_mangle]
pub unsafe extern "C" fn GetFilesW(
    h_plugin: HANDLE,
    panel_item: *mut PluginPanelItem,
    items_number: i32,
    move_files: i32,
    dest_path: *mut *mut u32,
    op_mode: i32,
) -> i32 {
    0
}

#[no_mangle]
pub unsafe extern "C" fn PutFilesW(
    h_plugin: HANDLE,
    panel_item: *mut PluginPanelItem,
    items_number: i32,
    move_files: i32,
    src_path: *const u32,
    op_mode: i32,
) -> i32 {
    0
}

#[no_mangle]
pub unsafe extern "C" fn ClosePluginW(h_plugin: HANDLE) {}

#[no_mangle]
pub unsafe extern "C" fn ProcessEventW(h_plugin: HANDLE, event: i32, param: *mut c_void) -> i32 {
    0
}

#[no_mangle]
pub unsafe extern "C" fn ConfigureW(item_number: i32) -> i32 {
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
