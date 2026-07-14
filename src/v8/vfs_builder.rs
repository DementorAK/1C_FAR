#![allow(dead_code)]
use std::collections::HashMap;

/// A single entry in the virtual filesystem.
#[derive(Debug, Clone)]
pub enum VfsEntry {
    File {
        name: String,
        data: Vec<u8>,
        is_protected: bool,
        origin_row_id: Option<String>,
        original_container: Option<Vec<u8>>,
    },
    Dir {
        name: String,
        children: Vec<VfsEntry>,
        origin_row_id: Option<String>,
    },
}

impl VfsEntry {
    pub fn name(&self) -> &str {
        match self {
            VfsEntry::File { name, .. } => name,
            VfsEntry::Dir { name, .. } => name,
        }
    }

    pub fn is_dir(&self) -> bool {
        matches!(self, VfsEntry::Dir { .. })
    }

    pub fn children(&self) -> Option<&[VfsEntry]> {
        match self {
            VfsEntry::Dir { children, .. } => Some(children),
            _ => None,
        }
    }

    pub fn find_child(&self, name: &str) -> Option<&VfsEntry> {
        self.children()
            .and_then(|children| children.iter().find(|c| c.name() == name))
    }

    pub fn file_data(&self) -> Option<&[u8]> {
        match self {
            VfsEntry::File { data, .. } => Some(data),
            _ => None,
        }
    }

    /// Recursively extract the entry to the given physical path.
    pub fn extract_to(&self, dest_path: &std::path::Path) -> std::io::Result<()> {
        match self {
            VfsEntry::File { data, .. } => {
                std::fs::write(dest_path, data)?;
            }
            VfsEntry::Dir { children, .. } => {
                if !dest_path.exists() {
                    std::fs::create_dir_all(dest_path)?;
                }
                for child in children {
                    let child_path = dest_path.join(child.name());
                    child.extract_to(&child_path)?;
                }
            }
        }
        Ok(())
    }

    /// Update file data. Returns true if successful (entry is a file).
    pub fn update_file_data(&mut self, new_data: Vec<u8>) -> bool {
        match self {
            VfsEntry::File { data, .. } => {
                *data = new_data;
                true
            }
            _ => false,
        }
    }
}

// ---------------------------------------------------------------------------
// Unified entry point
// ---------------------------------------------------------------------------

/// Build VFS tree for any 1C container (EPF/ERF/CF/CFE).
///
/// Delegates to `ConfiguratorStyle::build_vfs` as the default style.
pub fn build_vfs(rows_map: &HashMap<String, Vec<u8>>) -> Result<Vec<VfsEntry>, BuildVfsError> {
    use crate::v8::styles::PresentationStyle;
    crate::v8::styles::configurator::ConfiguratorStyle.build_vfs(rows_map)
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum BuildVfsError {
    Io(std::io::Error),
    MetadataError(String),
}

impl From<std::io::Error> for BuildVfsError {
    fn from(e: std::io::Error) -> Self {
        BuildVfsError::Io(e)
    }
}

impl std::fmt::Display for BuildVfsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildVfsError::Io(e) => write!(f, "I/O error: {}", e),
            BuildVfsError::MetadataError(msg) => write!(f, "Metadata error: {}", msg),
        }
    }
}

impl std::error::Error for BuildVfsError {}
