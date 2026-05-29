use crate::far::api::IntPtr;
#[cfg(feature = "far3")]
use crate::far::PLUGIN_GUID;
use crate::far::STARTUP_INFO;

#[repr(isize)]
#[derive(Clone, Copy, Debug)]
pub enum Msg {
    PluginTitle = 0,
    SettingsTitle = 1,
    BackupCheckbox = 2,
    UnpackStyleLabel = 3,
    UnpackStyleRaw = 4,
    UnpackStyleFull = 5,
    UnpackStyleV8 = 6,
    UnpackStyleSaby = 7,
    Ok = 8,
    Cancel = 9,
    SavingTitle = 10,
    PackingMessage = 11,
}

#[cfg(feature = "far3")]
pub fn get_msg(id: Msg) -> String {
    unsafe {
        if let Some(psi) = STARTUP_INFO {
            if let Some(gm) = psi.GetMsg {
                let ptr = gm(&PLUGIN_GUID, id as IntPtr);
                if !ptr.is_null() {
                    return crate::far::string_utils::from_wide_ptr(ptr);
                }
            }
        }
        format!("MsgId:{}", id as isize)
    }
}

#[cfg(feature = "far2")]
pub fn get_msg(id: Msg) -> String {
    unsafe {
        if let Some(psi) = STARTUP_INFO {
            if let Some(gm) = psi.GetMsg {
                let ptr = gm(psi.ModuleNumber, id as i32);
                if !ptr.is_null() {
                    return crate::far::string_utils::from_wide_ptr(ptr);
                }
            }
        }
        format!("MsgId:{}", id as isize)
    }
}
