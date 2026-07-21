use crate::base::reader::StringReader;
use crate::v8::container::{is_container_data, Container};
use crate::v8::styles::PresentationStyle;
use crate::v8::vfs_builder::{BuildVfsError, VfsEntry};
use crate::v8::writer::ContainerWriter;
use std::collections::HashMap;

pub struct V8UnpackStyle;

impl V8UnpackStyle {
    fn build_vfs_recursive(
        rows_map: &HashMap<String, Vec<u8>>,
        packed_map: &HashMap<String, bool>,
    ) -> Result<Vec<VfsEntry>, BuildVfsError> {
        let mut vfs = Vec::new();

        for (id, data) in rows_map {
            if is_container_data(data) {
                let reader = StringReader::new(data.clone());
                if let Ok(mut container) = Container::new(reader, 0) {
                    let mut child_rows = HashMap::new();
                    let mut child_packed = HashMap::new();
                    for row in container.rows().flatten() {
                        child_rows.insert(row.id.clone(), row.data.clone());
                        child_packed.insert(row.id.clone(), row.is_packed);
                    }
                    let children = Self::build_vfs_recursive(&child_rows, &child_packed)?;
                    vfs.push(VfsEntry::Dir {
                        name: id.clone(),
                        children,
                        origin_row_id: Some(id.clone()),
                        origin_row_packed: packed_map.get(id).copied(),
                    });
                    continue;
                }
            }

            let packed = packed_map.get(id).copied();
            // End payload: create .header and .data files
            // The .header holds the 1C row preamble (20-byte prefix + id bytes);
            // restoring it on save is required for a faithful round-trip (see B2).
            let header_data = vec![0u8; 20]; // Mocking timestamps and attributes

            vfs.push(VfsEntry::File {
                name: format!("{}.header", id),
                data: header_data,
                is_protected: false,
                origin_row_id: Some(id.clone()),
                original_container: None,
                origin_row_packed: packed,
            });

            vfs.push(VfsEntry::File {
                name: format!("{}.data", id),
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
}

impl PresentationStyle for V8UnpackStyle {
    fn build_vfs(
        &self,
        rows_map: &HashMap<String, Vec<u8>>,
        packed_map: &HashMap<String, bool>,
    ) -> Result<Vec<VfsEntry>, BuildVfsError> {
        Self::build_vfs_recursive(rows_map, packed_map)
    }

    fn sync_vfs_to_rows(
        &self,
        vfs: &[VfsEntry],
        updates: &mut HashMap<String, Vec<u8>>,
        packed_out: &mut HashMap<String, bool>,
    ) {
        // V8Unpack emits two files per row: <id>.data and <id>.header.
        // Both carry the same origin_row_id and origin_row_packed; prefer .data
        // for the actual blob. .header is preserved only for B2 (still a mock here).
        let mut data_files: HashMap<String, (Vec<u8>, Option<bool>)> = HashMap::new();

        for entry in vfs {
            match entry {
                VfsEntry::File {
                    name,
                    data,
                    origin_row_id,
                    origin_row_packed,
                    ..
                } if name.ends_with(".data") => {
                    let id = origin_row_id
                        .clone()
                        .unwrap_or_else(|| name.trim_end_matches(".data").to_string());
                    data_files.insert(id, (data.clone(), *origin_row_packed));
                }
                VfsEntry::Dir {
                    children,
                    origin_row_id: Some(row_id),
                    origin_row_packed,
                    ..
                } => {
                    let mut nested_updates = HashMap::new();
                    let mut nested_packed = HashMap::new();
                    self.sync_vfs_to_rows(children, &mut nested_updates, &mut nested_packed);

                    if !nested_updates.is_empty() {
                        let mut writer = ContainerWriter::new(512, false);
                        writer.revision = 6;
                        let mut nested_rows = HashMap::new();
                        for (cid, cdata) in &nested_updates {
                            let packed = nested_packed.get(cid).copied().unwrap_or(false);
                            nested_rows.insert(cid.clone(), (cdata.clone(), packed));
                        }
                        let mut buffer = Vec::new();
                        if writer
                            .write(&mut buffer, &nested_rows, None::<fn(usize, usize)>)
                            .is_ok()
                        {
                            updates.insert(row_id.clone(), buffer);
                            packed_out.insert(row_id.clone(), origin_row_packed.unwrap_or(false));
                        }
                    }
                }
                _ => {}
            }
        }

        for (id, (data, packed)) in data_files {
            updates.insert(id.clone(), data);
            packed_out.insert(id, packed.unwrap_or(false));
        }
    }
}
