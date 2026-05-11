use std::collections::HashMap;
use crate::v8_artifacts::container::{Container, SIG};
use crate::v8_artifacts::uuids;
use crate::base::reader::StringReader;
use crate::base::parser::{StructParser, strip_quotes};

const UTF8_BOM: &[u8] = &[0xEF, 0xBB, 0xBF];

/// A single entry in the virtual filesystem.
#[derive(Debug)]
pub enum VfsEntry {
    File {
        name: String,
        data: Vec<u8>,
        is_protected: bool,
    },
    Dir {
        name: String,
        children: Vec<VfsEntry>,
    },
}

impl VfsEntry {
    pub fn name(&self) -> &str {
        match self {
            VfsEntry::File { name, .. } => name,
            VfsEntry::Dir { name, .. } => name,
        }
    }

    pub fn is_dir(&self) -> bool {
        matches!(self, VfsEntry::Dir { .. })
    }

    pub fn children(&self) -> Option<&[VfsEntry]> {
        match self {
            VfsEntry::Dir { children, .. } => Some(children),
            _ => None,
        }
    }

    pub fn find_child(&self, name: &str) -> Option<&VfsEntry> {
        self.children().and_then(|children| {
            children.iter().find(|c| c.name() == name)
        })
    }

    pub fn file_data(&self) -> Option<&[u8]> {
        match self {
            VfsEntry::File { data, .. } => Some(data),
            _ => None,
        }
    }

    /// Recursively extract the entry to the given physical path.
    pub fn extract_to(&self, dest_path: &std::path::Path) -> std::io::Result<()> {
        match self {
            VfsEntry::File { data, .. } => {
                std::fs::write(dest_path, data)?;
            }
            VfsEntry::Dir { children, .. } => {
                if !dest_path.exists() {
                    std::fs::create_dir_all(dest_path)?;
                }
                for child in children {
                    let child_path = dest_path.join(child.name());
                    child.extract_to(&child_path)?;
                }
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Utility helpers
// ---------------------------------------------------------------------------

fn strip_bom(data: &[u8]) -> &[u8] {
    if data.starts_with(UTF8_BOM) { &data[3..] } else { data }
}

fn data_to_string(data: &[u8]) -> Option<String> {
    let clean = strip_bom(data);
    std::str::from_utf8(clean).ok().map(|s| s.to_string())
}

/// Extract the text module from a row's .0 data (may be nested container).
fn extract_module_text(data: &[u8]) -> Option<Vec<u8>> {
    if data.is_empty() { return None; }
    if data.len() >= 4 {
        if let Ok(bytes) = data[0..4].try_into() {
            let sig = u32::from_le_bytes(bytes);
            if sig == SIG {
                let reader = StringReader::new(data.to_vec());
                if let Ok(mut container) = Container::new(reader, 0) {
                    for row_res in container.rows() {
                        if let Ok(row) = row_res {
                            if row.id == "text" { return Some(row.data); }
                        }
                    }
                }
                return None;
            }
        }
    }
    Some(data.to_vec())
}

/// Extract name from a subordinate object header.
/// Tries multiple known positional paths.
fn extract_item_name(header_data: &[u8]) -> Option<String> {
    let source = data_to_string(header_data)?;
    let parser = StructParser::new(source).ok()?;
    // Form name paths
    parser.get_leaf(&[1, 1, 1, 1, 2])
        // Template name path
        .or_else(|| parser.get_leaf(&[1, 2, 2]))
        // Common module / role / language name
        .or_else(|| parser.get_leaf(&[1, 1, 2]))
        // Fallback paths
        .or_else(|| parser.get_leaf(&[2, 2, 2, 3]))
        .or_else(|| parser.get_leaf(&[2, 2, 2, 2, 3]))
        .map(|n| strip_quotes(n).to_string())
        .filter(|n| !n.is_empty() && n.len() <= 150)
}

fn is_protected_module(data: &[u8]) -> bool {
    if data.is_empty() { return false; }
    if data.starts_with(UTF8_BOM) { return false; }
    if let Ok(s) = std::str::from_utf8(data) {
        let t = s.trim_start();
        if t.starts_with("//") || t.starts_with("Процедура")
            || t.starts_with("Функция") || t.starts_with("Procedure")
            || t.starts_with("Function") { return false; }
    }
    true
}

fn has_real_content(data: &[u8]) -> bool {
    let stripped = strip_bom(data);
    !stripped.is_empty() && stripped.iter().any(|&b| !b.is_ascii_whitespace())
}

// ---------------------------------------------------------------------------
// Row collection & root parsing
// ---------------------------------------------------------------------------

// collect_rows removed

fn parse_root(rows_map: &HashMap<String, Vec<u8>>) -> Result<(String, StructParser), BuildVfsError> {
    // EPF/ERF/CF: have a "root" row with {2, UUID}
    if let Some(root_data) = rows_map.get("root") {
        let root_str = data_to_string(root_data)
            .ok_or_else(|| BuildVfsError::MetadataError("'root' not UTF-8".into()))?;
        let root_parser = StructParser::new(root_str)
            .map_err(|e| BuildVfsError::MetadataError(format!("root parse: {}", e)))?;
        let root_uuid = root_parser.get_leaf(&[1])
            .ok_or_else(|| BuildVfsError::MetadataError("root[1] UUID missing".into()))?
            .to_string();
        return Ok((root_uuid, root_parser));
    }

    // CFE: no "root" row. Find the extension descriptor by looking for the
    // largest UUID row (the extension root descriptor is typically the biggest).
    let utility_rows = ["version", "versions", "copyinfo", "configinfo"];
    let mut best: Option<(&str, usize)> = None;
    for (id, data) in rows_map {
        if utility_rows.contains(&id.as_str()) || id.ends_with(".0") {
            continue;
        }
        if id.contains('-') {
            let size = data.len();
            if best.is_none() || size > best.unwrap().1 {
                best = Some((id.as_str(), size));
            }
        }
    }

    if let Some((uuid, _)) = best {
        let uuid_str = uuid.to_string();
        // Create a synthetic root parser with the UUID
        let synthetic = format!("{{2,{},}}", uuid_str);
        let root_parser = StructParser::new(synthetic)
            .map_err(|e| BuildVfsError::MetadataError(format!("synthetic root: {}", e)))?;
        return Ok((uuid_str, root_parser));
    }

    Err(BuildVfsError::MetadataError("no 'root' or 'configinfo' found".into()))
}

fn parse_object(rows_map: &HashMap<String, Vec<u8>>, uuid: &str) -> Result<StructParser, BuildVfsError> {
    let data = rows_map.get(uuid)
        .ok_or_else(|| BuildVfsError::MetadataError(format!("object '{}' not found", uuid)))?;
    let s = data_to_string(data)
        .ok_or_else(|| BuildVfsError::MetadataError(format!("object '{}' not UTF-8", uuid)))?;
    StructParser::new(s).map_err(|e| BuildVfsError::MetadataError(format!("parse '{}': {}", uuid, e)))
}

// ---------------------------------------------------------------------------
// Subordinate enumeration (Forms, Templates, Commands, etc.)
// ---------------------------------------------------------------------------

/// Collected subordinate instances grouped by type name.
struct SubordinateGroup {
    display_name: String,
    instance_uuids: Vec<String>,
}

/// Enumerate subordinate types from a metadata branch.
/// `parser` is the object parser, `base_path` is the path to the branch
/// containing the subordinate groups (e.g., &[3,1] for EPF or &[] for CF objects).
/// `start_idx` is the first index where subordinate groups begin.
fn enumerate_subordinates(parser: &StructParser, base_path: &[usize], start_idx: usize) -> Vec<SubordinateGroup> {
    let mut groups = Vec::new();
    let branch_len = match parser.branch_len(base_path) {
        Some(len) => len,
        None => return groups,
    };

    for idx in start_idx..branch_len {
        let mut type_path = base_path.to_vec();
        type_path.push(idx);

        // type_uuid at [base][idx][0]
        let mut uuid_path = type_path.clone();
        uuid_path.push(0);

        if let Some(type_uuid) = parser.get_leaf(&uuid_path) {
            let type_uuid = type_uuid.trim();
            let display_name = uuids::metadata_type_name(type_uuid)
                .unwrap_or("Unknown")
                .to_string();

            // Count at [base][idx][1], instances at [base][idx][2..]
            let mut instance_uuids = Vec::new();
            if let Some(grp_len) = parser.branch_len(&type_path) {
                for j in 2..grp_len {
                    let mut inst_path = type_path.clone();
                    inst_path.push(j);
                    if let Some(uuid) = parser.get_leaf(&inst_path) {
                        let uuid = uuid.trim();
                        if !uuid.is_empty() && uuid != "0" && uuid != "\"\"" {
                            instance_uuids.push(uuid.to_string());
                        }
                    }
                }
            }

            if !instance_uuids.is_empty() && display_name != "Unknown" {
                groups.push(SubordinateGroup { display_name, instance_uuids });
            }
        }
    }
    groups
}

// ---------------------------------------------------------------------------
// Build VFS for subordinate items
// ---------------------------------------------------------------------------

/// Build VFS entries for subordinate groups of a single object.
fn build_subordinate_entries(
    rows_map: &HashMap<String, Vec<u8>>,
    groups: &[SubordinateGroup],
) -> Vec<VfsEntry> {
    let mut entries = Vec::new();

    for group in groups {
        let mut children = Vec::new();

        for inst_uuid in &group.instance_uuids {
            let header = rows_map.get(inst_uuid);
            let body_key = format!("{}.0", inst_uuid);
            let body = rows_map.get(&body_key);

            let name = header
                .and_then(|d| extract_item_name(d))
                .unwrap_or_else(|| short_uuid(inst_uuid));

            if group.display_name == "Forms" {
                // Form: directory with Module.bsl inside
                let module_data = body.map(|b| b.to_vec()).unwrap_or_default();
                let is_prot = is_protected_module(&module_data);
                children.push(VfsEntry::Dir {
                    name,
                    children: vec![VfsEntry::File {
                        name: "Module.bsl".to_string(),
                        data: module_data,
                        is_protected: is_prot,
                    }],
                });
            } else if group.display_name == "Templates" {
                // Template: file with raw data
                let data = body.cloned().unwrap_or_default();
                children.push(VfsEntry::File {
                    name,
                    data,
                    is_protected: false,
                });
            } else {
                // Other subordinate types: show as file/dir depending on body presence
                let data = body.cloned().unwrap_or_default();
                children.push(VfsEntry::File {
                    name,
                    data,
                    is_protected: false,
                });
            }
        }

        if !children.is_empty() {
            entries.push(VfsEntry::Dir {
                name: group.display_name.clone(),
                children,
            });
        }
    }
    entries
}

fn short_uuid(uuid: &str) -> String {
    if uuid.len() >= 8 { uuid[..8].to_string() } else { uuid.to_string() }
}

// ---------------------------------------------------------------------------
// Single-object VFS (EPF/ERF/CFE)
// ---------------------------------------------------------------------------

fn build_single_object_vfs(
    rows_map: &HashMap<String, Vec<u8>>,
    root_uuid: &str,
) -> Result<Vec<VfsEntry>, BuildVfsError> {
    let root_obj = parse_object(rows_map, root_uuid)?;
    let mut vfs = Vec::new();

    // ObjectModule.bsl from [3][1][1][3][1][1][2] → {uuid}.0
    let module_uuid = root_obj.get_leaf(&[3, 1, 1, 3, 1, 1, 2]).map(|s| s.to_string());
    let body_data = module_uuid.as_ref()
        .and_then(|u| rows_map.get(&format!("{}.0", u)))
        .or_else(|| rows_map.get(&format!("{}.0", root_uuid)));

    if let Some(body) = body_data {
        if let Some(text) = extract_module_text(body) {
            if has_real_content(&text) {
                vfs.push(VfsEntry::File {
                    name: "ObjectModule.bsl".to_string(),
                    is_protected: is_protected_module(&text),
                    data: text,
                });
            }
        }
    }

    // Enumerate subordinates from [3][1], starting at index 3
    let groups = enumerate_subordinates(&root_obj, &[3, 1], 3);
    vfs.extend(build_subordinate_entries(rows_map, &groups));

    // Fallback: if VFS empty, show raw non-utility rows
    if vfs.is_empty() {
        for (id, data) in rows_map {
            if id != "root" && id != "version" && id != "versions" && id != "copyinfo" {
                vfs.push(VfsEntry::File {
                    name: id.clone(),
                    data: data.clone(),
                    is_protected: is_protected_module(data),
                });
            }
        }
    }

    Ok(vfs)
}

// ---------------------------------------------------------------------------
// Configuration VFS (CF/CFE with multiple top-level groups)
// ---------------------------------------------------------------------------

/// Extract object name from its descriptor row.
/// Object descriptor: [N][1][1][3][1][2] = "Name" (same as EPF [3][1][1][3][1][2])
/// For objects embedded in CF, the path may differ — try multiple.
fn extract_object_name(rows_map: &HashMap<String, Vec<u8>>, uuid: &str) -> String {
    if let Some(data) = rows_map.get(uuid) {
        if let Some(s) = data_to_string(data) {
            if let Ok(parser) = StructParser::new(s) {
                // Try common paths for object name
                let name = parser.get_leaf(&[1, 3, 1, 2])        // CF object
                    .or_else(|| parser.get_leaf(&[3, 1, 1, 3, 1, 2]))  // EPF-like
                    .or_else(|| parser.get_leaf(&[1, 1, 2]))            // simple objects (role, language)
                    .or_else(|| parser.get_leaf(&[1, 2, 2]));           // template-like

                if let Some(n) = name {
                    let stripped = strip_quotes(n);
                    if !stripped.is_empty() && stripped.len() <= 150 {
                        return stripped.to_string();
                    }
                }
            }
        }
    }
    short_uuid(uuid)
}

/// Build VFS for a single CF top-level object (e.g., a Report or Catalog).
/// Recursively includes its Forms, Templates, Commands, etc.
fn build_cf_object_vfs(
    rows_map: &HashMap<String, Vec<u8>>,
    obj_uuid: &str,
) -> Vec<VfsEntry> {
    let mut children = Vec::new();

    // Try to parse the object descriptor and find its subordinates
    if let Ok(obj_parser) = parse_object(rows_map, obj_uuid) {
        // Object module: from [1][3][1][1][2] → {uuid}.0
        let module_uuid = obj_parser.get_leaf(&[1, 3, 1, 1, 2])
            .or_else(|| obj_parser.get_leaf(&[3, 1, 1, 3, 1, 1, 2]))
            .map(|s| s.to_string());
        let body_data = module_uuid.as_ref()
            .and_then(|u| rows_map.get(&format!("{}.0", u)))
            .or_else(|| rows_map.get(&format!("{}.0", obj_uuid)));

        if let Some(body) = body_data {
            if let Some(text) = extract_module_text(body) {
                if has_real_content(&text) {
                    children.push(VfsEntry::File {
                        name: "ObjectModule.bsl".to_string(),
                        is_protected: is_protected_module(&text),
                        data: text,
                    });
                }
            }
        }

        // Enumerate subordinates.
        // In CF objects, subordinate types start at index [2] or [3] of root.
        // The count is at [2], subordinates at [3..].
        let count_str = obj_parser.get_leaf(&[2]).unwrap_or("0");
        let start_idx: usize = count_str.parse::<usize>().ok()
            .map(|_| 3)     // If [2] is a count, subordinates start at [3]
            .unwrap_or(3);
        let groups = enumerate_subordinates(&obj_parser, &[], start_idx);
        children.extend(build_subordinate_entries(rows_map, &groups));
        
        // Fallback: if VFS empty, show raw non-utility rows for this object
        if children.is_empty() {
            let mut added = std::collections::HashSet::new();
            if let Some(data) = rows_map.get(obj_uuid) {
                children.push(VfsEntry::File {
                    name: obj_uuid.to_string(),
                    data: data.clone(),
                    is_protected: is_protected_module(data),
                });
                added.insert(obj_uuid.to_string());
            }
            let obj_0 = format!("{}.0", obj_uuid);
            if let Some(data) = rows_map.get(&obj_0) {
                children.push(VfsEntry::File {
                    name: obj_0.clone(),
                    data: data.clone(),
                    is_protected: is_protected_module(data),
                });
                added.insert(obj_0);
            }
            if let Some(muid) = module_uuid {
                if !added.contains(&muid) {
                    if let Some(data) = rows_map.get(&muid) {
                        children.push(VfsEntry::File {
                            name: muid.clone(),
                            data: data.clone(),
                            is_protected: is_protected_module(data),
                        });
                        added.insert(muid.clone());
                    }
                }
                let muid_0 = format!("{}.0", muid);
                if !added.contains(&muid_0) {
                    if let Some(data) = rows_map.get(&muid_0) {
                        children.push(VfsEntry::File {
                            name: muid_0,
                            data: data.clone(),
                            is_protected: is_protected_module(data),
                        });
                    }
                }
            }
        }
    }

    children
}

/// Build VFS for a CF container with multiple top-level groups.
fn build_configuration_vfs(
    rows_map: &HashMap<String, Vec<u8>>,
    root_uuid: &str,
) -> Result<Vec<VfsEntry>, BuildVfsError> {
    let root_obj = parse_object(rows_map, root_uuid)?;
    let mut vfs = Vec::new();

    // CF root object: each top-level element [N] is either:
    //   - A metadata group: {group_uuid, {details...}} where group_uuid is a MetaDataGroup
    //   - Or a branch with subordinate type listings
    //
    // Structure observed:
    //   [N] = {group_uuid, {ver, {name_data}, count, {sub_type_1}, ...}}
    //   Each sub_type: {type_uuid, count, obj_uuid_1, obj_uuid_2, ...}

    let root_len = root_obj.branch_len(&[]).unwrap_or(0);

    for idx in 0..root_len {
        // Check if [idx][0] is a metadata group UUID
        let group_uuid = root_obj.get_leaf(&[idx, 0]);

        if let Some(group_uuid) = group_uuid {
            if uuids::is_metadata_group(group_uuid) {
                // Enumerate object types in this group.
                // Different groups use different nesting:
                //   General: types at [idx][1][N] for N >= 3
                //   Main:    types at [idx][1][1][N] for N >= 3
                let mut groups = enumerate_subordinates(&root_obj, &[idx, 1], 3);
                let deeper = enumerate_subordinates(&root_obj, &[idx, 1, 1], 3);
                groups.extend(deeper);

                for group in &groups {
                    let mut obj_entries = Vec::new();

                    for obj_uuid in &group.instance_uuids {
                        let obj_name = extract_object_name(rows_map, obj_uuid);
                        let obj_children = build_cf_object_vfs(rows_map, obj_uuid);

                        obj_entries.push(VfsEntry::Dir {
                            name: obj_name, children: obj_children,
                        });
                    }

                    if !obj_entries.is_empty() {
                        vfs.push(VfsEntry::Dir {
                            name: group.display_name.clone(),
                            children: obj_entries,
                        });
                    }
                }
            }
        }
    }

    // Fallback if nothing was found
    if vfs.is_empty() {
        for (id, data) in rows_map {
            if id != "root" && id != "version" && id != "versions" && id != "copyinfo" {
                vfs.push(VfsEntry::File {
                    name: id.clone(),
                    data: data.clone(),
                    is_protected: is_protected_module(data),
                });
            }
        }
    }

    Ok(vfs)
}

// ---------------------------------------------------------------------------
// Unified entry point
// ---------------------------------------------------------------------------

/// Detect whether the root object is a single-object format (EPF/ERF/CFE)
/// or a multi-group configuration (CF).
///
/// Heuristic: EPF/ERF/CFE root objects have [3][0] as a metadata group UUID
/// (or [3] is a branch with [3][0] being a known type UUID like c3831ec8...).
/// CF root objects have [0] as a branch (metadata group) or [0] as a group UUID.
fn is_configuration_format(rows_map: &HashMap<String, Vec<u8>>, root_uuid: &str) -> bool {
    // 1C 8.0+: The actual configuration metadata tree is structured with multiple groups.
    // In CF files, the main configuration descriptor can have UUID 7f84e2f0... or 30ffe4cc...
    
    let empty_vec = Vec::new();
    let root_data = rows_map.get(root_uuid).unwrap_or(&empty_vec);
    
    // Check if the raw string representation contains configuration-specific metadata group UUIDs
    let s = String::from_utf8_lossy(root_data).to_lowercase();
    let has_reports = s.contains("631b75a0-29e2-11d6-a3c7-0050bae0a776");
    let has_catalogs = s.contains("cf4abea6-37b2-11d4-940f-008048da11f9");
    let has_documents = s.contains("061d872a-5787-460e-95ac-ed74ea3a3e84");
    let has_dataprocessors = s.contains("bf845118-327b-4682-b5c6-285d2a0eb296") || s.contains("84f1eb25-06ab-445a-8b89-9a2eb242cecd");
    
    // CF has multiple groups
    let groups = [has_reports, has_catalogs, has_documents, has_dataprocessors];
    let group_count = groups.iter().filter(|&&x| x).count();
    
    group_count >= 1  // CF has at least one top-level group or Reports
}

/// Build VFS tree for any 1C container (EPF/ERF/CF/CFE).
///
/// Automatically detects the container format and applies the appropriate
/// parsing strategy:
/// - EPF/ERF: single object with subordinate Forms/Templates (has "root" row)
/// - CF: configuration with multiple top-level metadata groups (has "root" row, multiple groups)
/// - CFE: extension with flat object list (has "configinfo" row, no "root")
pub fn build_vfs(rows_map: HashMap<String, Vec<u8>>) -> Result<Vec<VfsEntry>, BuildVfsError> {
    // CFE detection: has "configinfo" but no "root"
    if !rows_map.contains_key("root") && rows_map.contains_key("configinfo") {
        return build_extension_vfs(&rows_map);
    }

    let (root_uuid, _root_parser) = parse_root(&rows_map)?;
    
    if is_configuration_format(&rows_map, &root_uuid) {
        build_configuration_vfs(&rows_map, &root_uuid)
    } else {
        build_single_object_vfs(&rows_map, &root_uuid)
    }
}

/// Build VFS for a CFE (extension) container.
///
/// CFE stores objects flat — each object is a separate row with its own
/// subordinate type tree. We enumerate all UUID rows, classify each object,
/// and build a grouped tree.
fn build_extension_vfs(
    rows_map: &HashMap<String, Vec<u8>>,
) -> Result<Vec<VfsEntry>, BuildVfsError> {
    let utility_rows = ["version", "versions", "copyinfo", "configinfo"];
    let mut groups: HashMap<String, Vec<VfsEntry>> = HashMap::new();

    // Find the extension root descriptor (largest UUID row)
    let mut root_uuid: Option<String> = None;
    let mut root_size = 0;
    for (id, data) in rows_map {
        if utility_rows.contains(&id.as_str()) || id.ends_with(".0") { continue; }
        if id.contains('-') && data.len() > root_size {
            root_size = data.len();
            root_uuid = Some(id.clone());
        }
    }

    // Parse the extension root to enumerate its top-level object references
    if let Some(ref ext_root_uuid) = root_uuid {
        if let Ok(root_obj) = parse_object(rows_map, ext_root_uuid) {
            let root_len = root_obj.branch_len(&[]).unwrap_or(0);

            for idx in 0..root_len {
                // Look for metadata group branches: [idx] = {group_uuid, {details...}}
                if let Some(group_uuid) = root_obj.get_leaf(&[idx, 0]) {
                    if uuids::is_metadata_group(group_uuid) {
                        // Try both nesting levels (same as CF)
                        let mut sub_groups = enumerate_subordinates(&root_obj, &[idx, 1], 3);
                        let deeper = enumerate_subordinates(&root_obj, &[idx, 1, 1], 3);
                        sub_groups.extend(deeper);
                        for sub_group in &sub_groups {
                            let mut obj_entries = Vec::new();
                            for obj_uuid in &sub_group.instance_uuids {
                                let obj_name = extract_object_name(rows_map, obj_uuid);
                                let obj_children = build_cf_object_vfs(rows_map, obj_uuid);

                                obj_entries.push(VfsEntry::Dir {
                                    name: obj_name, children: obj_children,
                                });
                            }
                            if !obj_entries.is_empty() {
                                groups.entry(sub_group.display_name.clone())
                                    .or_default()
                                    .extend(obj_entries);
                            }
                        }
                    }
                }
            }
        }
    }

    // Build VFS from grouped entries
    let mut vfs: Vec<VfsEntry> = groups.into_iter()
        .map(|(name, children)| VfsEntry::Dir { name, children })
        .collect();

    // Sort groups alphabetically for consistent display
    vfs.sort_by(|a, b| a.name().cmp(b.name()));

    // Fallback if empty
    if vfs.is_empty() {
        for (id, data) in rows_map {
            if !utility_rows.contains(&id.as_str()) && !id.ends_with(".0") {
                vfs.push(VfsEntry::File {
                    name: id.clone(), data: data.clone(), is_protected: false,
                });
            }
        }
    }

    Ok(vfs)
}



// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum BuildVfsError {
    Io(std::io::Error),
    MetadataError(String),
}

impl From<std::io::Error> for BuildVfsError {
    fn from(e: std::io::Error) -> Self { BuildVfsError::Io(e) }
}

impl std::fmt::Display for BuildVfsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildVfsError::Io(e) => write!(f, "I/O error: {}", e),
            BuildVfsError::MetadataError(msg) => write!(f, "Metadata error: {}", msg),
        }
    }
}

impl std::error::Error for BuildVfsError {}