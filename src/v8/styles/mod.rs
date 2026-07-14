use crate::v8::vfs_builder::{BuildVfsError, VfsEntry};
use std::collections::HashMap;

pub mod configurator;
pub mod edt;
pub mod full_parse;
pub mod json;
pub mod metadata_parser;
pub mod raw;
pub mod v8unpack;

pub trait PresentationStyle: Send + Sync {
    /// Builds the VFS tree from the raw container rows
    fn build_vfs(
        &self,
        rows_map: &HashMap<String, Vec<u8>>,
    ) -> Result<Vec<VfsEntry>, BuildVfsError>;

    /// Synchronizes the modified VFS tree back to the raw rows map
    fn sync_vfs_to_rows(&self, vfs: &[VfsEntry], updates: &mut HashMap<String, Vec<u8>>);
}
