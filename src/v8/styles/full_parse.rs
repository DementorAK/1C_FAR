use crate::base::parser::StructParser;
use crate::base::reader::StringReader;
use crate::v8::container::{is_container_data, Container};
use crate::v8::styles::PresentationStyle;
use crate::v8::vfs_builder::{BuildVfsError, VfsEntry};
use crate::v8::writer::ContainerWriter;
use std::collections::HashMap;

const UTF8_BOM: &[u8] = &[0xEF, 0xBB, 0xBF];

pub struct FullParseStyle;

fn determine_extension(data: &[u8]) -> &'static str {
    if data.starts_with(UTF8_BOM) {
        if let Ok(s) = std::str::from_utf8(&data[3..]) {
            let t = s.trim_start();
            if t.starts_with('{') {
                return ".txt";
            }
            if t.contains("Процедура")
                || t.contains("Функция")
                || t.contains("Procedure")
                || t.contains("Function")
            {
                return ".bsl";
            }
        }
        ".txt"
    } else {
        ".bin"
    }
}

/// Detect whether the row data is a bracket-structured text (BOM + starts with `{`).
/// Used to decide whether to re-canonicalise via the AST serializer on save.
pub fn is_bracket_text(data: &[u8]) -> bool {
    if !data.starts_with(UTF8_BOM) {
        return false;
    }
    if let Ok(s) = std::str::from_utf8(&data[3..]) {
        return s.trim_start().starts_with('{');
    }
    false
}

/// Validate and canonicalise bracket-format data via the AST serializer.
///
/// Returns Some(re_serialised) when:
///   - the data is a UTF-8 bracket-structured text (BOM + `{...}` body), and
///   - re-parsing the AST-serialised output succeeds.
///
/// The output preserves the BOM and uses the compact bracket form
/// (`{a,{b,c},d}`). When parsing fails (e.g. the user introduced malformed
/// brackets), `None` is returned and the caller falls back to writing the raw
/// bytes verbatim so no user data is lost.
pub fn canonicalize_bracket_content(data: &[u8]) -> Option<Vec<u8>> {
    if !is_bracket_text(data) {
        return None;
    }
    let body_str = std::str::from_utf8(&data[3..]).ok()?;
    let parser = StructParser::new(body_str.to_string()).ok()?;
    let serialised = parser.to_bracket();
    // Re-validate by reparsing our own output (defensive — both should
    // succeed, but if they don't we keep the original bytes).
    let _re_check = StructParser::new(serialised.clone()).ok()?;
    let mut out = Vec::with_capacity(UTF8_BOM.len() + serialised.len());
    out.extend_from_slice(UTF8_BOM);
    out.extend_from_slice(serialised.as_bytes());
    Some(out)
}

impl FullParseStyle {
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

            let ext = determine_extension(data);
            vfs.push(VfsEntry::File {
                name: format!("{}{}", id, ext),
                data: data.clone(),
                is_protected: false,
                origin_row_id: Some(id.clone()),
                original_container: None,
                origin_row_packed: packed_map.get(id).copied(),
            });
        }

        vfs.sort_by(|a, b| {
            let a_is_dir = matches!(a, VfsEntry::Dir { .. });
            let b_is_dir = matches!(b, VfsEntry::Dir { .. });
            if a_is_dir && !b_is_dir {
                std::cmp::Ordering::Less
            } else if !a_is_dir && b_is_dir {
                std::cmp::Ordering::Greater
            } else {
                a.name().cmp(b.name())
            }
        });

        Ok(vfs)
    }
}

impl PresentationStyle for FullParseStyle {
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
        for entry in vfs {
            match entry {
                VfsEntry::File {
                    name,
                    data,
                    origin_row_id: Some(row_id),
                    origin_row_packed,
                    ..
                } => {
                    // For bracket-structured text files (`.txt`), re-canonicalise
                    // via the Bracket-AST serializer to validate and normalise the
                    // user's edits. `.bsl` modules and `.bin` blobs are written
                    // verbatim. If canonicalisation fails (malformed bracket input),
                    // fall back to the raw bytes to avoid losing user data.
                    let final_data = if name.ends_with(".txt") {
                        canonicalize_bracket_content(data).unwrap_or_else(|| data.clone())
                    } else {
                        data.clone()
                    };
                    updates.insert(row_id.clone(), final_data);
                    packed_out.insert(row_id.clone(), origin_row_packed.unwrap_or(false));
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
    }
}
