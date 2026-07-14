use crate::v8::styles::metadata_parser::{self, ContainerType};
use std::collections::HashMap;

use crate::v8::styles::PresentationStyle;
use crate::v8::vfs_builder::{BuildVfsError, VfsEntry};

pub mod schema;

pub struct ConfiguratorStyle;

// ---------------------------------------------------------------------------
// Build VFS from parsed metadata objects (Configurator style)
// ---------------------------------------------------------------------------

/// Build VFS entries for subordinate groups (Configurator style — with Ext/ and XML).
fn build_subordinate_entries_configurator(
    groups: &[metadata_parser::SubordinateGroupResolved],
    parent_name: &str,
    parent_type: &str,
    uuid_map: &HashMap<String, String>,
) -> Vec<VfsEntry> {
    let mut entries = Vec::new();

    for group in groups {
        let mut children = Vec::new();

        for obj in &group.objects {
            // Generate metadata xml file from header
            if let Some(ref h) = obj.header_data {
                if let Some(xml) = schema::bracket_to_xml(
                    &String::from_utf8_lossy(h),
                    &group.display_name,
                    Some(&obj.name),
                ) {
                    children.push(VfsEntry::File {
                        name: format!("{}.xml", obj.name),
                        data: xml.into_bytes(),
                        is_protected: false,
                        origin_row_id: Some(obj.uuid.clone()),
                        original_container: None,
                    });
                }
            }

            if group.display_name == "Forms" {
                let mut ext_children = Vec::new();
                if let Some(ref body) = obj.body_data {
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

                    let form_xml_and_module = schema::bracket_to_formlayout_xml(
                        &String::from_utf8_lossy(body),
                        parent_name,
                        parent_type,
                        Some(uuid_map),
                    );
                    if let Some((xml, form_module_content)) = form_xml_and_module {
                        ext_children.push(VfsEntry::File {
                            name: "Form.xml".to_string(),
                            data: xml.into_bytes(),
                            is_protected: false,
                            origin_row_id: Some(obj.body_key.clone()),
                            original_container: orig_cont.clone(),
                        });

                        if let Some(module_str) = form_module_content {
                            if metadata_parser::has_real_content(module_str.as_bytes()) {
                                let module_ext_children = vec![VfsEntry::File {
                                    name: "Module.bsl".to_string(),
                                    data: module_str.into_bytes(),
                                    is_protected: false,
                                    origin_row_id: Some(obj.body_key.clone()),
                                    original_container: orig_cont.clone(),
                                }];
                                ext_children.push(VfsEntry::Dir {
                                    name: "Form".to_string(),
                                    children: module_ext_children,
                                    origin_row_id: None,
                                });
                            }
                        }
                    }

                    if metadata_parser::has_real_content(&module_data) && orig_cont.is_some() {
                        ext_children.push(VfsEntry::File {
                            name: "Module.bsl".to_string(),
                            data: module_data,
                            is_protected: is_prot,
                            origin_row_id: Some(obj.body_key.clone()),
                            original_container: orig_cont,
                        });
                    }
                }
                if !ext_children.is_empty() {
                    children.push(VfsEntry::Dir {
                        name: obj.name.clone(),
                        children: vec![VfsEntry::Dir {
                            name: "Ext".to_string(),
                            children: ext_children,
                            origin_row_id: None,
                        }],
                        origin_row_id: None,
                    });
                }
            } else if group.display_name == "Templates" {
                if let Some(ref b) = obj.body_data {
                    let mut ext_children = Vec::new();
                    if let Some(xml) =
                        schema::bracket_to_xml(&String::from_utf8_lossy(b), "TemplateLayout", None)
                    {
                        ext_children.push(VfsEntry::File {
                            name: "Template.xml".to_string(),
                            data: xml.into_bytes(),
                            is_protected: false,
                            origin_row_id: Some(obj.body_key.clone()),
                            original_container: None,
                        });
                    } else {
                        ext_children.push(VfsEntry::File {
                            name: "Template.bin".to_string(),
                            data: b.clone(),
                            is_protected: false,
                            origin_row_id: Some(obj.body_key.clone()),
                            original_container: None,
                        });
                    }
                    children.push(VfsEntry::Dir {
                        name: obj.name.clone(),
                        children: vec![VfsEntry::Dir {
                            name: "Ext".to_string(),
                            children: ext_children,
                            origin_row_id: None,
                        }],
                        origin_row_id: None,
                    });
                }
            } else {
                let data = obj.body_data.clone().unwrap_or_default();
                if !data.is_empty() {
                    children.push(VfsEntry::Dir {
                        name: obj.name.clone(),
                        children: vec![VfsEntry::Dir {
                            name: "Ext".to_string(),
                            children: vec![VfsEntry::File {
                                name: "Module.bsl".to_string(),
                                data,
                                is_protected: false,
                                origin_row_id: Some(obj.body_key.clone()),
                                original_container: None,
                            }],
                            origin_row_id: None,
                        }],
                        origin_row_id: None,
                    });
                }
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

/// Build VFS for a single EPF/ERF object (Configurator style).
fn build_single_object_vfs_configurator(
    rows_map: &HashMap<String, Vec<u8>>,
    metadata: &metadata_parser::ContainerMetadata,
) -> Vec<VfsEntry> {
    let mut vfs = Vec::new();

    if let Some(obj) = metadata.objects.first() {
        let mut obj_children = Vec::new();

        // Build child refs for ChildObjects XML section
        let mut child_refs = Vec::new();
        for group in &obj.subordinate_groups {
            if let Some(elem_type) = schema::group_to_element_type(&group.display_name) {
                for sub_obj in &group.objects {
                    child_refs.push(schema::ChildObjectRef {
                        element_type: elem_type.to_string(),
                        name: sub_obj.name.clone(),
                        raw_xml: None,
                    });
                }
            }
        }

        let mut epf_uuid_map = HashMap::new();

        // Create root XML file
        if let Some(root_data) = rows_map.get(&obj.uuid) {
            // Determine class name from bracket header
            let class_name = {
                if let Ok(json_val2) = crate::base::bracket_json::parse_bracket_to_json(root_data) {
                    if let Some(root) = json_val2.as_array() {
                        if let Some(inner) = root.get(3).and_then(|v| v.as_array()) {
                            if let Some(cid) = inner.first().and_then(|s| s.as_str()) {
                                if cid == "e41aff26-25cf-4bb6-b6c1-3f478a75f374" {
                                    "ExternalReport"
                                } else {
                                    "ExternalDataProcessor"
                                }
                            } else {
                                "ExternalDataProcessor"
                            }
                        } else {
                            "ExternalDataProcessor"
                        }
                    } else {
                        "ExternalDataProcessor"
                    }
                } else {
                    "ExternalDataProcessor"
                }
            };

            // Extract inline Attribute and TabularSection definitions
            if let Ok(json_val) = crate::base::bracket_json::parse_bracket_to_json(root_data) {
                let inline_children =
                    schema::extract_epf_child_objects(&json_val, class_name, &obj.name);
                child_refs.extend(inline_children);
                epf_uuid_map = schema::extract_epf_uuid_name_map(&json_val);
            }

            // Find the form name from child_refs
            let form_name = child_refs
                .iter()
                .find(|c| c.element_type == "Form")
                .map(|c| c.name.clone());

            let default_form =
                form_name.map(|fn_name| format!("{}.{}.Form.{}", class_name, obj.name, fn_name));

            if let Some(xml_data) = schema::bracket_to_xml_with_children(
                &String::from_utf8_lossy(root_data),
                class_name,
                &child_refs,
                default_form.as_deref(),
            ) {
                vfs.push(VfsEntry::File {
                    name: format!("{}.xml", obj.name),
                    data: xml_data.into_bytes(),
                    is_protected: false,
                    origin_row_id: Some(obj.uuid.clone()),
                    original_container: None,
                });
            }
        }

        // ObjectModule.bsl in Ext/
        if let Some(ref module) = obj.module {
            obj_children.push(VfsEntry::Dir {
                name: "Ext".to_string(),
                children: vec![VfsEntry::File {
                    name: if module.is_protected {
                        "ObjectModule.bin".to_string()
                    } else {
                        "ObjectModule.bsl".to_string()
                    },
                    is_protected: module.is_protected,
                    data: module.text.clone(),
                    origin_row_id: module.origin_row_id.clone(),
                    original_container: module.original_container.clone(),
                }],
                origin_row_id: None,
            });
        }

        let parent_type = &obj.class_name;
        obj_children.extend(build_subordinate_entries_configurator(
            &obj.subordinate_groups,
            &obj.name,
            parent_type,
            &epf_uuid_map,
        ));

        // Put everything inside <obj_name> folder except root XML
        if !obj_children.is_empty() {
            vfs.push(VfsEntry::Dir {
                name: obj.name.clone(),
                children: obj_children,
                origin_row_id: None,
            });
        }

        // Fallback: if VFS empty, show raw non-utility rows
        if vfs.is_empty() {
            for (id, data) in rows_map {
                if id != "root" && id != "version" && id != "versions" && id != "copyinfo" {
                    vfs.push(VfsEntry::File {
                        name: id.clone(),
                        data: data.clone(),
                        is_protected: metadata_parser::is_protected_module(data),
                        origin_row_id: Some(id.clone()),
                        original_container: None,
                    });
                }
            }
        }
    }

    vfs
}

/// Build VFS for configuration (CF) — Configurator style.
fn build_configuration_vfs_configurator(
    rows_map: &HashMap<String, Vec<u8>>,
    metadata: &metadata_parser::ContainerMetadata,
) -> Vec<VfsEntry> {
    let mut vfs = Vec::new();

    // Root Configuration.xml
    if let Some(root_data) = rows_map.get(&metadata.root_uuid) {
        if let Some(xml) =
            schema::bracket_to_xml(&String::from_utf8_lossy(root_data), "Configuration", None)
        {
            vfs.push(VfsEntry::File {
                name: "Configuration.xml".to_string(),
                data: xml.into_bytes(),
                is_protected: false,
                origin_row_id: Some(metadata.root_uuid.clone()),
                original_container: None,
            });
        }
    }

    for type_group in &metadata.type_groups {
        let mut obj_entries = Vec::new();

        for obj in &type_group.objects {
            let parent_type = schema::group_to_element_type(&type_group.display_name).unwrap_or("");
            let mut obj_children = Vec::new();

            // Object XML file
            if let Some(ref header) = obj.header_data {
                if let Some(xml) = schema::bracket_to_xml(
                    &String::from_utf8_lossy(header),
                    &type_group.display_name,
                    Some(&obj.name),
                ) {
                    obj_children.push(VfsEntry::File {
                        name: format!("{}.xml", obj.name),
                        data: xml.into_bytes(),
                        is_protected: false,
                        origin_row_id: Some(obj.uuid.clone()),
                        original_container: None,
                    });
                }
            }

            // ObjectModule.bsl in Ext/
            if let Some(ref module) = obj.module {
                obj_children.push(VfsEntry::Dir {
                    name: "Ext".to_string(),
                    children: vec![VfsEntry::File {
                        name: "ObjectModule.bsl".to_string(),
                        is_protected: module.is_protected,
                        data: module.text.clone(),
                        origin_row_id: module.origin_row_id.clone(),
                        original_container: module.original_container.clone(),
                    }],
                    origin_row_id: None,
                });
            }

            // Subordinate entries
            obj_children.extend(build_subordinate_entries_configurator(
                &obj.subordinate_groups,
                &obj.name,
                parent_type,
                &HashMap::new(),
            ));

            // Fallback if children empty
            if obj_children.is_empty() {
                if let Some(ref header) = obj.header_data {
                    obj_children.push(VfsEntry::File {
                        name: obj.uuid.clone(),
                        data: header.clone(),
                        is_protected: metadata_parser::is_protected_module(header),
                        origin_row_id: Some(obj.uuid.clone()),
                        original_container: None,
                    });
                }
            }

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

    // Fallback if nothing was found
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

    vfs
}

impl PresentationStyle for ConfiguratorStyle {
    fn build_vfs(
        &self,
        rows_map: &HashMap<String, Vec<u8>>,
    ) -> Result<Vec<VfsEntry>, BuildVfsError> {
        // Step 1: Parse container into objects
        let metadata = metadata_parser::parse_container(rows_map)?;

        // Step 2: Build VFS from objects
        let mut vfs = match metadata.container_type {
            ContainerType::SingleObject => {
                build_single_object_vfs_configurator(rows_map, &metadata)
            }
            ContainerType::Configuration => {
                build_configuration_vfs_configurator(rows_map, &metadata)
            }
            ContainerType::Extension => {
                let mut ext_vfs = build_configuration_vfs_configurator(rows_map, &metadata);
                // Add Configuration.xml for extension root if not already present
                if !ext_vfs.iter().any(|e| e.name() == "Configuration.xml") {
                    if let Some(root_data) = rows_map.get(&metadata.root_uuid) {
                        if let Some(xml) = schema::bracket_to_xml(
                            &String::from_utf8_lossy(root_data),
                            "Configuration",
                            None,
                        ) {
                            ext_vfs.push(VfsEntry::File {
                                name: "Configuration.xml".to_string(),
                                data: xml.into_bytes(),
                                is_protected: false,
                                origin_row_id: Some(metadata.root_uuid.clone()),
                                original_container: None,
                            });
                        }
                    }
                }
                ext_vfs.sort_by(|a, b| a.name().cmp(b.name()));
                ext_vfs
            }
        };

        // Fallback if empty
        if vfs.is_empty() {
            for (id, data) in rows_map {
                let utility = ["version", "versions", "copyinfo", "configinfo", "root"];
                if !utility.contains(&id.as_str()) {
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

        Ok(vfs)
    }

    fn sync_vfs_to_rows(&self, vfs: &[VfsEntry], updates: &mut HashMap<String, Vec<u8>>) {
        sync_nodes_to_map_configurator(vfs, updates);
    }
}

pub fn sync_nodes_to_map_configurator(
    entries: &[VfsEntry],
    updates: &mut HashMap<String, Vec<u8>>,
) {
    for entry in entries {
        match entry {
            VfsEntry::File {
                name,
                data,
                origin_row_id,
                original_container,
                ..
            } => {
                if name.ends_with(".xml") {
                    let xml_str = String::from_utf8_lossy(data);
                    if let Some(row_id) = origin_row_id.as_ref() {
                        if let Some(bracket_bytes) = schema::xml_to_bracket(&xml_str) {
                            updates.insert(row_id.clone(), bracket_bytes);
                        }
                    }
                }

                if !name.ends_with(".xml") && origin_row_id.is_some() {
                    let row_id = origin_row_id.as_ref().unwrap();
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
                sync_nodes_to_map_configurator(children, updates);
            }
        }
    }
}
