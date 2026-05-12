use crate::far::{STARTUP_INFO, PLUGIN_GUID};
use crate::far::api::IntPtr;

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

pub fn get_msg(id: Msg) -> String {
    unsafe {
        if let Some(psi) = STARTUP_INFO {
            if let Some(gm) = psi.GetMsg {
                let ptr = gm(&PLUGIN_GUID, id as IntPtr);
                if !ptr.is_null() {
                    let mut len = 0;
                    while *ptr.offset(len) != 0 {
                        len += 1;
                    }
                    return String::from_utf16_lossy(std::slice::from_raw_parts(ptr, len as usize));
                }
            }
        }
        format!("MsgId:{}", id as isize)
    }
}
