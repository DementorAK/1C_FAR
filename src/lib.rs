pub mod base;
pub mod far;
pub mod v8_artifacts;

use crate::far::api::*;
use crate::far::STARTUP_INFO;
use std::ptr;
use std::ffi::c_void;

// Helper to define plugin GUID
const PLUGIN_GUID: GUID = GUID {
    Data1: 0x1c1c1c1c,
    Data2: 0x1c1c,
    Data3: 0x1c1c,
    Data4: [0x1c, 0x1c, 0x1c, 0x1c, 0x1c, 0x1c, 0x1c, 0x1c],
};

const MENU_GUID: GUID = GUID {
    Data1: 0x1c1c1c1d,
    Data2: 0x1c1c,
    Data3: 0x1c1c,
    Data4: [0x1c, 0x1c, 0x1c, 0x1c, 0x1c, 0x1c, 0x1c, 0x1d],
};

#[no_mangle]
pub unsafe extern "C" fn GetGlobalInfoW(info: *mut GlobalInfo) {
    if info.is_null() {
        return;
    }
    let info = &mut *info;
    info.StructSize = std::mem::size_of::<GlobalInfo>();
    info.MinFarVersion = VersionInfo { Major: 3, Minor: 0, Revision: 0, Build: 3000, Stage: 0 };
    info.Version = VersionInfo { Major: 0, Minor: 1, Revision: 0, Build: 1, Stage: 0 };
    info.Guid = PLUGIN_GUID;
    
    // We leak these strings because FAR expects them to be valid for the lifetime of the plugin
    info.Title = Box::leak(to_wide("1C:Enterprise Artifacts (Rust)").into_boxed_slice()).as_ptr();
    info.Description = Box::leak(to_wide("FAR Manager plugin for 1C:Enterprise artifacts").into_boxed_slice()).as_ptr();
    info.Author = Box::leak(to_wide("1C_FAR Team").into_boxed_slice()).as_ptr();
}

#[no_mangle]
pub unsafe extern "C" fn SetStartupInfoW(info: *const PluginStartupInfo) {
    if !info.is_null() {
        STARTUP_INFO = Some(*info);
    }
}

#[no_mangle]
pub unsafe extern "C" fn GetPluginInfoW(info: *mut PluginInfo) {
    if info.is_null() {
        return;
    }
    let info = &mut *info;
    info.StructSize = std::mem::size_of::<PluginInfo>();
    info.Flags = 0; // standard flags

    // Command prefix for command line execution
    info.CommandPrefix = Box::leak(to_wide("1c").into_boxed_slice()).as_ptr();

    // Add to plugin menu (F11)
    let menu_string = Box::leak(to_wide("1C:Enterprise Artifacts").into_boxed_slice()).as_ptr();
    let strings_arr = Box::leak(Box::new([menu_string]));
    
    let guids_arr = Box::leak(Box::new([MENU_GUID]));

    info.PluginMenu = PluginMenuItem {
        Guids: guids_arr.as_ptr(),
        Strings: strings_arr.as_ptr(),
        Count: 1,
    };
    
    // Empty disk menu and config menu
    info.DiskMenu = PluginMenuItem { Guids: ptr::null(), Strings: ptr::null(), Count: 0 };
    info.PluginConfig = PluginMenuItem { Guids: ptr::null(), Strings: ptr::null(), Count: 0 };
}

#[no_mangle]
pub unsafe extern "C" fn AnalyseW(_info: *const AnalyseInfo) -> HANDLE {
    // For MVP, if it gets here, we just accept files we know. We'll implement real logic later.
    // Return null handle means we don't process it (for now to prevent FAR crashing if we don't handle OpenW properly)
    ptr::null_mut()
}

// Structure to represent our panel instance
struct PluginPanel {
    // In the future, this will hold state about the opened 1C artifact
}

#[no_mangle]
pub unsafe extern "C" fn OpenW(info: *const OpenInfo) -> HANDLE {
    if info.is_null() {
        return ptr::null_mut();
    }
    
    // For now, we always open a panel, regardless of where we were called from (F11, command line, etc.)
    let panel = Box::new(PluginPanel {});
    Box::into_raw(panel) as HANDLE
}

#[no_mangle]
pub unsafe extern "C" fn GetOpenPanelInfoW(info: *mut GetOpenPanelInfo) {
    if info.is_null() {
        return;
    }
    let info = &mut *info;
    info.StructSize = std::mem::size_of::<GetOpenPanelInfo>();
    
    // Set panel title
    info.PanelTitle = Box::leak(to_wide(" [ 1C:Enterprise Artifacts ] ").into_boxed_slice()).as_ptr();
    
    // Set current directory - very important!
    info.CurDir = Box::leak(to_wide("\\").into_boxed_slice()).as_ptr();
    
    // OPIF_ADDDOTS (1) | OPIF_USEFILTER (8) | OPIF_USESORTGROUPS (16)
    info.Flags = 1 | 8 | 16; 
}

#[no_mangle]
pub unsafe extern "C" fn ClosePanelW(info: *const c_void) {
    // info here is the handle we returned from OpenW
    if !info.is_null() {
        let _ = Box::from_raw(info as *mut PluginPanel);
    }
}

#[no_mangle]
pub unsafe extern "C" fn GetFindDataW(info: *mut GetFindDataInfo) -> IntPtr {
    if info.is_null() {
        return 0; // FALSE
    }
    let info = &mut *info;

    // Create dummy files for the panel
    let mut items = Vec::with_capacity(2);
    
    unsafe fn leak_wstr(s: &str) -> *const u16 {
        Box::leak(to_wide(s).into_boxed_slice()).as_ptr()
    }

    let mut item1 = PluginPanelItem::default();
    item1.FileName = leak_wstr("test_module.bsl");
    item1.FileSize = 1024;
    items.push(item1);

    let mut item2 = PluginPanelItem::default();
    item2.FileName = leak_wstr("Forms");
    item2.FileAttributes = 0x10; // FILE_ATTRIBUTE_DIRECTORY
    items.push(item2);

    let items_boxed = items.into_boxed_slice();
    let len = items_boxed.len();
    let ptr = items_boxed.as_ptr();
    
    // We must leak the array itself so it stays valid until FreeFindDataW
    std::mem::forget(items_boxed);
    
    // In this SDK version, PanelItem and ItemsNumber are fields we fill directly
    info.PanelItem = ptr as *mut PluginPanelItem;
    info.ItemsNumber = len;

    1 // TRUE
}

#[no_mangle]
pub unsafe extern "C" fn FreeFindDataW(info: *const FreeFindDataInfo) {
    if info.is_null() {
        return;
    }
    let info = &*info;
    if !info.PanelItem.is_null() && info.ItemsNumber > 0 {
        let _items = Box::from_raw(std::slice::from_raw_parts_mut(info.PanelItem, info.ItemsNumber));
    }
}

#[no_mangle]
pub unsafe extern "C" fn SetDirectoryW(info: *const SetDirectoryInfo) -> IntPtr {
    if info.is_null() {
        return 0; // FALSE
    }
    
    // For the dummy implementation, we allow navigating everywhere.
    // In the real one, we'll update the internal state of the panel.
    1 // TRUE
}
