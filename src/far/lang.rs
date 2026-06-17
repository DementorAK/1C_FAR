
#[cfg(feature = "far3")]
use crate::far::{api::IntPtr, PLUGIN_GUID};
#[cfg(feature = "far3")]
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
    SaveModifiedMsg = 12,
    No = 13,
}

#[cfg(feature = "far3")]
pub fn get_msg(id: Msg) -> String {
    unsafe {
        if let Some(ref psi) = *std::ptr::addr_of!(STARTUP_INFO) {
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
    let mut logged_mod_num: isize = -1;
    let res = unsafe {
        if let Some(ref api) = *std::ptr::addr_of!(crate::far::FAR_API) {
            logged_mod_num = api.module_number;
            if let Some(gm) = api.get_msg {
                let ptr = gm(api.module_number, id as i32);
                if !ptr.is_null() {
                    let s = crate::far::string_utils::from_wide_ptr(ptr);
                    if s.is_empty() {
                        format!("MsgId:{}", id as isize)
                    } else {
                        s
                    }
                } else {
                    format!("MsgId:{}", id as isize)
                }
            } else {
                format!("MsgId:{}", id as isize)
            }
        } else {
            format!("MsgId:{}", id as isize)
        }
    };
    // Log every request to get_msg
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/far1c_getmsg.log")
        .unwrap();
    use std::io::Write;
    let _ = writeln!(file, "get_msg(id={}) with mod={} returned '{}'", id as isize, logged_mod_num, res);
    res
}

