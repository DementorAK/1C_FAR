use crate::v8::styles::metadata_parser::{self, ContainerType};
use std::collections::HashMap;

use crate::v8::styles::PresentationStyle;
use crate::v8::vfs_builder::{BuildVfsError, VfsEntry};

pub struct JsonStyle;

// ---------------------------------------------------------------------------
// Build VFS from parsed metadata objects
// ---------------------------------------------------------------------------

/// Build VFS entries for subordinate groups of a single object (Json style).
fn build_subordinate_entries_json(
    groups: &[metadata_parser::SubordinateGroupResolved],
    packed_map: &HashMap<String, bool>,
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
                        origin_row_packed: metadata_parser::lookup_packed(
                            packed_map,
                            Some(&obj.body_key),
                        ),
                    }],
                    origin_row_id: Some(obj.uuid.clone()),
                    origin_row_packed: None,
                });
            } else {
                let data = obj.body_data.clone().unwrap_or_default();
                children.push(VfsEntry::File {
                    name: obj.name.clone(),
                    data,
                    is_protected: false,
                    origin_row_id: Some(obj.body_key.clone()),
                    original_container: None,
                    origin_row_packed: metadata_parser::lookup_packed(
                        packed_map,
                        Some(&obj.body_key),
                    ),
                });
            }
        }

        if !children.is_empty() {
            entries.push(VfsEntry::Dir {
                name: group.display_name.clone(),
                children,
                origin_row_id: None,
                origin_row_packed: None,
            });
        }
    }
    entries
}

/// Build VFS for a single EPF/ERF object (Json style).
fn build_single_object_vfs_json(
    metadata: &metadata_parser::ContainerMetadata,
    packed_map: &HashMap<String, bool>,
) -> Vec<VfsEntry> {
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
                origin_row_packed: metadata_parser::lookup_packed(
                    packed_map,
                    module.origin_row_id.as_deref(),
                ),
            });
        }

        // Subordinate entries
        vfs.extend(build_subordinate_entries_json(
            &obj.subordinate_groups,
            packed_map,
        ));
    }

    vfs
}

/// Build VFS for configuration (CF) — Json style.
fn build_configuration_vfs_json(
    metadata: &metadata_parser::ContainerMetadata,
    packed_map: &HashMap<String, bool>,
) -> Vec<VfsEntry> {
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
                    origin_row_packed: metadata_parser::lookup_packed(
                        packed_map,
                        module.origin_row_id.as_deref(),
                    ),
                });
            }

            // Subordinate entries
            obj_children.extend(build_subordinate_entries_json(
                &obj.subordinate_groups,
                packed_map,
            ));

            obj_entries.push(VfsEntry::Dir {
                name: obj.name.clone(),
                children: obj_children,
                origin_row_id: Some(obj.uuid.clone()),
                origin_row_packed: None,
            });
        }

        if !obj_entries.is_empty() {
            vfs.push(VfsEntry::Dir {
                name: type_group.display_name.clone(),
                children: obj_entries,
                origin_row_id: None,
                origin_row_packed: None,
            });
        }
    }

    vfs
}

impl PresentationStyle for JsonStyle {
    fn build_vfs(
        &self,
        rows_map: &HashMap<String, Vec<u8>>,
        packed_map: &HashMap<String, bool>,
    ) -> Result<Vec<VfsEntry>, BuildVfsError> {
        // Step 1: Parse container into objects
        let metadata = metadata_parser::parse_container(rows_map)?;

        // Step 2: Build base VFS tree from objects, propagating per-row packed flag.
        let mut vfs = match metadata.container_type {
            ContainerType::SingleObject => build_single_object_vfs_json(&metadata, packed_map),
            ContainerType::Configuration => build_configuration_vfs_json(&metadata, packed_map),
            ContainerType::Extension => build_configuration_vfs_json(&metadata, packed_map),
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
                        is_protected: metadata_parser::is_protected_module(data),
                        origin_row_id: Some(id.clone()),
                        original_container: None,
                        origin_row_packed: packed_map.get(id).copied(),
                    });
                }
            }
        }

        // Step 3: Serialize remaining rows to index.json
        // Track used IDs from the VFS
        let used_ids = crate::v8::styles::rewrap::collect_used_row_ids(&vfs, metadata.used_row_ids);

        // Build index.json from remaining strings (serialize bracket→JSON)
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
                        origin_row_packed: packed_map.get(id).copied(),
                    });
                }
            }
        }

        if !index_obj.is_empty() {
            let index_val = serde_json::Value::Object(index_obj);
            if let Ok(index_data) = serde_json::to_vec_pretty(&index_val) {
                vfs.push(VfsEntry::File {
                    name: "index.json".to_string(),
                    data: index_data,
                    is_protected: false,
                    origin_row_id: None,
                    original_container: None,
                    origin_row_packed: None,
                });
            }
        }

        Ok(vfs)
    }

    fn sync_vfs_to_rows(
        &self,
        vfs: &[VfsEntry],
        updates: &mut HashMap<String, Vec<u8>>,
        packed_out: &mut HashMap<String, bool>,
    ) {
        sync_nodes_to_map_json(vfs, updates, packed_out);
    }
}

pub fn sync_nodes_to_map_json(
    entries: &[VfsEntry],
    updates: &mut HashMap<String, Vec<u8>>,
    packed_out: &mut HashMap<String, bool>,
) {
    for entry in entries {
        match entry {
            VfsEntry::File {
                name,
                data,
                origin_row_id,
                original_container,
                origin_row_packed,
                ..
            } => {
                if name == "index.json" {
                    if let Ok(json_val) = serde_json::from_slice::<serde_json::Value>(data) {
                        if let Some(obj) = json_val.as_object() {
                            for (k, v) in obj {
                                if let Ok(bracket_bytes) =
                                    crate::base::bracket_json::serialize_json_to_bracket(v)
                                {
                                    // JSON-recovered bracket strings are never deflate-compressed.
                                    updates.insert(k.clone(), bracket_bytes);
                                    packed_out.insert(k.clone(), false);
                                }
                            }
                        }
                    }
                } else if let Some(row_id) = origin_row_id {
                    // Preserve the original row packed flag across the re-wrap; the
                    // top-level writer will deflate the resulting container blob if
                    // and only if the original row was deflate-compressed.
                    let final_packed = origin_row_packed.unwrap_or(false);
                    let final_data = if let Some(orig_cont) = original_container {
                        match crate::v8::styles::rewrap::smart_rewrap_module(orig_cont, data) {
                            Some(buffer) => buffer,
                            None => data.clone(),
                        }
                    } else {
                        data.clone()
                    };
                    updates.insert(row_id.clone(), final_data);
                    packed_out.insert(row_id.clone(), final_packed);
                }
            }
            VfsEntry::Dir { children, .. } => {
                sync_nodes_to_map_json(children, updates, packed_out);
            }
        }
    }
}
