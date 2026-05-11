/// Plugin settings.
pub struct PluginSettings {
    /// Whether to create a backup before saving changes.
    pub create_backup: bool,
}

impl Default for PluginSettings {
    fn default() -> Self {
        Self {
            create_backup: true,
        }
    }
}

impl PluginSettings {
    pub fn load() -> Self {
        // TODO: Load from FAR registry/config
        Self::default()
    }
}
