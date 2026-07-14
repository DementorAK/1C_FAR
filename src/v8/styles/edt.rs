use crate::v8::styles::metadata_parser::{self, ContainerType};
use std::collections::HashMap;

use crate::v8::styles::PresentationStyle;
use crate::v8::vfs_builder::{BuildVfsError, VfsEntry};

pub struct EdtStyle;

// ---------------------------------------------------------------------------
// Build VFS from parsed metadata objects (EDT style)
// ---------------------------------------------------------------------------

/// Build VFS entries for subordinate groups (EDT style — same as Json).
fn build_subordinate_entries_edt(
    groups: &[metadata_parser::SubordinateGroupResolved],
) -> Vec<VfsEntry> {
    let mut entries = Vec::new();

    for group in groups {
        let mut children = Vec::new();

        for obj in &group.objects {
            if group.display_name == "Forms" {
                let module_data = obj
                    .module
                    .as_ref()
                    .map(|m| m.text.clone())
                    .unwrap_or_default();
                let is_prot = obj.module.as_ref().map(|m| m.is_protected).unwrap_or(false);
                let orig_cont = obj
                    .module
                    .as_ref()
                    .and_then(|m| m.original_container.clone());
                children.push(VfsEntry::Dir {
                    name: obj.name.clone(),
                    children: vec![VfsEntry::File {
                        name: "Module.bsl".to_string(),
                        data: module_data,
                        is_protected: is_prot,
                        origin_row_id: Some(obj.body_key.clone()),
                        original_container: orig_cont,
                    }],
                    origin_row_id: Some(obj.uuid.clone()),
                });
            } else {
                let data = obj.body_data.clone().unwrap_or_default();
                children.push(VfsEntry::File {
                    name: obj.name.clone(),
                    data,
                    is_protected: false,
                    origin_row_id: Some(obj.body_key.clone()),
                    original_container: None,
                });
            }
        }

        if !children.is_empty() {
            entries.push(VfsEntry::Dir {
                name: group.display_name.clone(),
                children,
                origin_row_id: None,
            });
        }
    }
    entries
}

/// Build VFS for a single EPF/ERF object (EDT style).
fn build_single_object_vfs_edt(metadata: &metadata_parser::ContainerMetadata) -> Vec<VfsEntry> {
    let mut vfs = Vec::new();

    if let Some(obj) = metadata.objects.first() {
        // ObjectModule.bsl
        if let Some(ref module) = obj.module {
            vfs.push(VfsEntry::File {
                name: "ObjectModule.bsl".to_string(),
                is_protected: module.is_protected,
                data: module.text.clone(),
                origin_row_id: module.origin_row_id.clone(),
                original_container: module.original_container.clone(),
            });
        }

        // Subordinate entries
        vfs.extend(build_subordinate_entries_edt(&obj.subordinate_groups));
    }

    vfs
}

/// Build VFS for configuration/extension (EDT style).
fn build_configuration_vfs_edt(metadata: &metadata_parser::ContainerMetadata) -> Vec<VfsEntry> {
    let mut vfs = Vec::new();

    for type_group in &metadata.type_groups {
        let mut obj_entries = Vec::new();

        for obj in &type_group.objects {
            let mut obj_children = Vec::new();

            // ObjectModule.bsl
            if let Some(ref module) = obj.module {
                obj_children.push(VfsEntry::File {
                    name: "ObjectModule.bsl".to_string(),
                    is_protected: module.is_protected,
                    data: module.text.clone(),
                    origin_row_id: module.origin_row_id.clone(),
                    original_container: module.original_container.clone(),
                });
            }

            // Subordinate entries
            obj_children.extend(build_subordinate_entries_edt(&obj.subordinate_groups));

            obj_entries.push(VfsEntry::Dir {
                name: obj.name.clone(),
                children: obj_children,
                origin_row_id: Some(obj.uuid.clone()),
            });
        }

        if !obj_entries.is_empty() {
            vfs.push(VfsEntry::Dir {
                name: type_group.display_name.clone(),
                children: obj_entries,
                origin_row_id: None,
            });
        }
    }

    vfs
}

impl PresentationStyle for EdtStyle {
    fn build_vfs(
        &self,
        rows_map: &HashMap<String, Vec<u8>>,
    ) -> Result<Vec<VfsEntry>, BuildVfsError> {
        // Step 1: Parse container into objects
        let metadata = metadata_parser::parse_container(rows_map)?;

        // Step 2: Build base VFS tree from objects
        let mut vfs = match metadata.container_type {
            ContainerType::SingleObject => build_single_object_vfs_edt(&metadata),
            ContainerType::Configuration => build_configuration_vfs_edt(&metadata),
            ContainerType::Extension => build_configuration_vfs_edt(&metadata),
        };

        // Sort extension groups
        if metadata.container_type == ContainerType::Extension {
            vfs.sort_by(|a, b| a.name().cmp(b.name()));
        }

        // Fallback if empty
        if vfs.is_empty() {
            for (id, data) in rows_map {
                if id != "root" && id != "version" && id != "versions" && id != "copyinfo" {
                    vfs.push(VfsEntry::File {
                        name: id.clone(),
                        data: data.clone(),
                        is_protected: false,
                        origin_row_id: Some(id.clone()),
                        original_container: None,
                    });
                }
            }
        }

        // Step 3: Serialize remaining rows to Configuration.mdo (EDT format)
        let mut used_ids = metadata.used_row_ids;
        fn collect_used(entries: &[VfsEntry], used: &mut std::collections::HashSet<String>) {
            for e in entries {
                match e {
                    VfsEntry::File {
                        origin_row_id: Some(id),
                        ..
                    } => {
                        used.insert(id.clone());
                    }
                    VfsEntry::Dir {
                        origin_row_id: Some(id),
                        children,
                        ..
                    } => {
                        used.insert(id.clone());
                        collect_used(children, used);
                    }
                    VfsEntry::Dir { children, .. } => {
                        collect_used(children, used);
                    }
                    _ => {}
                }
            }
        }
        collect_used(&vfs, &mut used_ids);

        // Build Configuration.mdo from remaining bracket strings
        let mut index_obj = serde_json::Map::new();
        for (id, data) in rows_map {
            if !used_ids.contains(id) {
                if let Ok(json_val) = crate::base::bracket_json::parse_bracket_to_json(data) {
                    index_obj.insert(id.clone(), json_val);
                } else {
                    vfs.push(VfsEntry::File {
                        name: format!("{}.raw", id),
                        data: data.clone(),
                        is_protected: false,
                        origin_row_id: Some(id.clone()),
                        original_container: None,
                    });
                }
            }
        }

        if !index_obj.is_empty() {
            let index_val = serde_json::Value::Object(index_obj);
            if let Ok(json_data) = serde_json::to_string(&index_val) {
                let xml_data = format!(
                    "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<MetaDataObject xmlns=\"http://v8.1c.ru/8.3/MDClasses\">\n    <RawData><![CDATA[{}]]></RawData>\n</MetaDataObject>",
                    json_data
                );
                vfs.push(VfsEntry::File {
                    name: "Configuration.mdo".to_string(),
                    data: xml_data.into_bytes(),
                    is_protected: false,
                    origin_row_id: None,
                    original_container: None,
                });
            }
        }

        Ok(vfs)
    }

    fn sync_vfs_to_rows(&self, vfs: &[VfsEntry], updates: &mut HashMap<String, Vec<u8>>) {
        sync_nodes_to_map_edt(vfs, updates);
    }
}

pub fn sync_nodes_to_map_edt(entries: &[VfsEntry], updates: &mut HashMap<String, Vec<u8>>) {
    for entry in entries {
        match entry {
            VfsEntry::File {
                name,
                data,
                origin_row_id,
                original_container,
                ..
            } => {
                if name == "Configuration.mdo" {
                    let xml_str = String::from_utf8_lossy(data);
                    if let Some(cdata_start) = xml_str.find("<![CDATA[") {
                        if let Some(cdata_end) = xml_str.find("]]>") {
                            let json_str = &xml_str[cdata_start + 9..cdata_end];
                            if let Ok(json_val) =
                                serde_json::from_str::<serde_json::Value>(json_str)
                            {
                                if let Some(obj) = json_val.as_object() {
                                    for (k, v) in obj {
                                        if let Ok(bracket_bytes) =
                                            crate::base::bracket_json::serialize_json_to_bracket(v)
                                        {
                                            updates.insert(k.clone(), bracket_bytes);
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else if let Some(row_id) = origin_row_id {
                    let mut final_data = data.clone();
                    if let Some(orig_cont) = original_container {
                        // Smart re-wrap: use original container as template
                        if let Ok(mut nested_rows) = crate::v8::container::read_container_rows(
                            crate::base::reader::StringReader::new(orig_cont.clone()),
                            0,
                        ) {
                            nested_rows.insert("text".to_string(), (data.clone(), false));
                            let mut writer = crate::v8::writer::ContainerWriter::new(512, false);
                            writer.use_triplets = true;
                            writer.pad_pt_to_page = false;
                            writer.revision = 6;
                            let mut buffer = Vec::new();
                            if writer
                                .write(&mut buffer, &nested_rows, None::<fn(usize, usize)>)
                                .is_ok()
                            {
                                final_data = buffer;
                            }
                        }
                    }
                    updates.insert(row_id.clone(), final_data);
                }
            }
            VfsEntry::Dir { children, .. } => {
                sync_nodes_to_map_edt(children, updates);
            }
        }
    }
}
