#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnpackStyle {
    Raw = 0,
    FullParse = 1,
    V8Unpack = 2,
    Saby = 3,
}

impl Default for UnpackStyle {
    fn default() -> Self {
        UnpackStyle::FullParse
    }
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
    pub fn load() -> Self {
        // TODO: Load from FAR registry/config
        Self::default()
    }

    pub fn save(&self) {
        // TODO: Save to FAR registry/config
    }
}
