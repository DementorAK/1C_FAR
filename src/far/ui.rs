#[cfg(feature = "far3")]
use crate::far::api::*;
#[cfg(feature = "far3")]
use crate::far::lang::{get_msg, Msg};
#[cfg(feature = "far3")]
use crate::far::{PLUGIN_GUID, STARTUP_INFO};
#[cfg(feature = "far3")]
use std::ptr;

#[cfg(feature = "far3")]
pub fn show_progress(title: &str, message: &str, current: usize, total: usize) {
    unsafe {
        if let Some(psi) = STARTUP_INFO {
            let percent = (current * 100).checked_div(total).unwrap_or(0) as u64;

            // 1. Taskbar progress (ACTL_SETPROGRESSVALUE)
            if let Some(ac) = psi.AdvControl {
                ac(
                    &PLUGIN_GUID,
                    ACTL_SETPROGRESSSTATE,
                    PS_NORMAL as IntPtr,
                    ptr::null_mut(),
                );
                ac(
                    &PLUGIN_GUID,
                    ACTL_SETPROGRESSVALUE,
                    0,
                    &percent as *const _ as *mut std::ffi::c_void,
                );
            }

            // 2. Message box progress
            if let Some(msg_fn) = psi.Message {
                let bar_width = 30;
                let filled = (current * bar_width).checked_div(total).unwrap_or(0);
                let mut bar = String::new();
                for _ in 0..filled {
                    bar.push('█');
                }
                for _ in filled..bar_width {
                    bar.push('░');
                }

                let line1 = to_wide(&format!("{}: {}", get_msg(Msg::PackingMessage), message));
                let line2 = to_wide(&format!("{} {}% ({} / {})", bar, percent, current, total));
                let title_wide = to_wide(title);

                let items = [title_wide.as_ptr(), line1.as_ptr(), line2.as_ptr()];

                msg_fn(
                    &PLUGIN_GUID,
                    ptr::null(),
                    FMSG_LEFTALIGN,
                    ptr::null(),
                    items.as_ptr(),
                    items.len(),
                    0,
                );
            }
        }
    }
}

#[cfg(feature = "far3")]
pub fn finish_progress() {
    unsafe {
        if let Some(psi) = STARTUP_INFO {
            if let Some(ac) = psi.AdvControl {
                ac(
                    &PLUGIN_GUID,
                    ACTL_SETPROGRESSSTATE,
                    PS_NOPROGRESS as IntPtr,
                    ptr::null_mut(),
                );
            }
        }
    }
}

use crate::far::settings::PluginSettings;
#[cfg(feature = "far3")]
use crate::far::settings::UnpackStyle;

#[cfg(feature = "far3")]
const INVALID_HANDLE_VALUE: HANDLE = -1isize as HANDLE;

#[cfg(feature = "far3")]
pub fn show_settings_dialog(settings: &PluginSettings) -> Option<PluginSettings> {
    unsafe {
        let psi = STARTUP_INFO?;
        let di = psi.DialogInit?;
        let dr = psi.DialogRun?;
        let df = psi.DialogFree?;
        let sc = psi.SendDlgMessage?;

        let title = to_wide(&get_msg(Msg::SettingsTitle));
        let backup_text = to_wide(&get_msg(Msg::BackupCheckbox));
        let style_label = to_wide(&get_msg(Msg::UnpackStyleLabel));
        let style_raw = to_wide(&get_msg(Msg::UnpackStyleRaw));
        let style_full = to_wide(&get_msg(Msg::UnpackStyleFull));
        let style_v8 = to_wide(&get_msg(Msg::UnpackStyleV8));
        let style_saby = to_wide(&get_msg(Msg::UnpackStyleSaby));
        let ok_text = to_wide(&get_msg(Msg::Ok));
        let cancel_text = to_wide(&get_msg(Msg::Cancel));

        let items = vec![
            // 0: Double box
            FarDialogItem {
                Type: DI_DOUBLEBOX,
                X1: 3,
                Y1: 1,
                X2: 60,
                Y2: 13,
                Data: title.as_ptr(),
                ..Default::default()
            },
            // 1: Checkbox (backup)
            FarDialogItem {
                Type: DI_CHECKBOX,
                X1: 5,
                Y1: 2,
                X2: 0,
                Y2: 0,
                Data: backup_text.as_ptr(),
                Param: FarDialogItemParam {
                    Selected: if settings.create_backup { 1 } else { 0 },
                },
                Flags: DIF_FOCUS,
                ..Default::default()
            },
            // 2: Separator
            FarDialogItem {
                Type: DI_TEXT,
                X1: 5,
                Y1: 3,
                X2: 0,
                Y2: 0,
                Flags: DIF_SEPARATOR,
                ..Default::default()
            },
            // 3: Style label
            FarDialogItem {
                Type: DI_TEXT,
                X1: 5,
                Y1: 4,
                X2: 0,
                Y2: 0,
                Data: style_label.as_ptr(),
                ..Default::default()
            },
            // 4: Radio Raw
            FarDialogItem {
                Type: DI_RADIOBUTTON,
                X1: 7,
                Y1: 5,
                X2: 0,
                Y2: 0,
                Data: style_raw.as_ptr(),
                Param: FarDialogItemParam {
                    Selected: if settings.unpack_style == UnpackStyle::Raw {
                        1
                    } else {
                        0
                    },
                },
                Flags: DIF_GROUP,
                ..Default::default()
            },
            // 5: Radio Full
            FarDialogItem {
                Type: DI_RADIOBUTTON,
                X1: 7,
                Y1: 6,
                X2: 0,
                Y2: 0,
                Data: style_full.as_ptr(),
                Param: FarDialogItemParam {
                    Selected: if settings.unpack_style == UnpackStyle::FullParse {
                        1
                    } else {
                        0
                    },
                },
                ..Default::default()
            },
            // 6: Radio V8
            FarDialogItem {
                Type: DI_RADIOBUTTON,
                X1: 7,
                Y1: 7,
                X2: 0,
                Y2: 0,
                Data: style_v8.as_ptr(),
                Param: FarDialogItemParam {
                    Selected: if settings.unpack_style == UnpackStyle::V8Unpack {
                        1
                    } else {
                        0
                    },
                },
                ..Default::default()
            },
            // 7: Radio Saby
            FarDialogItem {
                Type: DI_RADIOBUTTON,
                X1: 7,
                Y1: 8,
                X2: 0,
                Y2: 0,
                Data: style_saby.as_ptr(),
                Param: FarDialogItemParam {
                    Selected: if settings.unpack_style == UnpackStyle::Saby {
                        1
                    } else {
                        0
                    },
                },
                ..Default::default()
            },
            // 8: Separator
            FarDialogItem {
                Type: DI_TEXT,
                X1: 5,
                Y1: 10,
                X2: 0,
                Y2: 0,
                Flags: DIF_SEPARATOR,
                ..Default::default()
            },
            // 9: OK
            FarDialogItem {
                Type: DI_BUTTON,
                X1: 0,
                Y1: 11,
                X2: 0,
                Y2: 0,
                Data: ok_text.as_ptr(),
                Flags: DIF_CENTERGROUP | DIF_DEFAULTBUTTON,
                ..Default::default()
            },
            // 10: Cancel
            FarDialogItem {
                Type: DI_BUTTON,
                X1: 0,
                Y1: 11,
                X2: 0,
                Y2: 0,
                Data: cancel_text.as_ptr(),
                Flags: DIF_CENTERGROUP,
                ..Default::default()
            },
        ];

        let h_dlg = di(
            &PLUGIN_GUID,
            &GUID::default(),
            -1,
            -1,
            64,
            15,
            ptr::null(),
            items.as_ptr(),
            items.len(),
            0,
            FDLG_NONE,
            None,
            ptr::null_mut(),
        );

        if h_dlg == INVALID_HANDLE_VALUE {
            return None;
        }

        let ret = dr(h_dlg);
        if ret == 9 {
            // OK button index
            let mut new_settings = PluginSettings {
                create_backup: sc(h_dlg, DM_GETCHECK as IntPtr, 1, ptr::null_mut()) != 0,
                ..Default::default()
            };

            if sc(h_dlg, DM_GETCHECK as IntPtr, 4, ptr::null_mut()) != 0 {
                new_settings.unpack_style = UnpackStyle::Raw;
            } else if sc(h_dlg, DM_GETCHECK as IntPtr, 5, ptr::null_mut()) != 0 {
                new_settings.unpack_style = UnpackStyle::FullParse;
            } else if sc(h_dlg, DM_GETCHECK as IntPtr, 6, ptr::null_mut()) != 0 {
                new_settings.unpack_style = UnpackStyle::V8Unpack;
            } else if sc(h_dlg, DM_GETCHECK as IntPtr, 7, ptr::null_mut()) != 0 {
                new_settings.unpack_style = UnpackStyle::Saby;
            }

            df(h_dlg);
            Some(new_settings)
        } else {
            df(h_dlg);
            None
        }
    }
}

#[cfg(feature = "far2")]
pub fn show_progress(_title: &str, _message: &str, _current: usize, _total: usize) {
    // TODO: implement FAR2 progress
}

#[cfg(feature = "far2")]
pub fn finish_progress() {
    // TODO: implement FAR2 progress finish
}

#[cfg(feature = "far2")]
pub fn show_settings_dialog(settings: &PluginSettings) -> Option<PluginSettings> {
    use crate::far::api::*;
    use crate::far::lang::{get_msg, Msg};
    use crate::far::settings::UnpackStyle;
    use crate::far::STARTUP_INFO;
    use std::ptr;

    const INVALID_HANDLE_VALUE: HANDLE = -1isize as HANDLE;

    unsafe {
        let psi = STARTUP_INFO?;
        let di = psi.DialogInit?;
        let dr = psi.DialogRun?;
        let df = psi.DialogFree?;
        let sc = psi.SendDlgMessage?;

        let title = crate::far::string_utils::to_wide(&get_msg(Msg::SettingsTitle));
        let backup_text = crate::far::string_utils::to_wide(&get_msg(Msg::BackupCheckbox));
        let style_label = crate::far::string_utils::to_wide(&get_msg(Msg::UnpackStyleLabel));
        let style_raw = crate::far::string_utils::to_wide(&get_msg(Msg::UnpackStyleRaw));
        let style_full = crate::far::string_utils::to_wide(&get_msg(Msg::UnpackStyleFull));
        let style_v8 = crate::far::string_utils::to_wide(&get_msg(Msg::UnpackStyleV8));
        let style_saby = crate::far::string_utils::to_wide(&get_msg(Msg::UnpackStyleSaby));
        let ok_text = crate::far::string_utils::to_wide(&get_msg(Msg::Ok));
        let cancel_text = crate::far::string_utils::to_wide(&get_msg(Msg::Cancel));

        let mut items = vec![
            FarDialogItem {
                Type: DI_DOUBLEBOX,
                X1: 3, Y1: 1, X2: 60, Y2: 13,
                Focus: 0,
                Param: FarDialogItemParam { Selected: 0 },
                Flags: 0,
                DefaultButton: 0,
                PtrData: title.as_ptr(),
                MaxLen: 0,
            },
            FarDialogItem {
                Type: DI_CHECKBOX,
                X1: 5, Y1: 2, X2: 0, Y2: 0,
                Focus: 1,
                Param: FarDialogItemParam { Selected: if settings.create_backup { 1 } else { 0 } },
                Flags: 0,
                DefaultButton: 0,
                PtrData: backup_text.as_ptr(),
                MaxLen: 0,
            },
            FarDialogItem {
                Type: DI_TEXT,
                X1: 5, Y1: 3, X2: 0, Y2: 0,
                Focus: 0,
                Param: FarDialogItemParam { Selected: 0 },
                Flags: DIF_SEPARATOR,
                DefaultButton: 0,
                PtrData: ptr::null(),
                MaxLen: 0,
            },
            FarDialogItem {
                Type: DI_TEXT,
                X1: 5, Y1: 4, X2: 0, Y2: 0,
                Focus: 0,
                Param: FarDialogItemParam { Selected: 0 },
                Flags: 0,
                DefaultButton: 0,
                PtrData: style_label.as_ptr(),
                MaxLen: 0,
            },
            FarDialogItem {
                Type: DI_RADIOBUTTON,
                X1: 7, Y1: 5, X2: 0, Y2: 0,
                Focus: 0,
                Param: FarDialogItemParam {
                    Selected: if settings.unpack_style == UnpackStyle::Raw { 1 } else { 0 },
                },
                Flags: DIF_GROUP,
                DefaultButton: 0,
                PtrData: style_raw.as_ptr(),
                MaxLen: 0,
            },
            FarDialogItem {
                Type: DI_RADIOBUTTON,
                X1: 7, Y1: 6, X2: 0, Y2: 0,
                Focus: 0,
                Param: FarDialogItemParam {
                    Selected: if settings.unpack_style == UnpackStyle::FullParse { 1 } else { 0 },
                },
                Flags: 0,
                DefaultButton: 0,
                PtrData: style_full.as_ptr(),
                MaxLen: 0,
            },
            FarDialogItem {
                Type: DI_RADIOBUTTON,
                X1: 7, Y1: 7, X2: 0, Y2: 0,
                Focus: 0,
                Param: FarDialogItemParam {
                    Selected: if settings.unpack_style == UnpackStyle::V8Unpack { 1 } else { 0 },
                },
                Flags: 0,
                DefaultButton: 0,
                PtrData: style_v8.as_ptr(),
                MaxLen: 0,
            },
            FarDialogItem {
                Type: DI_RADIOBUTTON,
                X1: 7, Y1: 8, X2: 0, Y2: 0,
                Focus: 0,
                Param: FarDialogItemParam {
                    Selected: if settings.unpack_style == UnpackStyle::Saby { 1 } else { 0 },
                },
                Flags: 0,
                DefaultButton: 0,
                PtrData: style_saby.as_ptr(),
                MaxLen: 0,
            },
            FarDialogItem {
                Type: DI_TEXT,
                X1: 5, Y1: 10, X2: 0, Y2: 0,
                Focus: 0,
                Param: FarDialogItemParam { Selected: 0 },
                Flags: DIF_SEPARATOR,
                DefaultButton: 0,
                PtrData: ptr::null(),
                MaxLen: 0,
            },
            FarDialogItem {
                Type: DI_BUTTON,
                X1: 0, Y1: 11, X2: 0, Y2: 0,
                Focus: 0,
                Param: FarDialogItemParam { Selected: 0 },
                Flags: DIF_CENTERGROUP,
                DefaultButton: 1,
                PtrData: ok_text.as_ptr(),
                MaxLen: 0,
            },
            FarDialogItem {
                Type: DI_BUTTON,
                X1: 0, Y1: 11, X2: 0, Y2: 0,
                Focus: 0,
                Param: FarDialogItemParam { Selected: 0 },
                Flags: DIF_CENTERGROUP,
                DefaultButton: 0,
                PtrData: cancel_text.as_ptr(),
                MaxLen: 0,
            },
        ];

        let h_dlg = di(
            psi.ModuleNumber,
            -1, -1,
            64, 15,
            ptr::null(),
            items.as_mut_ptr(),
            items.len() as u32,
            0,
            0,
            None,
            0,
        );

        if h_dlg.is_null() || h_dlg == INVALID_HANDLE_VALUE {
            return None;
        }

        let ret = dr(h_dlg);
        if ret == 9 {
            let mut new_settings = PluginSettings {
                create_backup: sc(h_dlg, DM_GETCHECK, 1, 0) != 0,
                ..Default::default()
            };

            if sc(h_dlg, DM_GETCHECK, 4, 0) != 0 {
                new_settings.unpack_style = UnpackStyle::Raw;
            } else if sc(h_dlg, DM_GETCHECK, 5, 0) != 0 {
                new_settings.unpack_style = UnpackStyle::FullParse;
            } else if sc(h_dlg, DM_GETCHECK, 6, 0) != 0 {
                new_settings.unpack_style = UnpackStyle::V8Unpack;
            } else if sc(h_dlg, DM_GETCHECK, 7, 0) != 0 {
                new_settings.unpack_style = UnpackStyle::Saby;
            }

            df(h_dlg);
            Some(new_settings)
        } else {
            df(h_dlg);
            None
        }
    }
}
