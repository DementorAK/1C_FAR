use crate::v8_artifacts::vfs_builder::VfsEntry;
use crate::far::api::*;
use std::collections::HashMap;
use std::io;
use crate::v8_artifacts::writer::ContainerWriter;
use crate::far::settings::PluginSettings;
use chrono::Local;

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
    /// Has the content been modified?
    pub is_modified: bool,
    /// Original rows of the artifact (key is row ID).
    pub rows_map: HashMap<String, Vec<u8>>,
    /// Map of row ID to whether it was originally packed (compressed).
    pub packed_map: HashMap<String, bool>,
    /// Page size for re-packing.
    pub page_size: u32,
    /// Is the artifact using 64-bit format?
    pub is_64bit: bool,
    /// Plugin settings.
    pub settings: PluginSettings,
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
            is_modified: false,
            rows_map: HashMap::new(),
            packed_map: HashMap::new(),
            page_size: 512,
            is_64bit: false,
            settings: PluginSettings::load(),
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

    /// Find an entry in the current virtual directory by name.
    pub fn find_entry_in_current_dir(&self, name: &str) -> Option<&VfsEntry> {
        self.resolve_current_dir().iter().find(|e| e.name() == name)
    }

    /// Get the mutable entries in the current virtual directory.
    pub fn resolve_current_dir_mut(&mut self) -> &mut [VfsEntry] {
        let mut current: &mut [VfsEntry] = &mut self.vfs;

        for dir_name in self.current_dir.iter() {
            if let Some(pos) = current.iter().position(|e| e.name() == dir_name) {
                match &mut current[pos] {
                    VfsEntry::Dir { children, .. } => {
                        current = children;
                    }
                    _ => return &mut [],
                }
            } else {
                return &mut [];
            }
        }

        current
    }

    /// Find a mutable entry in the current virtual directory by name.
    pub fn find_entry_in_current_dir_mut(&mut self, name: &str) -> Option<&mut VfsEntry> {
        self.resolve_current_dir_mut().iter_mut().find(|e| e.name() == name)
    }

    /// Commit memory changes to disk artifact.
    pub fn commit_changes(&mut self) -> io::Result<()> {
        if !self.is_modified {
            return Ok(());
        }

        // 1. Create backup if enabled
        if self.settings.create_backup {
            let path = std::path::Path::new(&self.path);
            let stem = path.file_stem().map(|s| s.to_string_lossy()).unwrap_or_default();
            let extension = path.extension().map(|e| e.to_string_lossy()).unwrap_or_default();
            let timestamp = Local::now().format("%Y%m%d-%H%M%S").to_string();
            
            let bak_name = if extension.is_empty() {
                format!("{}.{}", stem, timestamp)
            } else {
                format!("{}.{}.{}", stem, timestamp, extension)
            };
            
            let bak_path = path.with_file_name(bak_name);
            std::fs::copy(&self.path, &bak_path)?;
        }

        // 2. Synchronize VFS modifications back to rows_map
        self.sync_vfs_to_rows();

        // 3. Serialize and write
        let mut writer_logic = ContainerWriter::new(self.page_size, self.is_64bit);
        // Root EPF files MUST use triplets and padded pointer tables
        writer_logic.use_triplets = true;
        writer_logic.pad_pt_to_page = true;
        writer_logic.revision = 6;
        let mut buffer = Vec::new();
        
        // Combine rows and packed state
        let mut full_rows = HashMap::new();
        for (id, data) in &self.rows_map {
            let is_packed = self.packed_map.get(id).cloned().unwrap_or(true);
            full_rows.insert(id.clone(), (data.clone(), is_packed));
        }
        
        writer_logic.write(&mut buffer, &full_rows)?;

        // Write to temp file then rename (atomic swap)
        let tmp_path = format!("{}.tmp", self.path);
        std::fs::write(&tmp_path, buffer)?;
        
        // On Windows, rename fails if destination exists, so we might need to remove it or use winapi.
        // But std::fs::rename usually handles it if it's the same volume.
        // If it fails, we try to remove first.
        if let Err(_) = std::fs::rename(&tmp_path, &self.path) {
            std::fs::remove_file(&self.path)?;
            std::fs::rename(&tmp_path, &self.path)?;
        }

        self.is_modified = false;
        Ok(())
    }

    fn sync_vfs_to_rows(&mut self) {
        sync_nodes_to_map(&self.vfs, &mut self.rows_map);
    }
}

fn sync_nodes_to_map(entries: &[VfsEntry], updates: &mut HashMap<String, Vec<u8>>) {
    for entry in entries {
        match entry {
            VfsEntry::File { data, origin_row_id, original_container, .. } => {
                if let Some(row_id) = origin_row_id {
                    let mut final_data = data.clone();
                    if let Some(orig_cont) = original_container {
                        // Smart re-wrap: use original container as template
                        if let Ok(mut nested_rows) = crate::v8_artifacts::container::read_container_rows(
                            crate::base::reader::StringReader::new(orig_cont.clone()), 0
                        ) {
                            // Update only the text row, preserve everything else (info, etc.)
                            nested_rows.insert("text".to_string(), (data.clone(), false));
                            
                            // Nested containers match the original 1C format (triplets, stored text)
                            let mut writer = ContainerWriter::new(512, false);
                            writer.use_triplets = true;
                            writer.pad_pt_to_page = false;
                            writer.revision = 6;
                            let mut buffer = Vec::new();
                            if let Ok(_) = writer.write(&mut buffer, &nested_rows) {
                                final_data = buffer;
                            }
                        }
                    }
                    updates.insert(row_id.clone(), final_data);
                }
            }
            VfsEntry::Dir { children, .. } => {
                sync_nodes_to_map(children, updates);
            }
        }
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
