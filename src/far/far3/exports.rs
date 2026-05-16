#![allow(clippy::missing_safety_doc)]

use crate::base::reader::FileReader;
/// FAR Manager 3 exported plugin functions (C ABI entry points).
///
/// FAR 3 calls these functions by name via the plugin DLL export table.
/// All functions use `extern "system"` (= `__stdcall` on x86, `__cdecl` on x64).
///
/// This module is only compiled when `feature = "far3"` is active.
/// It imports the business-logic helpers from `panels`, `ui`, `lang`, and `v8`,
/// which are shared between API versions.
use crate::far::far3::api::*;
use crate::far::panels::{FileType, PluginPanel};
use crate::far::settings::PluginSettings;
use crate::far::{MENU_GUID, PLUGIN_GUID, STARTUP_INFO};
use crate::v8::vfs_builder::build_vfs;
use log::info;
use std::collections::HashMap;
use std::fs::File;
use std::panic;
use std::ptr;

mod version {
    include!(concat!(env!("OUT_DIR"), "/version.rs"));
}

// ── Global plugin info ────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "system" fn GetGlobalInfoW(info: *mut GlobalInfo) {
    let _ = panic::catch_unwind(|| {
        if info.is_null() {
            return;
        }
        let info = &mut *info;
        info.StructSize = std::mem::size_of::<GlobalInfo>();
        info.MinFarVersion = VersionInfo {
            Major: 3,
            Minor: 0,
            Revision: 0,
            Build: 3000,
            Stage: 0,
        };
        info.Version = VersionInfo {
            Major: env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap_or(0),
            Minor: env!("CARGO_PKG_VERSION_MINOR").parse().unwrap_or(0),
            Revision: version::BUILD_NUMBER,
            Build: env!("CARGO_PKG_VERSION_PATCH").parse().unwrap_or(0),
            Stage: 0,
        };
        unsafe {
            std::ptr::copy_nonoverlapping(&PLUGIN_GUID, &mut info.Guid, 1);
        }
        info.Title = Box::leak(to_wide("1C:Enterprise Artifacts").into_boxed_slice()).as_ptr();
        info.Description =
            Box::leak(to_wide(env!("CARGO_PKG_DESCRIPTION")).into_boxed_slice()).as_ptr();
        info.Author = Box::leak(to_wide(env!("CARGO_PKG_AUTHORS")).into_boxed_slice()).as_ptr();
    });
}

#[no_mangle]
pub unsafe extern "system" fn SetStartupInfoW(info: *const PluginStartupInfo) {
    let _ = panic::catch_unwind(|| {
        if !info.is_null() {
            STARTUP_INFO = Some(*info);
            #[cfg(target_os = "windows")]
            let _ = windebug_logger::init();
            #[cfg(not(target_os = "windows"))]
            let _ = simple_logger::init();
            info!(
                "1C:Enterprise Artifacts plugin version {} loaded",
                env!("CARGO_PKG_VERSION")
            );
        }
    });
}

#[no_mangle]
pub unsafe extern "system" fn ExitFARW(_info: *const ExitInfo) {
    info!("1C:Enterprise Artifacts plugin unloaded");
}

// ── Plugin metadata ───────────────────────────────────────────────────────────

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

        let menu_title = crate::far::lang::get_msg(crate::far::lang::Msg::PluginTitle);
        let menu_string = Box::leak(to_wide(&menu_title).into_boxed_slice()).as_ptr();
        let strings_arr = Box::leak(Box::new([menu_string]));
        let guids_arr = Box::leak(Box::new([MENU_GUID]));

        info.PluginMenu = PluginMenuItem {
            Guids: guids_arr.as_ptr(),
            Strings: strings_arr.as_ptr(),
            Count: 1,
        };
        info.DiskMenu = PluginMenuItem {
            Guids: ptr::null(),
            Strings: ptr::null(),
            Count: 0,
        };
        info.PluginConfig = PluginMenuItem {
            Guids: guids_arr.as_ptr(),
            Strings: strings_arr.as_ptr(),
            Count: 1,
        };
    });
}

// ── File analysis & open ──────────────────────────────────────────────────────

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
        || path_lower.ends_with(".cfe");

    if recognized {
        info!("Artifact recognized: {}", path);
        let wide_path = to_wide(&path);
        return Box::into_raw(Box::new(wide_path)) as HANDLE;
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

        // OPEN_ANALYSE = 9
        if info.OpenFrom == 9 && info.Data != 0 {
            let analyse_info = &*(info.Data as *const OpenAnalyseInfo);
            if !analyse_info.Handle.is_null() {
                let path_wide_ptr = analyse_info.Handle as *mut Vec<u16>;
                let path_wide = &*path_wide_ptr;
                path = String::from_utf16_lossy(
                    path_wide.as_slice().strip_suffix(&[0]).unwrap_or(path_wide),
                );
            }
        } else if info.OpenFrom == 1 {
            // OPEN_PLUGINSMENU
            if let Some(current_path) = get_current_path() {
                path = current_path;
            }
        }

        if path.is_empty() {
            return ptr::null_mut();
        }

        let file_type = match FileType::from_extension(
            std::path::Path::new(&path)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or(""),
        ) {
            Some(t) => t,
            None => return ptr::null_mut(),
        };

        let mut panel = PluginPanel::new(path, file_type);

        if let Ok(file) = File::open(&panel.path) {
            if let Ok(mut reader) = FileReader::new(file) {
                if let Ok(header) = crate::v8::container::read_image_header(&mut reader, 0) {
                    panel.page_size = header.page_size;
                    panel.is_64bit = header.header_size == 20;
                }
                if let Ok(rows) = crate::v8::container::read_container_rows(reader, 0) {
                    let mut rows_map = HashMap::new();
                    let mut packed_map = HashMap::new();
                    for (id, (data, packed)) in rows {
                        rows_map.insert(id.clone(), data);
                        packed_map.insert(id, packed);
                    }
                    if let Ok(vfs) = build_vfs(&rows_map) {
                        panel.vfs = vfs;
                        panel.rows_map = rows_map;
                        panel.packed_map = packed_map;
                    }
                }
            }
        }

        Box::into_raw(Box::new(panel)) as HANDLE
    })
    .unwrap_or(ptr::null_mut())
}

// ── Panel info ────────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "system" fn GetOpenPanelInfoW(info: *mut GetOpenPanelInfo) {
    if info.is_null() {
        return;
    }
    let info = &mut *info;
    info.StructSize = std::mem::size_of::<GetOpenPanelInfo>();

    if info.hPanel.is_null() {
        info.PanelTitle =
            Box::leak(to_wide(" 1C:Enterprise Artifacts ").into_boxed_slice()).as_ptr();
        info.CurDir = Box::leak(to_wide("\\").into_boxed_slice()).as_ptr();
        info.Flags = 1 | 8 | 16;
        return;
    }

    let panel = &*(info.hPanel as *const PluginPanel);
    let title = panel.panel_title();
    info.PanelTitle = Box::leak(to_wide(&title).into_boxed_slice()).as_ptr();
    info.CurDir = Box::leak(to_wide(&panel.cur_dir_str()).into_boxed_slice()).as_ptr();
    info.Flags = 1 | 8 | 16; // OPIF_ADDDOTS | OPIF_USEFILTER | OPIF_USESORTGROUPS
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

// ── Directory listing ─────────────────────────────────────────────────────────

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

        let (items, _leaked) = vfs_to_panel_items(entries);
        let items_boxed = items.into_boxed_slice();
        let len = items_boxed.len();
        let ptr_val = items_boxed.as_ptr();
        std::mem::forget(items_boxed);

        info.PanelItem = ptr_val as *mut PluginPanelItem;
        info.ItemsNumber = len;
        1
    })
    .unwrap_or(0)
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

fn vfs_to_panel_items(
    entries: &[crate::v8::vfs_builder::VfsEntry],
) -> (Vec<PluginPanelItem>, Vec<*const u16>) {
    let mut items = Vec::new();
    let mut leaked_ptrs = Vec::new();

    for entry in entries {
        let mut item = PluginPanelItem::default();
        let wide = crate::far::string_utils::to_wide(entry.name());
        let ptr = Box::leak(wide.into_boxed_slice()).as_ptr();
        item.FileName = ptr;
        leaked_ptrs.push(ptr);

        if entry.is_dir() {
            item.FileAttributes = 0x10;
            item.FileSize = 0;
        } else {
            item.FileAttributes = 0x20;
            item.FileSize = entry.file_data().map(|d| d.len() as u64).unwrap_or(0);
        }
        items.push(item);
    }

    (items, leaked_ptrs)
}

// ── Navigation ────────────────────────────────────────────────────────────────

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
    })
    .unwrap_or(0)
}

// ── File transfer ─────────────────────────────────────────────────────────────

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
    })
    .unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "system" fn PutFilesW(info: *const PutFilesInfo) -> IntPtr {
    panic::catch_unwind(|| {
        if info.is_null() {
            return 0;
        }
        let info = &*info;
        if info.hPanel.is_null() {
            return 0;
        }
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
            let src_file_path = std::path::Path::new(&src_path_str).join(&name);

            if let Ok(new_data) = std::fs::read(&src_file_path) {
                if let Some(entry) = panel.find_entry_in_current_dir_mut(&name) {
                    if entry.update_file_data(new_data) {
                        panel.is_modified = true;
                    }
                }
            }
        }
        1
    })
    .unwrap_or(0)
}

// ── Panel events ──────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "system" fn ProcessPanelEventW(info: *const ProcessPanelEventInfo) -> IntPtr {
    panic::catch_unwind(|| {
        if info.is_null() {
            return 0;
        }
        let info = &*info;
        if info.hPanel.is_null() {
            return 0;
        }
        let panel = &mut *(info.hPanel as *mut PluginPanel);

        if info.Event == FE_CLOSE as isize && panel.is_modified {
            let msg_title = crate::far::far3::api::to_wide("Сохранение");
            let msg_text =
                crate::far::far3::api::to_wide("Состав контейнера был изменен. Сохранить?");
            let btn_yes = crate::far::far3::api::to_wide("Да");
            let btn_no = crate::far::far3::api::to_wide("Нет");
            let btn_cancel = crate::far::far3::api::to_wide("Отмена");

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
                        3,
                    );
                    match res {
                        0 => {
                            // Да
                            if let Err(e) = panel.commit_changes() {
                                info!("Failed to save changes: {}", e);
                                return 1;
                            }
                            return 0;
                        }
                        1 => return 0, // Нет
                        _ => return 1, // Отмена / Esc
                    }
                }
            }
        }
        0
    })
    .unwrap_or(0)
}

// ── Settings dialog ───────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "system" fn ConfigureW(info: *const ConfigureInfo) -> IntPtr {
    if info.is_null() {
        return 0;
    }
    let current_settings = PluginSettings::load();
    if let Some(new_settings) = crate::far::ui::show_settings_dialog(&current_settings) {
        new_settings.save();
        return 1;
    }
    0
}

// ── Internal helper: get current panel path ───────────────────────────────────

unsafe fn get_current_path() -> Option<String> {
    let psi = STARTUP_INFO?;
    let pc = psi.PanelControl?;

    // 1. Get current directory
    let dir_size = pc(PANEL_ACTIVE, FCTL_GETPANELDIRECTORY, 0, ptr::null_mut());
    if dir_size == 0 {
        return None;
    }

    let mut dir_buf = vec![0u8; dir_size as usize];
    let fpd = dir_buf.as_mut_ptr() as *mut FarPanelDirectory;
    (*fpd).StructSize = std::mem::size_of::<FarPanelDirectory>();
    pc(
        PANEL_ACTIVE,
        FCTL_GETPANELDIRECTORY,
        dir_size,
        fpd as *mut std::ffi::c_void,
    );

    let dir_wide = (*fpd).Name;
    if dir_wide.is_null() {
        return None;
    }
    let mut len = 0;
    while *dir_wide.offset(len) != 0 {
        len += 1;
    }
    let dir_str = String::from_utf16_lossy(std::slice::from_raw_parts(dir_wide, len as usize));

    // 2. Get current item
    let item_size = pc(PANEL_ACTIVE, FCTL_GETCURRENTPANELITEM, 0, ptr::null_mut());
    if item_size == 0 {
        return None;
    }

    let mut item_data = vec![0u8; item_size as usize];
    let mut fgpi = FarGetPluginPanelItem {
        StructSize: std::mem::size_of::<FarGetPluginPanelItem>(),
        Size: item_size as usize,
        Item: item_data.as_mut_ptr() as *mut PluginPanelItem,
    };
    pc(
        PANEL_ACTIVE,
        FCTL_GETCURRENTPANELITEM,
        item_size,
        &mut fgpi as *mut _ as *mut std::ffi::c_void,
    );

    let item = &*fgpi.Item;
    let name_wide = item.FileName;
    if name_wide.is_null() {
        return None;
    }
    let mut nlen = 0;
    while *name_wide.offset(nlen) != 0 {
        nlen += 1;
    }
    let name_str = String::from_utf16_lossy(std::slice::from_raw_parts(name_wide, nlen as usize));

    if name_str == ".." {
        return None;
    }

    let mut path = std::path::PathBuf::from(dir_str);
    path.push(name_str);
    Some(path.to_string_lossy().to_string())
}
