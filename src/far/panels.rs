use crate::v8_artifacts::vfs_builder::VfsEntry;
use crate::far::api::*;

/// Supported artifact types.
#[derive(Debug, Clone, Copy)]
pub enum FileType {
    EPF,
    ERF,
    CF,
    CFE,
    OneCD,
}

impl FileType {
    /// Create from file extension (case-insensitive).
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "epf" => Some(FileType::EPF),
            "erf" => Some(FileType::ERF),
            "cf" => Some(FileType::CF),
            "cfe" => Some(FileType::CFE),
            "1cd" => Some(FileType::OneCD),
            _ => None,
        }
    }

    /// Get the uppercase type string for panel titles.
    pub fn as_str(&self) -> &str {
        match self {
            FileType::EPF => "EPF",
            FileType::ERF => "ERF",
            FileType::CF => "CF",
            FileType::CFE => "CFE",
            FileType::OneCD => "1CD",
        }
    }
}

/// Represents an open virtual panel for a 1C artifact.
pub struct PluginPanel {
    /// Full path to the artifact file on disk.
    pub path: String,
    /// Type of artifact.
    pub file_type: FileType,
    /// Virtual filesystem tree (root entries).
    pub vfs: Vec<VfsEntry>,
    /// Navigation stack: each element is a directory name.
    /// Empty = root, ["Forms"] = inside Forms/, ["Forms", "MainForm"] = inside MainForm.
    pub current_dir: Vec<String>,
    /// Just the filename (e.g., "test.epf") for panel title.
    pub host_filename: String,
}

impl PluginPanel {
    /// Create a new panel for the given file.
    pub fn new(path: String, file_type: FileType) -> Self {
        let host_filename = std::path::Path::new(&path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.clone());

        Self {
            path,
            file_type,
            vfs: Vec::new(),
            current_dir: Vec::new(),
            host_filename,
        }
    }

    /// Get the entries in the current virtual directory.
    pub fn resolve_current_dir(&self) -> &[VfsEntry] {
        let mut current: &[VfsEntry] = &self.vfs;

        for dir_name in &self.current_dir {
            if let Some(entry) = current.iter().find(|e| e.name() == dir_name) {
                if let Some(children) = entry.children() {
                    current = children;
                } else {
                    return &[];
                }
            } else {
                return &[];
            }
        }

        current
    }

    /// Navigate to a directory.
    /// `dir` can be ".." to go up, or a directory name to go down.
    /// Returns true if navigation succeeded.
    pub fn set_directory(&mut self, dir: &str) -> bool {
        if dir == ".." {
            if self.current_dir.is_empty() {
                return false; // Already at root
            }
            self.current_dir.pop();
            return true;
        }

        // Try to find the directory in current level
        let current = self.resolve_current_dir();
        if current.iter().any(|e| e.is_dir() && e.name() == dir) {
            self.current_dir.push(dir.to_string());
            return true;
        }

        false
    }

    /// Build the CurDir string for FAR (e.g., "\\Forms\\MainForm").
    pub fn cur_dir_str(&self) -> String {
        if self.current_dir.is_empty() {
            "\\".to_string()
        } else {
            let mut s = String::from("\\");
            for (i, dir) in self.current_dir.iter().enumerate() {
                if i > 0 {
                    s.push('\\');
                }
                s.push_str(dir);
            }
            s
        }
    }

    /// Build the panel title per FR-003: "FAR 1C:<TYPE>:<FileName>"
    pub fn panel_title(&self) -> String {
        format!(" FAR 1C:{}:{} ", self.file_type.as_str(), self.host_filename)
    }
}

/// Convert VFS entries to FAR PluginPanelItems.
/// The strings are boxed and leaked to be owned by FAR until FreeFindDataW.
pub fn vfs_to_panel_items(entries: &[VfsEntry]) -> (Vec<PluginPanelItem>, Vec<*const u16>) {
    let mut items = Vec::new();
    let mut leaked_ptrs = Vec::new(); // track leaked strings for potential cleanup (not needed, FAR frees via our items)

    for entry in entries {
        let mut item = PluginPanelItem::default();

        // Leak the filename string (FAR will own it until FreeFindDataW)
        let wide = crate::far::api::to_wide(entry.name());
        let ptr = Box::leak(wide.into_boxed_slice()).as_ptr();
        item.FileName = ptr;
        leaked_ptrs.push(ptr);

        if entry.is_dir() {
            item.FileAttributes = 0x10; // FILE_ATTRIBUTE_DIRECTORY
            item.FileSize = 0;
        } else {
            item.FileAttributes = 0x20; // FILE_ATTRIBUTE_ARCHIVE
            item.FileSize = entry.file_data().map(|d| d.len() as u64).unwrap_or(0);
        }

        items.push(item);
    }

    (items, leaked_ptrs)
}
