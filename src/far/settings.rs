#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum UnpackStyle {
    Raw = 0,
    #[default]
    FullParse = 1,
    V8Unpack = 2,
    Saby = 3,
}

/// Plugin settings.
pub struct PluginSettings {
    /// Whether to create a backup before saving changes.
    pub create_backup: bool,
    /// Style of unpacking for files.
    pub unpack_style: UnpackStyle,
}

impl Default for PluginSettings {
    fn default() -> Self {
        Self {
            create_backup: true,
            unpack_style: UnpackStyle::default(),
        }
    }
}

impl PluginSettings {
    #[cfg(feature = "far3")]
    pub fn load() -> Self {
        use log::info;
        let mut settings = Self::default();
        unsafe {
            let psi = match crate::far::STARTUP_INFO {
                Some(psi) => psi,
                None => {
                    info!("Settings::load: STARTUP_INFO is None, using defaults");
                    return settings;
                }
            };
            let sctl = match psi.SettingsControl {
                Some(sctl) => sctl,
                None => {
                    info!("Settings::load: SettingsControl is None, using defaults");
                    return settings;
                }
            };

            // INVALID_HANDLE_VALUE is required by FAR 3 SDK for SCTL_CREATE
            let invalid_handle: crate::far::api::HANDLE = -1isize as crate::far::api::HANDLE;

            let mut sc = crate::far::api::FarSettingsCreate {
                StructSize: std::mem::size_of::<crate::far::api::FarSettingsCreate>(),
                Guid: crate::far::PLUGIN_GUID,
                Handle: invalid_handle,
            };
            let create_result = sctl(invalid_handle, crate::far::api::SCTL_CREATE, 0, &mut sc as *mut _ as *mut _);
            if create_result == 0 {
                info!("Settings::load: SCTL_CREATE failed (returned 0), using defaults");
                return settings;
            }
            info!("Settings::load: SCTL_CREATE succeeded, handle={:?}", sc.Handle);

            // Read CreateBackup (QWORD)
            let name_cb = crate::far::string_utils::to_wide("CreateBackup");
            let mut item = crate::far::api::FarSettingsItem {
                StructSize: std::mem::size_of::<crate::far::api::FarSettingsItem>(),
                Root: 0,
                Name: name_cb.as_ptr(),
                Type: crate::far::api::FST_QWORD,
                Value: crate::far::api::FarSettingsValueData { Number: 0 },
            };
            let get_result = sctl(sc.Handle, crate::far::api::SCTL_GET, 0, &mut item as *mut _ as *mut _);
            if get_result != 0 {
                settings.create_backup = item.Value.Number != 0;
                info!("Settings::load: CreateBackup = {} (raw={})", settings.create_backup, item.Value.Number);
            } else {
                info!("Settings::load: CreateBackup not found, using default ({})", settings.create_backup);
            }

            // Read UnpackStyle (QWORD)
            let name_us = crate::far::string_utils::to_wide("UnpackStyle");
            let mut item2 = crate::far::api::FarSettingsItem {
                StructSize: std::mem::size_of::<crate::far::api::FarSettingsItem>(),
                Root: 0,
                Name: name_us.as_ptr(),
                Type: crate::far::api::FST_QWORD,
                Value: crate::far::api::FarSettingsValueData { Number: 0 },
            };
            let get_result2 = sctl(sc.Handle, crate::far::api::SCTL_GET, 0, &mut item2 as *mut _ as *mut _);
            if get_result2 != 0 {
                settings.unpack_style = match item2.Value.Number {
                    0 => UnpackStyle::Raw,
                    1 => UnpackStyle::FullParse,
                    2 => UnpackStyle::V8Unpack,
                    3 => UnpackStyle::Saby,
                    _ => UnpackStyle::default(),
                };
                info!("Settings::load: UnpackStyle = {:?} (raw={})", settings.unpack_style, item2.Value.Number);
            } else {
                info!("Settings::load: UnpackStyle not found, using default ({:?})", settings.unpack_style);
            }

            sctl(sc.Handle, crate::far::api::SCTL_FREE, 0, std::ptr::null_mut());
        }
        settings
    }

    #[cfg(feature = "far3")]
    pub fn save(&self) {
        use log::info;
        unsafe {
            let psi = match crate::far::STARTUP_INFO {
                Some(psi) => psi,
                None => {
                    info!("Settings::save: STARTUP_INFO is None, cannot save");
                    return;
                }
            };
            let sctl = match psi.SettingsControl {
                Some(sctl) => sctl,
                None => {
                    info!("Settings::save: SettingsControl is None, cannot save");
                    return;
                }
            };

            // INVALID_HANDLE_VALUE is required by FAR 3 SDK for SCTL_CREATE
            let invalid_handle: crate::far::api::HANDLE = -1isize as crate::far::api::HANDLE;

            let mut sc = crate::far::api::FarSettingsCreate {
                StructSize: std::mem::size_of::<crate::far::api::FarSettingsCreate>(),
                Guid: crate::far::PLUGIN_GUID,
                Handle: invalid_handle,
            };
            let create_result = sctl(invalid_handle, crate::far::api::SCTL_CREATE, 0, &mut sc as *mut _ as *mut _);
            if create_result == 0 {
                info!("Settings::save: SCTL_CREATE failed (returned 0), cannot save");
                return;
            }
            info!("Settings::save: SCTL_CREATE succeeded, handle={:?}", sc.Handle);

            // Write CreateBackup (QWORD)
            let name_cb = crate::far::string_utils::to_wide("CreateBackup");
            let mut item = crate::far::api::FarSettingsItem {
                StructSize: std::mem::size_of::<crate::far::api::FarSettingsItem>(),
                Root: 0,
                Name: name_cb.as_ptr(),
                Type: crate::far::api::FST_QWORD,
                Value: crate::far::api::FarSettingsValueData { Number: if self.create_backup { 1 } else { 0 } },
            };
            let set_result = sctl(sc.Handle, crate::far::api::SCTL_SET, 0, &mut item as *mut _ as *mut _);
            info!("Settings::save: CreateBackup set={}, result={}", self.create_backup, set_result);

            // Write UnpackStyle (QWORD)
            let name_us = crate::far::string_utils::to_wide("UnpackStyle");
            let mut item2 = crate::far::api::FarSettingsItem {
                StructSize: std::mem::size_of::<crate::far::api::FarSettingsItem>(),
                Root: 0,
                Name: name_us.as_ptr(),
                Type: crate::far::api::FST_QWORD,
                Value: crate::far::api::FarSettingsValueData { Number: self.unpack_style as u64 },
            };
            let set_result2 = sctl(sc.Handle, crate::far::api::SCTL_SET, 0, &mut item2 as *mut _ as *mut _);
            info!("Settings::save: UnpackStyle set={:?}, result={}", self.unpack_style, set_result2);

            sctl(sc.Handle, crate::far::api::SCTL_FREE, 0, std::ptr::null_mut());
            info!("Settings::save: SCTL_FREE done, settings saved");
        }
    }

    #[cfg(feature = "far2")]
    pub fn load() -> Self {
        use log::info;
        let mut settings = Self::default();

        let ini_path = match crate::far::INI_FILE_PATH.lock() {
            Ok(guard) => guard.clone(),
            Err(_) => {
                info!("Settings::load (far2): INI_FILE_PATH lock poisoned, using defaults");
                return settings;
            }
        };
        let ini_path = match ini_path {
            Some(p) => p,
            None => {
                info!("Settings::load (far2): INI_FILE_PATH not set, using defaults");
                return settings;
            }
        };

        unsafe {
            let section = crate::far::string_utils::to_wide("MainSettings");
            let key_backup = crate::far::string_utils::to_wide("CreateBackup");
            let key_style = crate::far::string_utils::to_wide("UnpackStyle");

            let cb_raw = crate::far::api::GetPrivateProfileIntW(
                section.as_ptr(),
                key_backup.as_ptr(),
                1, // default: true
                ini_path.as_ptr(),
            );
            settings.create_backup = cb_raw != 0;
            info!("Settings::load (far2): CreateBackup = {}", settings.create_backup);

            let style_raw = crate::far::api::GetPrivateProfileIntW(
                section.as_ptr(),
                key_style.as_ptr(),
                1, // default: FullParse
                ini_path.as_ptr(),
            );
            settings.unpack_style = match style_raw {
                0 => UnpackStyle::Raw,
                1 => UnpackStyle::FullParse,
                2 => UnpackStyle::V8Unpack,
                3 => UnpackStyle::Saby,
                _ => UnpackStyle::default(),
            };
            info!("Settings::load (far2): UnpackStyle = {:?} (raw={})", settings.unpack_style, style_raw);
        }

        settings
    }

    #[cfg(feature = "far2")]
    pub fn save(&self) {
        use log::info;

        let ini_path = match crate::far::INI_FILE_PATH.lock() {
            Ok(guard) => guard.clone(),
            Err(_) => {
                info!("Settings::save (far2): INI_FILE_PATH lock poisoned, cannot save");
                return;
            }
        };
        let ini_path = match ini_path {
            Some(p) => p,
            None => {
                info!("Settings::save (far2): INI_FILE_PATH not set, cannot save");
                return;
            }
        };

        unsafe {
            let section = crate::far::string_utils::to_wide("MainSettings");
            let key_backup = crate::far::string_utils::to_wide("CreateBackup");
            let key_style = crate::far::string_utils::to_wide("UnpackStyle");

            let val_backup = crate::far::string_utils::to_wide(if self.create_backup { "1" } else { "0" });
            crate::far::api::WritePrivateProfileStringW(
                section.as_ptr(),
                key_backup.as_ptr(),
                val_backup.as_ptr(),
                ini_path.as_ptr(),
            );

            let val_style_str = format!("{}", self.unpack_style as u32);
            let val_style = crate::far::string_utils::to_wide(&val_style_str);
            crate::far::api::WritePrivateProfileStringW(
                section.as_ptr(),
                key_style.as_ptr(),
                val_style.as_ptr(),
                ini_path.as_ptr(),
            );

            info!("Settings::save (far2): saved CreateBackup={}, UnpackStyle={:?}",
                  self.create_backup, self.unpack_style);
        }
    }
}
