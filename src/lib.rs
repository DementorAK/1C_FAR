pub mod base;
pub mod far;
pub mod v8_artifacts;

mod version {
    include!(concat!(env!("OUT_DIR"), "/version.rs"));
}

use crate::far::api::*;
use crate::far::STARTUP_INFO;
use crate::far::panels::{PluginPanel, FileType};
use crate::base::reader::FileReader;
use crate::v8_artifacts::vfs_builder::build_vfs;
use std::ptr;
use std::fs::File;
use std::panic;
use log::info;

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
            // Initialize logger
            #[cfg(target_os = "windows")]
            let _ = windebug_logger::init();
            #[cfg(not(target_os = "windows"))]
            let _ = simple_logger::init();
            
            info!("1C:Enterprise Artifacts plugin version {} loaded", env!("CARGO_PKG_VERSION"));
        }
    });
}

#[no_mangle]
pub unsafe extern "system" fn ExitFARW(_info: *const ExitInfo) {
    info!("1C:Enterprise Artifacts plugin unloaded");
}

#[no_mangle]
pub unsafe extern "system" fn GetPluginInfoW(info: *mut PluginInfo) {
    let _ = panic::catch_unwind(|| {
        if info.is_null() {
            return;
        }
        let info = &mut *info;
        info.StructSize = std::mem::size_of::<PluginInfo>();
        info.Flags = 0;

        info.CommandPrefix = Box::leak(to_wide(env!("PLUGIN_PREFIX")).into_boxed_slice()).as_ptr();

        let menu_string = Box::leak(to_wide("1C:Enterprise Artifacts").into_boxed_slice()).as_ptr();
        let strings_arr = Box::leak(Box::new([menu_string]));
        let guids_arr = Box::leak(Box::new([MENU_GUID]));

        info.PluginMenu = PluginMenuItem {
            Guids: guids_arr.as_ptr(),
            Strings: strings_arr.as_ptr(),
            Count: 1,
        };
        
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
    let recognized = path_lower.ends_with(".cf")
        || path_lower.ends_with(".epf")
        || path_lower.ends_with(".erf")
        || path_lower.ends_with(".cfe"); // FR-001: .cfe support
    
    if recognized {
        info!("Artifact recognized: {}", path);
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
        
        if path.is_empty() {
            return ptr::null_mut();
        }

        let file_type = match FileType::from_extension(
            std::path::Path::new(&path).extension().and_then(|e| e.to_str()).unwrap_or("")
        ) {
            Some(t) => t,
            None => return ptr::null_mut(),
        };

        let mut panel = PluginPanel::new(path, file_type);

        // Build VFS tree for EPF/ERF
        match File::open(&panel.path) {
            Ok(file) => {
                let file_size = file.metadata().unwrap().len();
                match FileReader::new(file) {
                    Ok(reader) => {
                        match crate::v8_artifacts::container::read_all_rows(reader, file_size) {
                            Ok(rows_map) => {
                                if let Ok(vfs) = build_vfs(rows_map) {
                                    panel.vfs = vfs;
                                }
                            }
                            Err(_) => {}
                        }
                    }
                    Err(_) => {}
                }
            }
            Err(_) => {}
        }

        let panel_box = Box::new(panel);
        Box::into_raw(panel_box) as HANDLE
    }).unwrap_or(ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "system" fn GetOpenPanelInfoW(info: *mut GetOpenPanelInfo) {
    if info.is_null() {
        return;
    }
    let info = &mut *info;
    info.StructSize = std::mem::size_of::<GetOpenPanelInfo>();
    
    if info.hPanel.is_null() {
        info.PanelTitle = Box::leak(to_wide(" 1C:Enterprise Artifacts ").into_boxed_slice()).as_ptr();
        info.CurDir = Box::leak(to_wide("\\").into_boxed_slice()).as_ptr();
        info.Flags = 1 | 8 | 16;
        return;
    }

    let panel = &*(info.hPanel as *const PluginPanel);

    // FR-003: Panel title format
    let title = panel.panel_title();
    info.PanelTitle = Box::leak(to_wide(&title).into_boxed_slice()).as_ptr();
    
    // CurDir reflects virtual directory
    info.CurDir = Box::leak(to_wide(&panel.cur_dir_str()).into_boxed_slice()).as_ptr();
    
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
            let panel = Box::from_raw(info.hPanel as *mut PluginPanel);
            info!("Closing panel for: {}", panel.path);
        }
    });
}

#[no_mangle]
pub unsafe extern "system" fn GetFindDataW(info: *mut GetFindDataInfo) -> IntPtr {
    panic::catch_unwind(|| {
        if info.is_null() {
            return 0;
        }
        let info = &mut *info;
        if info.hPanel.is_null() {
            return 0;
        }
        let panel = &*(info.hPanel as *const PluginPanel);

        let entries = panel.resolve_current_dir();
        if entries.is_empty() {
            info.ItemsNumber = 0;
            info.PanelItem = ptr::null_mut();
            return 1;
        }

        let (items, _leaked) = crate::far::panels::vfs_to_panel_items(entries);

        let items_boxed = items.into_boxed_slice();
        let len = items_boxed.len();
        let ptr = items_boxed.as_ptr();
        
        std::mem::forget(items_boxed);
        
        info.PanelItem = ptr as *mut PluginPanelItem;
        info.ItemsNumber = len;

        1
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
            let slice_ptr = std::ptr::slice_from_raw_parts_mut(info.PanelItem, info.ItemsNumber);
            let _items = Box::from_raw(slice_ptr);
        }
    });
}

#[no_mangle]
pub unsafe extern "system" fn SetDirectoryW(info: *const SetDirectoryInfo) -> IntPtr {
    panic::catch_unwind(|| {
        if info.is_null() {
            return 0;
        }
        let info = &*info;
        if info.hPanel.is_null() {
            return 0;
        }

        let panel = &mut *(info.hPanel as *mut PluginPanel);

        let mut len = 0;
        while *info.Dir.offset(len) != 0 {
            len += 1;
        }
        let dir_wide = std::slice::from_raw_parts(info.Dir, len as usize);
        let dir = String::from_utf16_lossy(dir_wide);

        if panel.set_directory(&dir) {
            1
        } else {
            0
        }
    }).unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "system" fn GetFilesW(info: *const GetFilesInfo) -> IntPtr {
    panic::catch_unwind(|| {
        if info.is_null() {
            return 0;
        }
        let info = &*info;
        if info.hPanel.is_null() {
            return 0;
        }

        let panel = &*(info.hPanel as *const PluginPanel);
        
        let mut len = 0;
        while *info.DestPath.offset(len) != 0 {
            len += 1;
        }
        let dest_path_wide = std::slice::from_raw_parts(info.DestPath, len as usize);
        let dest_path_str = String::from_utf16_lossy(dest_path_wide);
        let dest_base = std::path::Path::new(&dest_path_str);

        let items = std::slice::from_raw_parts(info.PanelItem, info.ItemsNumber);
        
        for item in items {
            let mut nlen = 0;
            while *item.FileName.offset(nlen) != 0 {
                nlen += 1;
            }
            let name_wide = std::slice::from_raw_parts(item.FileName, nlen as usize);
            let name = String::from_utf16_lossy(name_wide);
            
            if let Some(entry) = panel.find_entry_in_current_dir(&name) {
                let dest_item_path = dest_base.join(&name);
                if let Err(e) = entry.extract_to(&dest_item_path) {
                    info!("Failed to extract {}: {}", name, e);
                    return 0;
                }
            } else {
                return 0;
            }
        }

        1
    }).unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "system" fn PutFilesW(info: *const PutFilesInfo) -> IntPtr {
    panic::catch_unwind(|| {
        if info.is_null() { return 0; }
        let info = &*info;
        if info.hPanel.is_null() { return 0; }
        let panel = &mut *(info.hPanel as *mut PluginPanel);
        
        let items = std::slice::from_raw_parts(info.PanelItem, info.ItemsNumber);
        
        for item in items {
            let mut nlen = 0;
            while *item.FileName.offset(nlen) != 0 {
                nlen += 1;
            }
            let name_wide = std::slice::from_raw_parts(item.FileName, nlen as usize);
            let name = String::from_utf16_lossy(name_wide);
            
            let mut dlen = 0;
            while *info.SrcPath.offset(dlen) != 0 {
                dlen += 1;
            }
            let src_path_wide = std::slice::from_raw_parts(info.SrcPath, dlen as usize);
            let src_path_str = String::from_utf16_lossy(src_path_wide);
            let src_base = std::path::Path::new(&src_path_str);
            let src_file_path = src_base.join(&name);

            if let Ok(new_data) = std::fs::read(&src_file_path) {
                if let Some(entry) = panel.find_entry_in_current_dir_mut(&name) {
                    if entry.update_file_data(new_data) {
                        panel.is_modified = true;
                    }
                }
            }
        }
        
        1
    }).unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "system" fn ProcessPanelEventW(info: *const ProcessPanelEventInfo) -> IntPtr {
    panic::catch_unwind(|| {
        if info.is_null() { return 0; }
        let info = &*info;
        if info.hPanel.is_null() { return 0; }
        let panel = &mut *(info.hPanel as *mut PluginPanel);
        
        match info.Event {
            e if e == FE_CLOSE as isize => {
                if panel.is_modified {
                    let msg_title = crate::far::api::to_wide("Сохранение");
                    let msg_text = crate::far::api::to_wide("Состав контейнера был изменен. Сохранить?");
                    let btn_yes = crate::far::api::to_wide("Да");
                    let btn_no = crate::far::api::to_wide("Нет");
                    let btn_cancel = crate::far::api::to_wide("Отмена");

                    let items = [
                        msg_title.as_ptr(),
                        msg_text.as_ptr(),
                        btn_yes.as_ptr(),
                        btn_no.as_ptr(),
                        btn_cancel.as_ptr(),
                    ];

                    if let Some(psi) = crate::far::STARTUP_INFO {
                        if let Some(message_fn) = psi.Message {
                            let res = message_fn(
                                &PLUGIN_GUID,
                                ptr::null(),
                                FMSG_WARNING,
                                ptr::null(),
                                items.as_ptr(),
                                items.len(),
                                3 // Buttons
                            );

                            match res {
                                0 | 1 => { // Да | Нет
                                    return 0; // Close
                                }
                                _ => { // Отмена / Esc
                                    return 1; // Abort close
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        0
    }).unwrap_or(0)
}
