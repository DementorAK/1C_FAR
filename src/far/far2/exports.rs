#![allow(clippy::missing_safety_doc)]

use crate::far::far2::api::*;
use crate::far::lang::{get_msg, Msg};
use crate::far::panels::{FileType, PluginPanel};
use crate::far::string_utils::to_wide;


use log::info;
use std::collections::HashMap;
use std::ffi::c_void;
use std::fs::File;
use std::panic;
use std::ptr;

// ── Module init ────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn PluginModuleOpen(_path: *const std::ffi::c_char) {}

#[no_mangle]
pub unsafe extern "C" fn GetMinFarVersionW() -> i32 {
    (2 << 16) | 6
}

#[no_mangle]
pub unsafe extern "system" fn SetStartupInfoW(info: *const PluginStartupInfo) {
    let _ = std::panic::catch_unwind(|| {
        // info!("SetStartupInfoW called");
        if !info.is_null() {
            let struct_size = unsafe { std::ptr::read_unaligned(info as *const i32) };
            let _ = std::fs::write("/tmp/far1c_startup.log", format!("StructSize: {}\n", struct_size));
        }

        // Initialize logging backend (must be before any info!() calls)
        #[cfg(not(target_os = "windows"))]
        let _ = simple_logger::init();

        if info.is_null() {
            return;
        }

        // ── Build FarApi from raw pointer (variant-aware) ──────────
        match crate::far::far2::far_api::FarApi::from_raw_info(info) {
            Some(api) => {
                let module_name_ptr = api.module_name;
                let root_key_ptr = api.root_key;
                let variant = api.variant;

                crate::far::FAR_API = Some(api);

                // Compute path to config.ini for settings persistence
                if !module_name_ptr.is_null() {
                    let module_name = crate::far::string_utils::from_wide_ptr(module_name_ptr);
                    let is_far2m = variant == crate::far::far2::far_api::FarHostVariant::Far2m
                        || module_name.to_lowercase().contains("far2m");

                    let ini_path = if is_far2m {
                        // far2m: save config.ini in plugin directory
                        let parent = std::path::Path::new(&module_name)
                            .parent()
                            .map(|p| p.to_string_lossy().to_string())
                            .unwrap_or_else(|| ".".to_string());
                        std::path::Path::new(&parent).join("config.ini").to_string_lossy().to_string()
                    } else {
                        // far2l: save config.ini in ~/.config/far2l/plugins/<PluginName>/
                        let plugin_name = if !root_key_ptr.is_null() {
                            let root_key = crate::far::string_utils::from_wide_ptr(root_key_ptr);
                            root_key.rsplit('\\').next()
                                .or_else(|| root_key.rsplit('/').next())
                                .unwrap_or("1C_FAR")
                                .to_string()
                        } else {
                            "1C_FAR".to_string()
                        };

                        let xdg_config = std::env::var("XDG_CONFIG_HOME")
                            .unwrap_or_else(|_| {
                                let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
                                format!("{}/.config", home)
                            });

                        let plugin_config_dir = std::path::Path::new(&xdg_config)
                            .join("far2l")
                            .join("plugins")
                            .join(&plugin_name);

                        // Create directory if it doesn't exist
                        let _ = std::fs::create_dir_all(&plugin_config_dir);

                        plugin_config_dir.join("config.ini").to_string_lossy().to_string()
                    };

                    info!("Settings ini path: {}", ini_path);
                    if let Ok(mut guard) = crate::far::INI_FILE_PATH.lock() {
                        *guard = Some(ini_path);
                    }
                }

                info!("1C:Enterprise Artifacts plugin loaded (far2, variant={:?})", variant);
            }
            None => {
                info!("FarApi: failed to build from raw info");
            }
        }
    });
}

// ── Plugin metadata ────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn GetPluginInfoW(info: *mut PluginInfo) {
    let _ = panic::catch_unwind(|| {
        let _ = std::fs::write("/tmp/far1c_getplugin.log", "GetPluginInfoW called\n");
        if info.is_null() {
            return;
        }
        
        let p_info = &mut *info;
        
        // Zero the entire buffer first. We use the size of our struct.
        // It's 88 bytes with the newly added GUIDs.
        let total_size = std::mem::size_of::<PluginInfo>();
        std::ptr::write_bytes(info as *mut u8, 0, total_size);

        // Prepare strings
        let menu_string = Box::leak(
            to_wide(&get_msg(Msg::PluginTitle)).into_boxed_slice(),
        )
        .as_ptr();
        let menu_strings_arr = Box::leak(Box::new([menu_string]));
        let config_string = Box::leak(
            to_wide(&get_msg(Msg::SettingsTitle)).into_boxed_slice(),
        )
        .as_ptr();
        let config_strings_arr = Box::leak(Box::new([config_string]));
        let prefix = Box::leak(
            to_wide(env!("PLUGIN_PREFIX")).into_boxed_slice(),
        )
        .as_ptr();

        p_info.StructSize = total_size as i32;
        p_info.Flags = 0;
        p_info.DiskMenuStrings = ptr::null();
        p_info.Reserved0 = ptr::null_mut();
        p_info.DiskMenuStringsNumber = 0;
        p_info.PluginMenuStrings = menu_strings_arr.as_ptr();
        p_info.PluginMenuStringsNumber = 1;
        p_info.PluginConfigStrings = config_strings_arr.as_ptr();
        p_info.PluginConfigStringsNumber = 1;
        p_info.CommandPrefix = prefix;
        p_info.SysID = 0x1c1c;
    });
}

// ── File open ──────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn OpenFilePluginW(
    name: *const u32,
    _data: *const u8,
    _data_size: i32,
    _op_mode: i32,
) -> HANDLE {
    panic::catch_unwind(|| {
        if name.is_null() {
            return ptr::null_mut();
        }
        let path = crate::far::string_utils::from_wide_ptr(name);
        let path_lower = path.to_lowercase();
        let recognized = path_lower.ends_with(".cf")
            || path_lower.ends_with(".epf")
            || path_lower.ends_with(".erf")
            || path_lower.ends_with(".cfe");
        if !recognized {
            return ptr::null_mut();
        }
        open_artifact_panel(&path)
    })
    .unwrap_or(ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn OpenPluginW(open_from: i32, _item: isize) -> HANDLE {
    panic::catch_unwind(|| {
        if open_from == OPEN_PLUGINSMENU {
            if let Some(path) = get_current_panel_path() {
                let path_lower = path.to_lowercase();
                if path_lower.ends_with(".cf")
                    || path_lower.ends_with(".epf")
                    || path_lower.ends_with(".erf")
                    || path_lower.ends_with(".cfe")
                {
                    return open_artifact_panel(&path);
                }
            }
        }
        ptr::null_mut()
    })
    .unwrap_or(ptr::null_mut())
}

// ── Panel info ─────────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn GetOpenPluginInfoW(h_plugin: HANDLE, info: *mut OpenPluginInfo) {
    let _ = panic::catch_unwind(|| {
        if info.is_null() || h_plugin.is_null() {
            return;
        }

        let p_info = &mut *info;
        let total_size = std::mem::size_of::<OpenPluginInfo>();

        // Zero the entire buffer first
        std::ptr::write_bytes(info as *mut u8, 0, total_size);

        let panel = &*(h_plugin as *const PluginPanel);
        let title = panel.panel_title();

        let panel_title_ptr = Box::leak(
            crate::far::string_utils::to_wide(&title).into_boxed_slice(),
        )
        .as_ptr();
        let cur_dir_ptr = Box::leak(
            crate::far::string_utils::to_wide(&panel.cur_dir_str()).into_boxed_slice(),
        )
        .as_ptr();
        let format_ptr = Box::leak(
            crate::far::string_utils::to_wide("1C").into_boxed_slice(),
        )
        .as_ptr();

        p_info.StructSize = total_size as i32;
        p_info.Flags = OPIF_USEFILTER | OPIF_ADDDOTS | OPIF_RAWSELECTION;
        p_info.HostFile = ptr::null();
        p_info.CurDir = cur_dir_ptr;
        p_info.Format = format_ptr;
        p_info.PanelTitle = panel_title_ptr;
        p_info.InfoLines = ptr::null();
        p_info.InfoLinesNumber = 0;
        p_info.DescrFiles = ptr::null();
        p_info.DescrFilesNumber = 0;
        p_info.PanelModesArray = ptr::null();
        p_info.PanelModesNumber = 0;
        p_info.StartPanelMode = 0;
        p_info.StartSortMode = 0;
        p_info.StartSortOrder = 0;
        p_info.KeyBar = ptr::null();
        p_info.ShortcutData = ptr::null();
        p_info.CurURL = ptr::null();
        p_info.Reserved = 0;
    });
}

// ── Directory listing ──────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn GetFindDataW(
    h_plugin: HANDLE,
    panel_item: *mut *mut PluginPanelItem,
    items_number: *mut i32,
    _op_mode: i32,
) -> i32 {
    panic::catch_unwind(|| {
        if h_plugin.is_null() || panel_item.is_null() || items_number.is_null() {
            return 0;
        }
        let panel = &*(h_plugin as *const PluginPanel);

        let entries = panel.resolve_current_dir();
        if entries.is_empty() {
            *items_number = 0;
            *panel_item = ptr::null_mut();
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
    let _ = panic::catch_unwind(|| {
        if !_panel_item.is_null() && _items_number > 0 {
            let slice_ptr =
                std::ptr::slice_from_raw_parts_mut(_panel_item as *mut PluginPanelItem, _items_number as usize);
            let _items = Box::from_raw(slice_ptr);
        }
    });
}

#[no_mangle]
pub unsafe extern "C" fn SetDirectoryW(h_plugin: HANDLE, dir: *const u32, _op_mode: i32) -> i32 {
    panic::catch_unwind(|| {
        if h_plugin.is_null() || dir.is_null() {
            return 0;
        }
        let panel = &mut *(h_plugin as *mut PluginPanel);
        let dir_str = crate::far::string_utils::from_wide_ptr(dir);
        if panel.set_directory(&dir_str) { 
            1 
        } else { 
            if dir_str == ".." || dir_str == "\\" || dir_str == "/" || dir_str.is_empty() {
                if let Some(ref api) = *std::ptr::addr_of!(crate::far::FAR_API) {
                    if let Some(control) = api.control {
                        control(h_plugin, crate::far::far2::api::FCTL_CLOSEPLUGIN, 0, 0);
                    }
                }
            }
            0 
        }
    })
    .unwrap_or(0)
}

// ── File transfer ──────────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn GetFilesW(
    h_plugin: HANDLE,
    panel_item: *mut PluginPanelItem,
    items_number: i32,
    _move_files: i32,
    _dest_path: *mut *mut u32,
    _op_mode: i32,
) -> i32 {
    panic::catch_unwind(|| {
        if h_plugin.is_null() || panel_item.is_null() || items_number <= 0 {
            return 0;
        }
        let panel = &*(h_plugin as *mut PluginPanel);
        let items = std::slice::from_raw_parts(panel_item, items_number as usize);

        // dest_path is actually const wchar_t **, but declared as *mut *mut u32 for FFI
        if _dest_path.is_null() || (*_dest_path).is_null() {
            return 0;
        }
        let dest_path = crate::far::string_utils::from_wide_ptr(*_dest_path);
        let dest_base = std::path::Path::new(&dest_path);

        for item in items {
            let mut nlen = 0;
            while !item.FindData.lpwszFileName.is_null() && *item.FindData.lpwszFileName.offset(nlen) != 0 {
                nlen += 1;
            }
            let name_wide =
                std::slice::from_raw_parts(item.FindData.lpwszFileName, nlen as usize);
            let name: String = name_wide.iter().filter_map(|&c| char::from_u32(c)).collect();

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
pub unsafe extern "C" fn PutFilesW(
    h_plugin: HANDLE,
    panel_item: *mut PluginPanelItem,
    items_number: i32,
    _move_files: i32,
    src_path: *const u32,
    _op_mode: i32,
) -> i32 {
    panic::catch_unwind(|| {
        if h_plugin.is_null() || panel_item.is_null() || items_number <= 0 || src_path.is_null() {
            return 0;
        }
        let panel = &mut *(h_plugin as *mut PluginPanel);
        let items = std::slice::from_raw_parts(panel_item, items_number as usize);
        let src_path_str = crate::far::string_utils::from_wide_ptr(src_path);

        for item in items {
            let mut nlen = 0;
            while !item.FindData.lpwszFileName.is_null() && *item.FindData.lpwszFileName.offset(nlen) != 0 {
                nlen += 1;
            }
            let name_wide =
                std::slice::from_raw_parts(item.FindData.lpwszFileName, nlen as usize);
            let name: String = name_wide.iter().filter_map(|&c| char::from_u32(c)).collect();

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

// ── Plugin lifecycle ───────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn ClosePluginW(h_plugin: HANDLE) {
    let _ = panic::catch_unwind(|| {
        if !h_plugin.is_null() {
            let panel = Box::from_raw(h_plugin as *mut PluginPanel);
            info!("Closing panel for: {}", panel.path);
        }
    });
}

#[no_mangle]
pub unsafe extern "C" fn ProcessEventW(h_plugin: HANDLE, event: i32, _param: *mut c_void) -> i32 {
    panic::catch_unwind(|| {
        if h_plugin.is_null() {
            return 0;
        }
        let panel = &mut *(h_plugin as *mut PluginPanel);
        
        // FE_CLOSE == 3
        if event == 3 && panel.is_modified {
            let msg_title = to_wide(&get_msg(Msg::SavingTitle));
            let msg_text = to_wide(&get_msg(Msg::SaveModifiedMsg));
            let btn_yes = to_wide(&get_msg(Msg::Ok));
            let btn_no = to_wide(&get_msg(Msg::No));
            let btn_cancel = to_wide(&get_msg(Msg::Cancel));

            let items = [
                msg_title.as_ptr(),
                msg_text.as_ptr(),
                btn_yes.as_ptr(),
                btn_no.as_ptr(),
                btn_cancel.as_ptr(),
            ];

            if let Some(ref api) = *std::ptr::addr_of!(crate::far::FAR_API) {
                if let Some(message_fn) = api.message {
                    let res = message_fn(
                        api.module_number,
                        crate::far::far2::api::FMSG_WARNING,
                        ptr::null(),
                        items.as_ptr(),
                        items.len() as i32,
                        3,
                    );
                    match res {
                        0 => {
                            if let Err(e) = panel.commit_changes() {
                                log::info!("Failed to save changes: {}", e);
                                return 1;
                            }
                            return 0;
                        }
                        1 => return 0,
                        _ => return 1,
                    }
                }
            }
        }
        0
    }).unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "C" fn ConfigureW(_item_number: i32) -> i32 {
    let result = std::panic::catch_unwind(|| {
        info!("ConfigureW: enter, item_number={}", _item_number);
        let current_settings = crate::far::settings::PluginSettings::load();
        info!("ConfigureW: settings loaded, create_backup={}, unpack_style={:?}",
            current_settings.create_backup, current_settings.unpack_style);
        if let Some(new_settings) = crate::far::ui::show_settings_dialog(&current_settings) {
            info!("ConfigureW: saving settings");
            new_settings.save();
            1
        } else {
            info!("ConfigureW: dialog cancelled");
            0
        }
    });
    result.unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "C" fn ExitFARW() {
    let _ = panic::catch_unwind(|| {});
}

// ── Helpers ────────────────────────────────────────────────────────────────

unsafe fn open_artifact_panel(path: &str) -> HANDLE {
    use crate::base::reader::FileReader;
    use crate::v8::vfs_builder::build_vfs;

    let file_type = match FileType::from_extension(
        std::path::Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or(""),
    ) {
        Some(t) => t,
        None => return ptr::null_mut(),
    };

    let mut panel = PluginPanel::new(path.to_string(), file_type);

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

    info!("Opened artifact: {}", path);
    Box::into_raw(Box::new(panel)) as HANDLE
}

unsafe fn get_current_panel_path() -> Option<String> {
    let api = (*std::ptr::addr_of!(crate::far::FAR_API)).as_ref()?;
    let control = api.control?;

    // Get current panel directory
    let mut dir_buf = vec![0u32; 4096];
    let dir_len = control(PANEL_ACTIVE, FCTL_GETPANELDIR, 4096, dir_buf.as_mut_ptr() as IntPtr);
    if dir_len <= 0 {
        return None;
    }
    let dir_len = dir_len as usize;
    let dir_str: String = dir_buf[..dir_len]
        .iter()
        .filter_map(|&c| char::from_u32(c))
        .collect();

    // Get current item
    let item_size = control(PANEL_ACTIVE, FCTL_GETCURRENTPANELITEM, 0, 0);
    if item_size <= 0 {
        return None;
    }
    let mut item_data = vec![0u8; item_size as usize];
    control(
        PANEL_ACTIVE,
        FCTL_GETCURRENTPANELITEM,
        0,
        item_data.as_mut_ptr() as IntPtr,
    );
    let item = std::ptr::read_unaligned(item_data.as_ptr() as *const PluginPanelItem);
    let name_ptr = item.FindData.lpwszFileName;
    if name_ptr.is_null() {
        return None;
    }
    let mut nlen = 0;
    while *name_ptr.offset(nlen) != 0 {
        nlen += 1;
    }
    let name_str: String = std::slice::from_raw_parts(name_ptr, nlen as usize)
        .iter()
        .filter_map(|&c| char::from_u32(c))
        .collect();

    if name_str == ".." {
        return None;
    }

    let mut path = std::path::PathBuf::from(dir_str);
    path.push(name_str);
    Some(path.to_string_lossy().to_string())
}

fn vfs_to_panel_items(
    entries: &[crate::v8::vfs_builder::VfsEntry],
) -> (Vec<PluginPanelItem>, Vec<*const u32>) {
    let mut items = Vec::new();
    let mut leaked_ptrs = Vec::new();

    for entry in entries {
        let mut item = PluginPanelItem::default();
        let wide = crate::far::string_utils::to_wide(entry.name());
        let ptr = Box::leak(wide.into_boxed_slice()).as_ptr();
        item.FindData.lpwszFileName = ptr as *mut u32;
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
