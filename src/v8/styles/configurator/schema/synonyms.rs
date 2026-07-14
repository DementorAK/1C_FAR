//! Synonym extraction and XML rendering — language-agnostic.
//!
//! All synonym data in 1C bracket format is stored as
//! `{count, lang1, content1, lang2, content2, ...}`.
//! We iterate all pairs without hardcoding language codes.

use serde_json::Value;

/// Extract all (lang, content) synonym pairs from a synonym array.
/// Input: `[count_str, lang1, content1, lang2, content2, ...]`
pub fn extract_synonyms_from_arr(syn_arr: &[Value]) -> Vec<(String, String)> {
    let mut result = Vec::new();
    if let Some(Value::String(count_str)) = syn_arr.first() {
        let count: usize = count_str.trim_matches('"').parse().unwrap_or(0);
        for i in 0..count {
            let lang_idx = 1 + i * 2;
            let content_idx = 2 + i * 2;
            if let (Some(lang_v), Some(content_v)) =
                (syn_arr.get(lang_idx), syn_arr.get(content_idx))
            {
                let lang = lang_v
                    .as_str()
                    .map(|s| s.trim_matches('"').to_string())
                    .unwrap_or_default();
                let content = content_v
                    .as_str()
                    .map(|s| s.trim_matches('"').to_string())
                    .unwrap_or_default();
                if !lang.is_empty() {
                    result.push((lang, content));
                }
            }
        }
    }
    result
}

/// Extract synonyms from a tc=27 attribute definition.
/// Path: `item_arr[1][1][3]` = `[count, lang1, content1, ...]`
pub fn extract_tc27_synonyms(item_arr: &[Value]) -> Vec<(String, String)> {
    let syn_arr = item_arr
        .get(1)
        .and_then(|v| v.as_array())
        .and_then(|a| a.get(1))
        .and_then(|v| v.as_array())
        .and_then(|s| s.get(3))
        .and_then(|v| v.as_array());
    match syn_arr {
        Some(arr) => extract_synonyms_from_arr(arr),
        None => Vec::new(),
    }
}

/// Extract synonyms from a property sub-array (used in TabularSection gen).
/// Path: `sub[3]` = `[count, lang1, content1, ...]`
pub fn extract_tc27_synonyms_from_sub(sub: &[Value]) -> Vec<(String, String)> {
    match sub.get(3).and_then(|v| v.as_array()) {
        Some(arr) => extract_synonyms_from_arr(arr),
        None => Vec::new(),
    }
}

/// Extract all synonyms from a standard metadata object JSON value.
///
/// Covers:
/// - Form/Language/Role subordinate header (3-element structure) `["1", [data], "0"]`:
///   synonyms at `root[1][1][0][3]`
/// - Standard EPF/ERF/CF 4-element root: synonyms at `root[3][1][3]`
/// - EPF/ERF object with nested type_data: `root[3][1][3][1][3]`
pub fn extract_all_synonyms(json_val: &Value) -> Vec<(String, String)> {
    if let Some(root) = json_val.as_array() {
        // Form/Language/Role subordinate header (3-element structure):
        // root = ["1", ["1", ["0", [tc, [props_branch]]]], "0"]
        // The outer root[1] is the data wrapper, root[1][1] is the inner data,
        // root[1][1][1] is [tc, [props_branch]], root[1][1][1][1] is props_branch.
        // props_branch[3] = synonym array
        if root.len() == 3 {
            if let Some(syn_arr) = root
                .get(1)
                .and_then(|v| v.as_array())
                .and_then(|a| a.get(1))
                .and_then(|v| v.as_array())
                .and_then(|a| a.get(1))
                .and_then(|v| v.as_array())
                .and_then(|a| a.get(1))
                .and_then(|v| v.as_array())
                .and_then(|pb| pb.get(3))
                .and_then(|v| v.as_array())
            {
                let syns = extract_synonyms_from_arr(syn_arr);
                if !syns.is_empty() {
                    return syns;
                }
            }
        }

        // Standard 4-element: root[3][1][3] = synonym array
        if let Some(syn_arr) = root
            .get(3)
            .and_then(|v| v.as_array())
            .and_then(|a| a.get(1))
            .and_then(|v| v.as_array())
            .and_then(|pb| pb.get(3))
            .and_then(|v| v.as_array())
        {
            let syns = extract_synonyms_from_arr(syn_arr);
            if !syns.is_empty() {
                return syns;
            }
        }

        // EPF/ERF: root[3][1][3][1][3] (props inside type_data)
        if let Some(syn_arr) = root
            .get(3)
            .and_then(|v| v.as_array())
            .and_then(|a| a.get(1))
            .and_then(|v| v.as_array())
            .and_then(|td| td.get(3))
            .and_then(|v| v.as_array())
            .and_then(|po| po.get(1))
            .and_then(|v| v.as_array())
            .and_then(|pb| pb.get(3))
            .and_then(|v| v.as_array())
        {
            let syns = extract_synonyms_from_arr(syn_arr);
            if !syns.is_empty() {
                return syns;
            }
        }
    }
    Vec::new()
}

/// Render `<Synonym>` XML block from synonym pairs.
/// `indent` is the base indentation of the `<Synonym>` tag itself.
pub fn render_synonym_xml(synonyms: &[(String, String)], indent: &str) -> String {
    if synonyms.is_empty() {
        return format!("{}<Synonym/>\n", indent);
    }
    let item_indent = format!("{}\t", indent);
    let mut xml = format!("{}<Synonym>\n", indent);
    for (lang, content) in synonyms {
        xml.push_str(&format!(
            "{}<v8:item>\n{}\t<v8:lang>{}</v8:lang>\n{}\t<v8:content>{}</v8:content>\n{}</v8:item>\n",
            item_indent, item_indent, lang, item_indent, content, item_indent
        ));
    }
    xml.push_str(&format!("{}</Synonym>\n", indent));
    xml
}

/// Render synonym `<v8:item>` entries at the given indent level (without wrapping tags).
/// Used inside pre-built format strings where only the inner items are needed.
pub fn render_synonym_items(synonyms: &[(String, String)], indent: &str) -> String {
    let mut xml = String::new();
    for (lang, content) in synonyms {
        xml.push_str(&format!(
            "{}<v8:item>\n{}\t<v8:lang>{}</v8:lang>\n{}\t<v8:content>{}</v8:content>\n{}</v8:item>\n",
            indent, indent, lang, indent, content, indent
        ));
    }
    xml
}
