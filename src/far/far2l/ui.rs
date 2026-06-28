use crate::far::far2l::api::*;
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
        let api = crate::far::STARTUP_INFO?;
        let di = api.DialogInit?;
        let dr = api.DialogRun?;
        let df = api.DialogFree?;
        let sc = api.SendDlgMessage?;
        let module_number = api.ModuleNumber;

        // All wide strings must live until DialogFree — never pass null PtrData!
        // far2l dereferences PtrData without null check => SIGSEGV if null.
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
        let empty = crate::far::string_utils::to_wide("");

        let mut items: [FarDialogItem; 13] = [Default::default(); 13];

        // 0: Double box
        items[0].Type = DI_DOUBLEBOX;
        items[0].X1 = 3;
        items[0].Y1 = 1;
        items[0].X2 = 60;
        items[0].Y2 = 15;
        items[0].PtrData = title.as_ptr();

        // 1: Checkbox (backup)
        items[1].Type = DI_CHECKBOX;
        items[1].X1 = 5;
        items[1].Y1 = 2;
        items[1].X2 = 58;
        items[1].Y2 = 2;
        items[1].Focus = 1;
        items[1].Param.Selected = if settings.create_backup { 1 } else { 0 };
        items[1].PtrData = backup_text.as_ptr();

        // 2: Separator
        items[2].Type = DI_TEXT;
        items[2].X1 = 5;
        items[2].Y1 = 3;
        items[2].Flags = DIF_SEPARATOR;
        items[2].PtrData = empty.as_ptr();

        // 3: Style label
        items[3].Type = DI_TEXT;
        items[3].X1 = 5;
        items[3].Y1 = 4;
        items[3].X2 = 58;
        items[3].Y2 = 4;
        items[3].PtrData = style_label.as_ptr();

        // 4: Radio Raw
        items[4].Type = DI_RADIOBUTTON;
        items[4].X1 = 7;
        items[4].Y1 = 5;
        items[4].X2 = 58;
        items[4].Y2 = 5;
        items[4].Param.Selected = if settings.unpack_style == UnpackStyle::Raw {
            1
        } else {
            0
        };
        items[4].Flags = DIF_GROUP;
        items[4].PtrData = style_raw.as_ptr();

        // 5: Radio Full
        items[5].Type = DI_RADIOBUTTON;
        items[5].X1 = 7;
        items[5].Y1 = 6;
        items[5].X2 = 58;
        items[5].Y2 = 6;
        items[5].Param.Selected = if settings.unpack_style == UnpackStyle::FullParse {
            1
        } else {
            0
        };
        items[5].PtrData = style_full.as_ptr();

        // 6: Radio V8
        items[6].Type = DI_RADIOBUTTON;
        items[6].X1 = 7;
        items[6].Y1 = 7;
        items[6].X2 = 58;
        items[6].Y2 = 7;
        items[6].Param.Selected = if settings.unpack_style == UnpackStyle::V8Unpack {
            1
        } else {
            0
        };
        items[6].PtrData = style_v8.as_ptr();

        // 7: Radio Json
        items[7].Type = DI_RADIOBUTTON;
        items[7].X1 = 7;
        items[7].Y1 = 8;
        items[7].X2 = 58;
        items[7].Y2 = 8;
        items[7].Param.Selected = if settings.unpack_style == UnpackStyle::Json {
            1
        } else {
            0
        };
        items[7].PtrData = style_json.as_ptr();

        // 8: Radio Edt
        items[8].Type = DI_RADIOBUTTON;
        items[8].X1 = 7;
        items[8].Y1 = 9;
        items[8].X2 = 58;
        items[8].Y2 = 9;
        items[8].Param.Selected = if settings.unpack_style == UnpackStyle::Edt {
            1
        } else {
            0
        };
        items[8].PtrData = style_edt.as_ptr();

        // 9: Radio Configurator
        items[9].Type = DI_RADIOBUTTON;
        items[9].X1 = 7;
        items[9].Y1 = 10;
        items[9].X2 = 58;
        items[9].Y2 = 10;
        items[9].Param.Selected = if settings.unpack_style == UnpackStyle::Configurator {
            1
        } else {
            0
        };
        items[9].PtrData = style_configurator.as_ptr();

        // 10: Separator
        items[10].Type = DI_TEXT;
        items[10].X1 = 5;
        items[10].Y1 = 12;
        items[10].Flags = DIF_SEPARATOR;
        items[10].PtrData = empty.as_ptr();

        // 11: OK
        items[11].Type = DI_BUTTON;
        items[11].Y1 = 13;
        items[11].Y2 = 13;
        items[11].Flags = DIF_CENTERGROUP;
        items[11].DefaultButton = 1;
        items[11].PtrData = ok_text.as_ptr();

        // 12: Cancel
        items[12].Type = DI_BUTTON;
        items[12].Y1 = 13;
        items[12].Y2 = 13;
        items[12].Flags = DIF_CENTERGROUP;
        items[12].PtrData = cancel_text.as_ptr();

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
