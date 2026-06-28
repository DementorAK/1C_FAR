use crate::v8::styles::PresentationStyle;
use crate::v8::vfs_builder::{BuildVfsError, VfsEntry};
use std::collections::HashMap;

pub struct RawStyle;

impl PresentationStyle for RawStyle {
    fn build_vfs(
        &self,
        rows_map: &HashMap<String, Vec<u8>>,
    ) -> Result<Vec<VfsEntry>, BuildVfsError> {
        let mut vfs = Vec::new();
        for (id, data) in rows_map {
            vfs.push(VfsEntry::File {
                name: id.clone(),
                data: data.clone(),
                is_protected: false,
                origin_row_id: Some(id.clone()),
                original_container: None,
            });
        }
        vfs.sort_by(|a, b| a.name().cmp(b.name()));
        Ok(vfs)
    }

    fn sync_vfs_to_rows(&self, vfs: &[VfsEntry], updates: &mut HashMap<String, Vec<u8>>) {
        for entry in vfs {
            if let VfsEntry::File {
                data,
                origin_row_id: Some(row_id),
                ..
            } = entry
            {
                updates.insert(row_id.clone(), data.clone());
            }
        }
    }
}
