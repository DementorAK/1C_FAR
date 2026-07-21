use crate::v8::styles::PresentationStyle;
use crate::v8::vfs_builder::{BuildVfsError, VfsEntry};
use std::collections::HashMap;

pub struct RawStyle;

impl PresentationStyle for RawStyle {
    fn build_vfs(
        &self,
        rows_map: &HashMap<String, Vec<u8>>,
        packed_map: &HashMap<String, bool>,
    ) -> Result<Vec<VfsEntry>, BuildVfsError> {
        let mut vfs = Vec::new();
        for (id, data) in rows_map {
            let packed = packed_map.get(id).copied();
            vfs.push(VfsEntry::File {
                name: id.clone(),
                data: data.clone(),
                is_protected: false,
                origin_row_id: Some(id.clone()),
                original_container: None,
                origin_row_packed: packed,
            });
        }
        vfs.sort_by(|a, b| a.name().cmp(b.name()));
        Ok(vfs)
    }

    fn sync_vfs_to_rows(
        &self,
        vfs: &[VfsEntry],
        updates: &mut HashMap<String, Vec<u8>>,
        packed_out: &mut HashMap<String, bool>,
    ) {
        for entry in vfs {
            if let VfsEntry::File {
                data,
                origin_row_id: Some(row_id),
                origin_row_packed,
                ..
            } = entry
            {
                updates.insert(row_id.clone(), data.clone());
                packed_out.insert(row_id.clone(), origin_row_packed.unwrap_or(false));
            }
        }
    }
}
