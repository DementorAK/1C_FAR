
#[cfg(feature = "far3")]
use crate::far::api::IntPtr;
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
                let ptr = gm(&crate::far::PLUGIN_GUID, id as IntPtr);
                if !ptr.is_null() {
                    return crate::far::string_utils::from_wide_ptr(ptr);
                }
            }
        }
        format!("MsgId:{}", id as isize)
    }
}

#[cfg(any(feature = "far2l", feature = "far2m"))]
pub fn get_msg(id: Msg) -> String {
    let res = unsafe {
        if let Some(ref api) = crate::far::STARTUP_INFO {
            if let Some(gm) = api.GetMsg {
                let ptr = gm(api.ModuleNumber, id as i32);
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

    res
}

