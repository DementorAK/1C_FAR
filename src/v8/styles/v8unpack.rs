use crate::base::reader::StringReader;
use crate::v8::container::Container;
use crate::v8::styles::PresentationStyle;
use crate::v8::vfs_builder::{BuildVfsError, VfsEntry};
use crate::v8::writer::ContainerWriter;
use std::collections::HashMap;

pub struct V8UnpackStyle;

fn is_container(data: &[u8]) -> bool {
    if data.len() >= 4 {
        if let Ok(bytes) = data[0..4].try_into() {
            let sig = u32::from_le_bytes(bytes);
            if sig == crate::v8::container::SIG || (sig as u64) == crate::v8::container::SIG64 {
                return true;
            }
        }
    }
    false
}

impl V8UnpackStyle {
    fn build_vfs_recursive(
        rows_map: &HashMap<String, Vec<u8>>,
    ) -> Result<Vec<VfsEntry>, BuildVfsError> {
        let mut vfs = Vec::new();

        for (id, data) in rows_map {
            if is_container(data) {
                let reader = StringReader::new(data.clone());
                if let Ok(mut container) = Container::new(reader, 0) {
                    let mut child_rows = HashMap::new();
                    for row in container.rows().flatten() {
                        child_rows.insert(row.id.clone(), row.data.clone());
                    }
                    let children = Self::build_vfs_recursive(&child_rows)?;
                    vfs.push(VfsEntry::Dir {
                        name: id.clone(),
                        children,
                        origin_row_id: Some(id.clone()),
                    });
                    continue;
                }
            }

            // End payload: create .header and .data files
            let header_data = vec![0u8; 20]; // Mocking timestamps and attributes

            vfs.push(VfsEntry::File {
                name: format!("{}.header", id),
                data: header_data,
                is_protected: false,
                origin_row_id: None,
                original_container: None,
            });

            vfs.push(VfsEntry::File {
                name: format!("{}.data", id),
                data: data.clone(),
                is_protected: false,
                origin_row_id: None,
                original_container: None,
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
    ) -> Result<Vec<VfsEntry>, BuildVfsError> {
        Self::build_vfs_recursive(rows_map)
    }

    fn sync_vfs_to_rows(&self, vfs: &[VfsEntry], updates: &mut HashMap<String, Vec<u8>>) {
        let mut data_files = HashMap::new();

        for entry in vfs {
            match entry {
                VfsEntry::File { name, data, .. } => {
                    if let Some(id) = name.strip_suffix(".data") {
                        data_files.insert(id.to_string(), data.clone());
                    }
                }
                VfsEntry::Dir {
                    children,
                    origin_row_id: Some(row_id),
                    ..
                } => {
                    let mut nested_updates = HashMap::new();
                    self.sync_vfs_to_rows(children, &mut nested_updates);

                    if !nested_updates.is_empty() {
                        let mut writer = ContainerWriter::new(512, false);
                        writer.revision = 6;
                        let mut nested_rows = HashMap::new();
                        for (cid, cdata) in nested_updates {
                            nested_rows.insert(cid, (cdata, false));
                        }
                        let mut buffer = Vec::new();
                        if writer
                            .write(&mut buffer, &nested_rows, None::<fn(usize, usize)>)
                            .is_ok()
                        {
                            updates.insert(row_id.clone(), buffer);
                        }
                    }
                }
                _ => {}
            }
        }

        for (id, data) in data_files {
            updates.insert(id, data);
        }
    }
}
