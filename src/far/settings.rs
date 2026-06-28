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
                return settings;
            }

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
                return;
            }

            // Write CreateBackup (QWORD)
            let name_cb = crate::far::string_utils::to_wide("CreateBackup");
            let mut item = crate::far::api::FarSettingsItem {
                StructSize: std::mem::size_of::<crate::far::api::FarSettingsItem>(),
                Root: 0,
                Name: name_cb.as_ptr(),
                Type: crate::far::api::FST_QWORD,
                Value: crate::far::api::FarSettingsValueData { Number: if self.create_backup { 1 } else { 0 } },
            };
            sctl(sc.Handle, crate::far::api::SCTL_SET, 0, &mut item as *mut _ as *mut _);

            // Write UnpackStyle (QWORD)
            let name_us = crate::far::string_utils::to_wide("UnpackStyle");
            let mut item2 = crate::far::api::FarSettingsItem {
                StructSize: std::mem::size_of::<crate::far::api::FarSettingsItem>(),
                Root: 0,
                Name: name_us.as_ptr(),
                Type: crate::far::api::FST_QWORD,
                Value: crate::far::api::FarSettingsValueData { Number: self.unpack_style as u64 },
            };
            sctl(sc.Handle, crate::far::api::SCTL_SET, 0, &mut item2 as *mut _ as *mut _);

            sctl(sc.Handle, crate::far::api::SCTL_FREE, 0, std::ptr::null_mut());
        }
    }

    #[cfg(any(feature = "far2l", feature = "far2m"))]
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

        let content = match std::fs::read_to_string(&ini_path) {
            Ok(c) => c,
            Err(e) => {
                info!("Settings::load (far2): cannot read {}: {}, using defaults", ini_path, e);
                return settings;
            }
        };

        let mut in_section = false;
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with('[') {
                in_section = line.eq_ignore_ascii_case("[mainsettings]");
                continue;
            }
            if !in_section {
                continue;
            }
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();
                match key {
                    "CreateBackup" => {
                        settings.create_backup = value != "0";
                    }
                    "UnpackStyle" => {
                        let raw: u32 = value.parse().unwrap_or(1);
                        settings.unpack_style = match raw {
                            0 => UnpackStyle::Raw,
                            1 => UnpackStyle::FullParse,
                            2 => UnpackStyle::V8Unpack,
                            3 => UnpackStyle::Saby,
                            _ => UnpackStyle::default(),
                        };
                    }
                    _ => {}
                }
            }
        }

        settings
    }

    #[cfg(any(feature = "far2l", feature = "far2m"))]
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

        let content = format!(
            "[MainSettings]\nCreateBackup={}\nUnpackStyle={}\n",
            if self.create_backup { 1 } else { 0 },
            self.unpack_style as u32
        );

        let _ = std::fs::write(&ini_path, &content);
    }
}
