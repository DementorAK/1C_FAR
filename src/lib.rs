pub mod base;
pub mod far;
pub mod v8_artifacts;

mod version {
    include!(concat!(env!("OUT_DIR"), "/version.rs"));
}

use crate::far::api::*;
use crate::far::STARTUP_INFO;
use std::ptr;
use std::fs::File;
use std::panic;
use crate::base::reader::FileReader;
use crate::v8_artifacts::cf::Container;

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
pub unsafe extern "system" fn GetGlobalInfoW(info: *mut GlobalInfo) {
    let _ = panic::catch_unwind(|| {
        if info.is_null() {
            return;
        }
        let info = &mut *info;
        info.StructSize = std::mem::size_of::<GlobalInfo>();
        info.MinFarVersion = VersionInfo { Major: 3, Minor: 0, Revision: 0, Build: 3000, Stage: 0 };
        info.Version = VersionInfo { 
            Major: env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap_or(0), 
            Minor: env!("CARGO_PKG_VERSION_MINOR").parse().unwrap_or(0), 
            Revision: version::BUILD_NUMBER, 
            Build: env!("CARGO_PKG_VERSION_PATCH").parse().unwrap_or(0), 
            Stage: 0 
        };
        // Explicitly copy GUID to avoid any alignment issues
        unsafe {
            std::ptr::copy_nonoverlapping(&PLUGIN_GUID, &mut info.Guid, 1);
        }
        
        info.Title = Box::leak(to_wide("1C:Enterprise Artifacts").into_boxed_slice()).as_ptr();
        info.Description = Box::leak(to_wide(env!("CARGO_PKG_DESCRIPTION")).into_boxed_slice()).as_ptr();
        info.Author = Box::leak(to_wide(env!("CARGO_PKG_AUTHORS")).into_boxed_slice()).as_ptr();
    });
}

#[no_mangle]
pub unsafe extern "system" fn SetStartupInfoW(info: *const PluginStartupInfo) {
    let _ = panic::catch_unwind(|| {
        if !info.is_null() {
            STARTUP_INFO = Some(*info);
        }
    });
}

#[no_mangle]
pub unsafe extern "system" fn GetPluginInfoW(info: *mut PluginInfo) {
    let _ = panic::catch_unwind(|| {
        if info.is_null() {
            return;
        }
        let info = &mut *info;
        info.StructSize = std::mem::size_of::<PluginInfo>();
        info.Flags = 0; // standard flags

        // Command prefix for command line execution
        info.CommandPrefix = Box::leak(to_wide(env!("PLUGIN_PREFIX")).into_boxed_slice()).as_ptr();

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
    });
}

#[no_mangle]
pub unsafe extern "system" fn AnalyseW(info: *const AnalyseInfo) -> HANDLE {
    if info.is_null() {
        return ptr::null_mut();
    }
    let info = &*info;
    if info.FileName.is_null() {
        return ptr::null_mut();
    }

    let mut len = 0;
    while *info.FileName.offset(len) != 0 {
        len += 1;
    }
    let path_wide = std::slice::from_raw_parts(info.FileName, len as usize);
    let path = String::from_utf16_lossy(path_wide);
    
    let path_lower = path.to_lowercase();
    let recognized = path_lower.ends_with(".cf") || path_lower.ends_with(".epf") || path_lower.ends_with(".erf");
    
    if recognized {
        let wide_path = to_wide(&path);
        let handle = Box::into_raw(Box::new(wide_path)) as HANDLE;
        return handle;
    }
    
    ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn CloseAnalyseW(info: *const CloseAnalyseInfo) {
    let _ = panic::catch_unwind(|| {
        if info.is_null() {
            return;
        }
        let info = &*info;
        if !info.Handle.is_null() {
            let _ = Box::from_raw(info.Handle as *mut Vec<u16>);
        }
    });
}

// Structure to represent our panel instance
struct PluginPanel {
    path: String,
}

#[no_mangle]
pub unsafe extern "system" fn OpenW(info: *const OpenInfo) -> HANDLE {
    panic::catch_unwind(|| {
        if info.is_null() {
            return ptr::null_mut();
        }
        let info = &*info;
        
        let mut path = String::new();
        
        // OPEN_ANALYSE is 9
        if info.OpenFrom == 9 && info.Data != 0 {
            let analyse_info = &*(info.Data as *const OpenAnalyseInfo);
            if !analyse_info.Handle.is_null() {
                let path_wide_ptr = analyse_info.Handle as *mut Vec<u16>;
                let path_wide = &*path_wide_ptr;
                path = String::from_utf16_lossy(path_wide.as_slice().strip_suffix(&[0]).unwrap_or(path_wide));
            }
        }
        
        let panel = Box::new(PluginPanel { path });
        Box::into_raw(panel) as HANDLE
    }).unwrap_or(ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "system" fn GetOpenPanelInfoW(info: *mut GetOpenPanelInfo) {
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
pub unsafe extern "system" fn ClosePanelW(info: *const ClosePanelInfo) {
    let _ = panic::catch_unwind(|| {
        if info.is_null() {
            return;
        }
        let info = &*info;
        if !info.hPanel.is_null() {
            let _ = Box::from_raw(info.hPanel as *mut PluginPanel);
        }
    });
}

#[no_mangle]
pub unsafe extern "system" fn GetFindDataW(info: *mut GetFindDataInfo) -> IntPtr {
    panic::catch_unwind(|| {
        if info.is_null() {
            return 0; // FALSE
        }
        let info = &mut *info;
        if info.hPanel.is_null() {
            return 0;
        }
        let panel = &*(info.hPanel as *const PluginPanel);

        if panel.path.is_empty() {
            info.ItemsNumber = 0;
            info.PanelItem = ptr::null_mut();
            return 1;
        }

        let file = match File::open(&panel.path) {
            Ok(f) => f,
            Err(_) => return 0,
        };

        let reader = match FileReader::new(file) {
            Ok(r) => r,
            Err(_) => return 0,
        };

        let mut container = match Container::new(reader) {
            Ok(c) => c,
            Err(_) => return 0,
        };

        let mut items = Vec::new();
        
        unsafe fn leak_wstr(s: &str) -> *const u16 {
            Box::leak(to_wide(s).into_boxed_slice()).as_ptr()
        }

        for row_res in container.rows() {
            if let Ok(row) = row_res {
                let mut item = PluginPanelItem::default();
                item.FileName = leak_wstr(&row.id);
                item.FileSize = row.data.len() as u64;
                item.FileAttributes = 0x20; // FILE_ATTRIBUTE_ARCHIVE
                items.push(item);
            }
        }

        let items_boxed = items.into_boxed_slice();
        let len = items_boxed.len();
        let ptr = items_boxed.as_ptr();
        
        std::mem::forget(items_boxed);
        
        info.PanelItem = ptr as *mut PluginPanelItem;
        info.ItemsNumber = len;

        1 // TRUE
    }).unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "system" fn FreeFindDataW(info: *const FreeFindDataInfo) {
    let _ = panic::catch_unwind(|| {
        if info.is_null() {
            return;
        }
        let info = &*info;
        if !info.PanelItem.is_null() && info.ItemsNumber > 0 {
            // Reconstruct the fat pointer to the boxed slice
            let slice_ptr = std::ptr::slice_from_raw_parts_mut(info.PanelItem, info.ItemsNumber);
            let _items = Box::from_raw(slice_ptr);
        }
    });
}

#[no_mangle]
pub unsafe extern "system" fn SetDirectoryW(info: *const SetDirectoryInfo) -> IntPtr {
    panic::catch_unwind(|| {
        if info.is_null() {
            return 0; // FALSE
        }
        1 // TRUE
    }).unwrap_or(0)
}
