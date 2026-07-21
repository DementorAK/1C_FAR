use crate::far::far2m::api::*;
use crate::far::lang::{get_msg, Msg};
use crate::far::settings::{PluginSettings, UnpackStyle};
use std::ptr;

pub fn show_progress(_title: &str, _message: &str, _current: usize, _total: usize) {
    // TODO: implement FAR2 progress
}

pub fn finish_progress() {
    // TODO: implement FAR2 progress finish
}

const INVALID_HANDLE_VALUE: HANDLE = -1isize as HANDLE;

pub fn show_settings_dialog(settings: &PluginSettings) -> Option<PluginSettings> {
    unsafe {
        let api = match crate::far::STARTUP_INFO {
            Some(a) => a,
            None => {
                log::error!("show_settings: STARTUP_INFO is None");
                return None;
            }
        };
        // We will try DialogInitV3 first, then fallback to DialogInit
        let dr = match api.DialogRun {
            Some(f) => f,
            None => {
                log::error!("show_settings: DialogRun is None");
                return None;
            }
        };
        let df = match api.DialogFree {
            Some(f) => f,
            None => {
                log::error!("show_settings: DialogFree is None");
                return None;
            }
        };
        let sc = match api.SendDlgMessage {
            Some(f) => f,
            None => {
                log::error!("show_settings: SendDlgMessage is None");
                return None;
            }
        };
        let module_number = api.ModuleNumber;

        // All wide strings must live until DialogFree — never pass null PtrData!
        // far2m dereferences PtrData without null check => SIGSEGV if null.
        let title = crate::far::string_utils::to_wide(&get_msg(Msg::SettingsTitle));
        let backup_text = crate::far::string_utils::to_wide(&get_msg(Msg::BackupCheckbox));
        let style_label = crate::far::string_utils::to_wide(&get_msg(Msg::UnpackStyleLabel));
        let style_raw = crate::far::string_utils::to_wide(&get_msg(Msg::UnpackStyleRaw));
        let style_full = crate::far::string_utils::to_wide(&get_msg(Msg::UnpackStyleFull));
        let style_v8 = crate::far::string_utils::to_wide(&get_msg(Msg::UnpackStyleV8));
        let style_json = crate::far::string_utils::to_wide(&get_msg(Msg::UnpackStyleJson));
        let style_edt = crate::far::string_utils::to_wide(&get_msg(Msg::UnpackStyleEdt));
        let style_configurator =
            crate::far::string_utils::to_wide(&get_msg(Msg::UnpackStyleConfigurator));
        let ok_text = crate::far::string_utils::to_wide(&get_msg(Msg::Ok));
        let cancel_text = crate::far::string_utils::to_wide(&get_msg(Msg::Cancel));

        let mut items = vec![
            // 0: Double box
            FarDialogItem {
                Type: DI_DOUBLEBOX,
                X1: 3,
                Y1: 1,
                X2: 60,
                Y2: 15,
                PtrData: title.as_ptr(),
                ..Default::default()
            },
            // 1: Checkbox (backup)
            FarDialogItem {
                Type: DI_CHECKBOX,
                X1: 5,
                Y1: 2,
                X2: 58,
                Y2: 2,
                PtrData: backup_text.as_ptr(),
                Param: FarDialogItemParam {
                    Selected: if settings.create_backup { 1 } else { 0 },
                },
                Focus: 1,
                ..Default::default()
            },
            // 2: Separator
            FarDialogItem {
                Type: DI_TEXT,
                X1: 5,
                Y1: 3,
                X2: 58,
                Y2: 3,
                Flags: DIF_SEPARATOR,
                ..Default::default()
            },
            // 3: Style label
            FarDialogItem {
                Type: DI_TEXT,
                X1: 5,
                Y1: 4,
                X2: 58,
                Y2: 4,
                PtrData: style_label.as_ptr(),
                ..Default::default()
            },
            // 4: Radio Raw
            FarDialogItem {
                Type: DI_RADIOBUTTON,
                X1: 7,
                Y1: 5,
                X2: 58,
                Y2: 5,
                PtrData: style_raw.as_ptr(),
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
                X2: 58,
                Y2: 6,
                PtrData: style_full.as_ptr(),
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
                X2: 58,
                Y2: 7,
                PtrData: style_v8.as_ptr(),
                Param: FarDialogItemParam {
                    Selected: if settings.unpack_style == UnpackStyle::V8Unpack {
                        1
                    } else {
                        0
                    },
                },
                ..Default::default()
            },
            // 7: Radio Json
            FarDialogItem {
                Type: DI_RADIOBUTTON,
                X1: 7,
                Y1: 8,
                X2: 58,
                Y2: 8,
                PtrData: style_json.as_ptr(),
                Param: FarDialogItemParam {
                    Selected: if settings.unpack_style == UnpackStyle::Json {
                        1
                    } else {
                        0
                    },
                },
                ..Default::default()
            },
            // 8: Radio Edt
            FarDialogItem {
                Type: DI_RADIOBUTTON,
                X1: 7,
                Y1: 9,
                X2: 58,
                Y2: 9,
                PtrData: style_edt.as_ptr(),
                Param: FarDialogItemParam {
                    Selected: if settings.unpack_style == UnpackStyle::Edt {
                        1
                    } else {
                        0
                    },
                },
                ..Default::default()
            },
            // 9: Radio Configurator
            FarDialogItem {
                Type: DI_RADIOBUTTON,
                X1: 7,
                Y1: 10,
                X2: 58,
                Y2: 10,
                PtrData: style_configurator.as_ptr(),
                Param: FarDialogItemParam {
                    Selected: if settings.unpack_style == UnpackStyle::Configurator {
                        1
                    } else {
                        0
                    },
                },
                ..Default::default()
            },
            // 10: Separator
            FarDialogItem {
                Type: DI_TEXT,
                X1: 5,
                Y1: 12,
                X2: 58,
                Y2: 12,
                Flags: DIF_SEPARATOR,
                ..Default::default()
            },
            // 11: OK
            FarDialogItem {
                Type: DI_BUTTON,
                X1: 0,
                Y1: 13,
                X2: 58,
                Y2: 13,
                PtrData: ok_text.as_ptr(),
                Flags: DIF_CENTERGROUP,
                DefaultButton: 1,
                ..Default::default()
            },
            // 12: Cancel
            FarDialogItem {
                Type: DI_BUTTON,
                X1: 0,
                Y1: 13,
                X2: 58,
                Y2: 13,
                PtrData: cancel_text.as_ptr(),
                Flags: DIF_CENTERGROUP,
                ..Default::default()
            },
        ];

        let di = api.DialogInit?;
        let h_dlg = di(
            module_number,
            -1,
            -1,
            64,
            17,
            ptr::null(), // HelpTopic
            items.as_mut_ptr(),
            items.len() as u32,
            0,
            0,
            None,
            0,
        );

        if h_dlg.is_null() || h_dlg == INVALID_HANDLE_VALUE || h_dlg as usize == 0xFFFFFFFF {
            log::error!(
                "show_settings: DialogInitV3 returned invalid handle: {:?}",
                h_dlg
            );
            return None;
        }

        let ret = dr(h_dlg);

        if ret == 11 {
            // OK button index
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
                new_settings.unpack_style = UnpackStyle::Json;
            } else if sc(h_dlg, DM_GETCHECK, 8, 0) != 0 {
                new_settings.unpack_style = UnpackStyle::Edt;
            } else if sc(h_dlg, DM_GETCHECK, 9, 0) != 0 {
                new_settings.unpack_style = UnpackStyle::Configurator;
            }

            df(h_dlg);
            Some(new_settings)
        } else {
            df(h_dlg);
            None
        }
    }
}
