//! Smart re-wrap helper extracted from json/edt/configurator styles (C3/C4 dedup).
//!
//! When a presentation style exposes a nested 1C container (typically a form
//! module) as a single file editable by the user, the original bytes of that
//! nested container must be rebuilt on save. The container template is
//! preserved as `VfsEntry::File::original_container` and used as a structural
//! template: the user's edited module text replaces the `text` row inside
//! the nested container, while every other nested row keeps its contents
//! and (critically) its `is_packed` flag.

use crate::base::reader::StringReader;
use crate::v8::container::read_container_rows;
use crate::v8::writer::ContainerWriter;
use std::collections::HashMap;

/// Re-wrap a user-edited module body back into a nested 1C container blob.
///
/// `original_container` is the byte representation of the original nested
/// container (captured at parse time and stored on the VFS file entry).
/// `new_text` is the user's edited module bytes (treated as uncompressed —
/// the nested `text` row's packed flag inherited from the template decides
/// whether the writer deflates it again).
///
/// Returns `None` if the template cannot be parsed (in which case the caller
/// MUST fall back to writing the raw user bytes to avoid data loss).
///
/// The re-wrapped blob is left uncompressed at the parent level; whether it
/// is deflate-compressed in the outer container is decided by the caller's
/// per-row `packed_out` entry — typically preserved from `origin_row_packed`.
pub fn smart_rewrap_module(original_container: &[u8], new_text: &[u8]) -> Option<Vec<u8>> {
    let mut nested_rows: HashMap<String, (Vec<u8>, bool)> =
        read_container_rows(StringReader::new(original_container.to_vec()), 0).ok()?;

    // Inherit the original "text" row packed flag, defaulting to true —
    // most module rows are deflate-compressed in real artifacts, so when
    // the template has no row entry yet, we err on the side of compressing.
    let text_packed = nested_rows.get("text").map(|(_, p)| *p).unwrap_or(true);
    nested_rows.insert("text".to_string(), (new_text.to_vec(), text_packed));

    let mut writer = ContainerWriter::new(512, false);
    writer.use_triplets = true;
    writer.pad_pt_to_page = false;
    writer.revision = 6;

    let mut buffer = Vec::new();
    writer
        .write(&mut buffer, &nested_rows, None::<fn(usize, usize)>)
        .ok()?;
    Some(buffer)
}

/// Re-wrap a user-edited module body, but accept a pre-computed `text_packed`
/// override. Useful for callers that need to force a particular compression
/// behaviour on the nested `text` row regardless of the template (e.g. when
/// migrating from JSON-editable text that was always uncompressed).
///
/// Returns `None` on parse failure, same fallback semantics as
/// [`smart_rewrap_module`].
pub fn smart_rewrap_module_with_packed(
    original_container: &[u8],
    new_text: &[u8],
    text_packed: bool,
) -> Option<Vec<u8>> {
    let mut nested_rows: HashMap<String, (Vec<u8>, bool)> =
        read_container_rows(StringReader::new(original_container.to_vec()), 0).ok()?;
    nested_rows.insert("text".to_string(), (new_text.to_vec(), text_packed));

    let mut writer = ContainerWriter::new(512, false);
    writer.use_triplets = true;
    writer.pad_pt_to_page = false;
    writer.revision = 6;

    let mut buffer = Vec::new();
    writer
        .write(&mut buffer, &nested_rows, None::<fn(usize, usize)>)
        .ok()?;
    Some(buffer)
}

/// Collect all `origin_row_id` keys used by a VFS tree.
///
/// Used by presentation styles that emit a synthetic file (e.g. `index.json`
/// or `Configuration.mdo`) from the rows not referenced by any tree branch.
/// Returns the union of the seed set and every `origin_row_id` found in the
/// tree (recursively into directories).
pub fn collect_used_row_ids(
    entries: &[crate::v8::vfs_builder::VfsEntry],
    seed: std::collections::HashSet<String>,
) -> std::collections::HashSet<String> {
    use crate::v8::vfs_builder::VfsEntry;
    let mut used = seed;
    fn collect(entries: &[VfsEntry], used: &mut std::collections::HashSet<String>) {
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
                    collect(children, used);
                }
                VfsEntry::Dir { children, .. } => {
                    collect(children, used);
                }
                _ => {}
            }
        }
    }
    collect(entries, &mut used);
    used
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Synthetic template: {text: "hello"} container, used to exercise the
    /// re-wrap helper without depending on real 1C fixtures.
    fn make_text_template(text: &str, packed: bool) -> Vec<u8> {
        let mut rows = HashMap::new();
        rows.insert("text".to_string(), (text.as_bytes().to_vec(), packed));
        let mut writer = ContainerWriter::new(512, false);
        writer.use_triplets = true;
        writer.pad_pt_to_page = false;
        writer.revision = 6;
        let mut buffer = Vec::new();
        writer
            .write(&mut buffer, &rows, None::<fn(usize, usize)>)
            .expect("template write");
        buffer
    }

    fn extract_text(blob: &[u8]) -> (String, bool) {
        let rows = read_container_rows(StringReader::new(blob.to_vec()), 0).expect("re-read");
        let (data, packed) = rows.get("text").expect("text row present");
        (String::from_utf8_lossy(data).into_owned(), *packed)
    }

    #[test]
    fn test_rewrap_preserves_uncompressed_text() {
        let template = make_text_template("hello", false);
        let wrapped = smart_rewrap_module(&template, b"world").expect("re-wrap");
        let (text, packed) = extract_text(&wrapped);
        assert_eq!(text, "world");
        assert!(
            !packed,
            "text stays uncompressed when template is uncompressed"
        );
    }

    #[test]
    fn test_rewrap_preserves_packed_text() {
        let template = make_text_template("hello", true);
        let wrapped = smart_rewrap_module(&template, b"world").expect("re-wrap");
        let (text, packed) = extract_text(&wrapped);
        assert_eq!(text, "world");
        assert!(packed, "text stays compressed when template is compressed");
    }

    #[test]
    fn test_rewrap_with_packed_override() {
        let template_uncomp = make_text_template("hello", false);
        let wrapped =
            smart_rewrap_module_with_packed(&template_uncomp, b"forced", true).expect("re-wrap");
        let (text, packed) = extract_text(&wrapped);
        assert_eq!(text, "forced");
        assert!(
            packed,
            "override forces compression even when template is uncompressed"
        );
    }

    #[test]
    fn test_rewrap_returns_none_on_garbage() {
        assert!(smart_rewrap_module(b"not a container", b"x").is_none());
        assert!(smart_rewrap_module(b"", b"x").is_none());
    }

    #[test]
    fn test_collect_used_row_ids_traverses_dirs() {
        use crate::v8::vfs_builder::VfsEntry;
        let vfs = vec![
            VfsEntry::File {
                name: "a.bsl".to_string(),
                data: vec![],
                is_protected: false,
                origin_row_id: Some("row-1".to_string()),
                original_container: None,
                origin_row_packed: None,
            },
            VfsEntry::Dir {
                name: "Forms".to_string(),
                children: vec![VfsEntry::File {
                    name: "Module.bsl".to_string(),
                    data: vec![],
                    is_protected: false,
                    origin_row_id: Some("row-2".to_string()),
                    original_container: None,
                    origin_row_packed: None,
                }],
                origin_row_id: Some("row-dir".to_string()),
                origin_row_packed: None,
            },
            VfsEntry::Dir {
                name: "Synthetic".to_string(),
                children: vec![],
                origin_row_id: None,
                origin_row_packed: None,
            },
        ];
        let mut seed = std::collections::HashSet::new();
        seed.insert("seed-id".to_string());
        let used = collect_used_row_ids(&vfs, seed);
        assert!(used.contains("row-1"));
        assert!(used.contains("row-2"));
        assert!(used.contains("row-dir"));
        assert!(used.contains("seed-id"));
    }
}
