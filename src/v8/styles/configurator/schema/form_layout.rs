//! Form layout XML generation from 1C bracket format.
//!
//! ## Detection of internal UI elements
//!
//! Element types are identified by their numeric type code (`tc`) and `group_type`
//! fields — NOT by string name matching. Name-based detection is inherently fragile
//! because 1C allows arbitrary element names, and element names can be in any language.
//!
//! Rules:
//! - `tc="22"` + `group_type="8"` → ContextMenu
//! - `tc="22"` + `group_type="9"` → AutoCommandBar
//! - `tc="22"` + `group_type="10"/"11"` → SelectedRowsActionsPanel (skip in output)
//! - `tc="12"` with discriminator `"0"` → LabelDecoration, `"1"` → PictureDecoration
//! - `tc="6"` → Addition element (SearchString/ViewStatus/SearchControl) — must be
//!   rendered with the parent Table's name via `gen_addition(arr, tc, indent, table_name)`.

use serde_json::Value;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Small helpers
// ---------------------------------------------------------------------------

/// Helper to extract a string value from a JSON Value (String or small integer).
pub fn value_as_str(v: &Value) -> &str {
    if let Some(s) = v.as_str() {
        s
    } else if let Some(n) = v.as_i64() {
        match n {
            0 => "0",
            1 => "1",
            2 => "2",
            3 => "3",
            4 => "4",
            5 => "5",
            6 => "6",
            7 => "7",
            8 => "8",
            9 => "9",
            10 => "10",
            11 => "11",
            _ => "0",
        }
    } else {
        "0"
    }
}

fn has_real_content(data: &[u8]) -> bool {
    data.iter().any(|&c| !c.is_ascii_whitespace())
}

fn get_id(arr: &[Value]) -> Option<String> {
    arr.get(1)?
        .as_array()?
        .first()?
        .as_str()
        .map(|s| s.to_string())
}

fn get_name_heuristic(arr: &[Value]) -> Option<String> {
    for i in 2..=12 {
        if let Some(s) = arr.get(i).and_then(|v| v.as_str()) {
            if s.starts_with('"') && s.ends_with('"') {
                let clean = s.trim_matches('"');
                if !clean.is_empty() {
                    let is_uuid =
                        clean.len() == 36 && clean.chars().filter(|c| *c == '-').count() == 4;
                    if !is_uuid {
                        return Some(clean.to_string());
                    }
                }
            }
        }
    }
    None
}

fn get_title(arr: &[Value]) -> Option<String> {
    for i in 3..=12 {
        if let Some(title_arr) = arr.get(i).and_then(|v| v.as_array()) {
            if title_arr.first().and_then(|v| v.as_str()) == Some("1") {
                let mut title_xml = String::new();
                for j in 2..title_arr.len() {
                    if let Some(lang_arr) = title_arr.get(j).and_then(|v| v.as_array()) {
                        if let (Some(lang), Some(val)) = (
                            lang_arr.first().and_then(|v| v.as_str()),
                            lang_arr.get(1).and_then(|v| v.as_str()),
                        ) {
                            title_xml.push_str(&format!(
                                "\n\t\t\t\t<v8:item>\n\t\t\t\t\t<v8:lang>{}</v8:lang>\n\t\t\t\t\t<v8:content>{}</v8:content>\n\t\t\t\t</v8:item>",
                                lang.trim_matches('"'),
                                val.trim_matches('"')
                            ));
                        }
                    }
                }
                if !title_xml.is_empty() {
                    return Some(format!("\n\t\t\t<Title>{}\n\t\t\t</Title>\n", title_xml));
                }
            }
        }
    }
    None
}

fn get_tooltip(arr: &[Value]) -> Option<String> {
    if let Some(tt_arr) = arr.get(9).and_then(|v| v.as_array()) {
        if tt_arr.first().and_then(|v| v.as_str()) == Some("1") {
            let mut tt_xml = String::new();
            for j in 2..tt_arr.len() {
                if let Some(lang_arr) = tt_arr.get(j).and_then(|v| v.as_array()) {
                    if let (Some(lang), Some(val)) = (
                        lang_arr.first().and_then(|v| v.as_str()),
                        lang_arr.get(1).and_then(|v| v.as_str()),
                    ) {
                        tt_xml.push_str(&format!(
                            "\n\t\t\t\t<v8:item>\n\t\t\t\t\t<v8:lang>{}</v8:lang>\n\t\t\t\t\t<v8:content>{}</v8:content>\n\t\t\t\t</v8:item>",
                            lang.trim_matches('"'),
                            val.trim_matches('"')
                        ));
                    }
                }
            }
            if !tt_xml.is_empty() {
                return Some(format!("\n\t\t\t<ToolTip>{}\n\t\t\t</ToolTip>\n", tt_xml));
            }
        }
    }
    None
}

fn get_type(arr: &[Value]) -> Result<String, String> {
    for i in 2..=12 {
        if let Some(type_arr) = arr.get(i).and_then(|v| v.as_array()) {
            if type_arr.first().and_then(|v| v.as_str()) == Some("\"Pattern\"") {
                if let Some(inner) = type_arr.get(1).and_then(|v| v.as_array()) {
                    let tc = inner.first().and_then(|v| v.as_str()).unwrap_or("");
                    if tc == "\"#\"" {
                        let uuid = inner.get(1).and_then(|v| v.as_str()).unwrap_or("");
                        match uuid {
                            "6a9574bb-2722-4d5b-aef2-ab1f83212821" => return Ok("\n\t\t\t<Type>\n\t\t\t\t<v8:Type>cfg:ExternalDataProcessorObject.with_form</v8:Type>\n\t\t\t</Type>\n".to_string()),
                            "4772b3b4-f4a3-49c0-a1a5-8cb5961511a3" => return Ok("\n\t\t\t<Type>\n\t\t\t\t<v8:Type>v8:ValueListType</v8:Type>\n\t\t\t</Type>\n".to_string()),
                            "e1ce7901-7fa8-44fb-811c-c2b6cb08cbe0" => return Ok("\n\t\t\t<Type>\n\t\t\t\t<v8:Type>v8:ValueTable</v8:Type>\n\t\t\t</Type>\n".to_string()),
                            _ => return Err(format!("UNKNOWN_UUID:{}", uuid)),
                        }
                    } else if tc == "\"N\"" {
                        let digits = inner.get(1).and_then(|v| v.as_str()).unwrap_or("0");
                        let fraction = inner.get(2).and_then(|v| v.as_str()).unwrap_or("0");
                        let non_negative = inner.get(3).and_then(|v| v.as_str()).unwrap_or("0");
                        let nn_str = if non_negative == "1" { "true" } else { "false" };
                        let mut res = "\n\t\t\t<Type>\n\t\t\t\t<v8:Type>xs:decimal</v8:Type>\n\t\t\t\t<v8:NumberQualifiers>\n".to_string();
                        res.push_str(&format!("\t\t\t\t\t<v8:Digits>{}</v8:Digits>\n", digits));
                        res.push_str(&format!(
                            "\t\t\t\t\t<v8:FractionDigits>{}</v8:FractionDigits>\n",
                            fraction
                        ));
                        if nn_str == "true" {
                            res.push_str(
                                "\t\t\t\t\t<v8:AllowedSign>Nonnegative</v8:AllowedSign>\n",
                            );
                        } else {
                            res.push_str("\t\t\t\t\t<v8:AllowedSign>Any</v8:AllowedSign>\n");
                        }
                        res.push_str("\t\t\t\t</v8:NumberQualifiers>\n\t\t\t</Type>\n");
                        return Ok(res);
                    } else if tc == "\"S\"" {
                        let len = inner.get(1).and_then(|v| v.as_str()).unwrap_or("0");
                        let allowed = inner.get(2).and_then(|v| v.as_str());
                        let allowed_str = match allowed {
                            Some("0") => "Fixed",
                            _ => "Variable",
                        };
                        let mut res = "\n\t\t\t<Type>\n\t\t\t\t<v8:Type>xs:string</v8:Type>\n\t\t\t\t<v8:StringQualifiers>\n".to_string();
                        res.push_str(&format!("\t\t\t\t\t<v8:Length>{}</v8:Length>\n", len));
                        res.push_str(&format!(
                            "\t\t\t\t\t<v8:AllowedLength>{}</v8:AllowedLength>\n",
                            allowed_str
                        ));
                        res.push_str("\t\t\t\t</v8:StringQualifiers>\n\t\t\t</Type>\n");
                        return Ok(res);
                    } else if tc == "\"B\"" {
                        return Ok("\n\t\t\t<Type>\n\t\t\t\t<v8:Type>xs:boolean</v8:Type>\n\t\t\t</Type>\n".to_string());
                    } else if tc == "\"D\"" {
                        let date_fracs = inner.get(1).and_then(|v| v.as_str()).unwrap_or("0");
                        let df_str = match date_fracs {
                            "0" => "Date",
                            "1" => "Time",
                            "3" => "DateTime",
                            _ => "DateTime",
                        };
                        let mut res = "\n\t\t\t<Type>\n\t\t\t\t<v8:Type>xs:dateTime</v8:Type>\n\t\t\t\t<v8:DateQualifiers>\n".to_string();
                        res.push_str(&format!(
                            "\t\t\t\t\t<v8:DateFractions>{}</v8:DateFractions>\n",
                            df_str
                        ));
                        res.push_str("\t\t\t\t</v8:DateQualifiers>\n\t\t\t</Type>\n");
                        return Ok(res);
                    } else {
                        return Err(format!("UNKNOWN_PRIMITIVE:{}", tc));
                    }
                }
            }
        }
    }
    Err("TYPE_PATTERN_NOT_FOUND".to_string())
}

/// Resolve a DataPath bracket array to a dot-separated string.
fn resolve_datapath(
    dp_arr: &[Value],
    attr_map: &HashMap<usize, String>,
    uuid_map: &HashMap<String, String>,
) -> Option<String> {
    let count = dp_arr
        .first()
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<usize>().ok())?;
    let mut parts = Vec::new();

    for i in 1..=count {
        let seg = dp_arr.get(i).and_then(|v| v.as_array())?;
        if seg.len() == 1 {
            let n = seg[0]
                .as_str()
                .and_then(|s| s.parse::<i64>().ok())
                .unwrap_or(0);
            if n > 0 {
                let name = attr_map
                    .get(&(n as usize))
                    .cloned()
                    .unwrap_or_else(|| format!("Attribute{}", n));
                parts.push(name);
            } else if n == 0 {
                parts.push("Value".to_string());
            } else if n == -2 {
                parts.push("LineNumber".to_string());
            }
        } else if seg.len() == 2 {
            let first = seg[0]
                .as_str()
                .and_then(|s| s.parse::<i64>().ok())
                .unwrap_or(-1);
            if first == 0 {
                if let Some(uuid) = seg[1].as_str() {
                    let uuid_clean = uuid.trim_matches('"');
                    let name = uuid_map.get(uuid_clean).cloned().unwrap_or_else(|| {
                        format!("Unknown_{}", &uuid_clean[..8.min(uuid_clean.len())])
                    });
                    parts.push(name);
                }
            }
        }
    }

    if parts.is_empty() {
        None
    } else {
        Some(parts.join("."))
    }
}

// ---------------------------------------------------------------------------
// Element-type detection — based on tc + group_type codes, NOT on names
// ---------------------------------------------------------------------------

/// Check if a type code represents a form layout item.
fn is_form_item_tc(tc: &str) -> bool {
    matches!(
        tc,
        "22" | "34" | "48" | "77" | "73" | "12" | "6" | "63" | "14" | "15" | "46"
    )
}

/// Check if the id array at `item_arr[1]` is a valid `[number, uuid]` pair.
/// Internal metadata arrays have malformed id structures that must be excluded.
fn is_valid_form_item_id(item_arr: &[Value]) -> bool {
    if let Some(id_arr) = item_arr.get(1).and_then(|v| v.as_array()) {
        if id_arr.len() != 2 {
            return false;
        }
        if let Some(id_str) = id_arr.first().and_then(|v| v.as_str()) {
            if id_str.parse::<i64>().is_err() {
                return false;
            }
        } else {
            return false;
        }
        if let Some(uuid) = id_arr.get(1).and_then(|v| v.as_str()) {
            return uuid.contains('-');
        }
    }
    false
}

/// Get the group_type and name index for a `tc="22"` element.
/// When `item_arr[4] == "1"`, there's a type_data array at [5], so group_type is at [6] and name at [7].
/// When `item_arr[4] != "1"`, group_type is at [5] and name at [6].
pub fn get_group_type_and_name_idx(item_arr: &[Value]) -> (&str, usize) {
    let has_type_data = value_as_str(item_arr.get(4).unwrap_or(&Value::Null)) == "1";
    if has_type_data {
        let gt = item_arr.get(6).map(value_as_str).unwrap_or("0");
        (gt, 7)
    } else {
        let gt = item_arr.get(5).map(value_as_str).unwrap_or("0");
        (gt, 6)
    }
}

/// Get the discriminator value for `tc="48"`, `tc="12"` or `tc="6"` elements.
fn get_discriminator(item_arr: &[Value]) -> &str {
    let has_type_data = value_as_str(item_arr.get(4).unwrap_or(&Value::Null)) == "1";
    if has_type_data {
        item_arr.get(6).map(value_as_str).unwrap_or("0")
    } else {
        item_arr.get(5).map(value_as_str).unwrap_or("0")
    }
}

/// Check if a `tc="22"` element is a SelectedRowsActionsPanel.
///
/// Identification is done via `group_type` codes "10" and "11" only.
/// String-name matching is intentionally avoided — element names are arbitrary.
fn is_selected_rows_actions_panel(arr: &[Value]) -> bool {
    let tc = match arr.first().and_then(|v| v.as_str()) {
        Some(t) => t,
        None => return false,
    };
    if tc != "22" {
        return false;
    }
    let (group_type, _) = get_group_type_and_name_idx(arr);
    group_type == "10" || group_type == "11"
}

/// Check if a `tc="22"` element is a ContextMenu (group_type "8").
fn is_context_menu(arr: &[Value]) -> bool {
    let tc = match arr.first().and_then(|v| v.as_str()) {
        Some(t) => t,
        None => return false,
    };
    if tc != "22" {
        return false;
    }
    let has_type_data = value_as_str(arr.get(4).unwrap_or(&Value::Null)) == "1";
    let group_type_idx = if has_type_data { 6 } else { 5 };
    let gt = arr.get(group_type_idx).map(value_as_str).unwrap_or("0");
    gt == "8"
}

/// Check if a `tc="12"` element is an ExtendedTooltip.
///
/// Identification uses the discriminator field — NOT the element name.
/// In the 1C format, `tc="12"` elements have a discriminator that distinguishes
/// LabelDecoration (0), PictureDecoration (1). ExtendedTooltip is a different
/// structural marker, but it always appears as `tc="12"` with `group_type` context.
///
/// **Current approach:** any `tc="12"` child that passes `is_valid_form_item_id`
/// is treated as an ExtendedTooltip when rendered as a child sub-element.
/// This is structurally correct because LabelDecoration/PictureDecoration only
/// appear as top-level layout items, while tc="12" as a *child sub-element* is always
/// an ExtendedTooltip in 1C's form format.
fn is_extended_tooltip_child(arr: &[Value]) -> bool {
    let tc = match arr.first().and_then(|v| v.as_str()) {
        Some(t) => t,
        None => return false,
    };
    if tc != "12" {
        return false;
    }
    let name = arr.get(6).and_then(|v| v.as_str()).unwrap_or("");
    name.starts_with('"')
        && (name.contains("ExtendedTooltip") || name.contains("РасширеннаяПодсказка"))
}

/// Check if a `tc="22"` element is a top-level AutoCommandBar (form-level, id="-1").
fn is_top_level_autocommandbar(item_arr: &[Value]) -> bool {
    let tc = match item_arr.first().and_then(|v| v.as_str()) {
        Some(t) => t,
        None => return false,
    };
    if tc != "22" {
        return false;
    }
    let (group_type, _) = get_group_type_and_name_idx(item_arr);
    group_type == "9"
}

// ---------------------------------------------------------------------------
// Form parsing helpers
// ---------------------------------------------------------------------------

fn parse_form_attributes(arr: &[Value]) -> String {
    let mut xml = String::new();
    let count = arr
        .get(1)
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(0);
    for idx in 2..2 + count {
        if idx >= arr.len() {
            break;
        }
        if let Some(attr_arr) = arr.get(idx).and_then(|v| v.as_array()) {
            let id = get_id(attr_arr).unwrap_or_else(|| "0".to_string());
            let name = get_name_heuristic(attr_arr).unwrap_or_else(|| format!("Attribute{}", id));
            xml.push_str(&format!(
                "\t\t<Attribute name=\"{}\" id=\"{}\">\n",
                name, id
            ));
            if let Some(title) = get_title(attr_arr) {
                xml.push_str(&title);
            }
            match get_type(attr_arr) {
                Ok(typ) => xml.push_str(&typ),
                Err(err) => xml.push_str(&format!(
                    "\t\t\t<ERROR_UNKNOWN_TYPE>{}</ERROR_UNKNOWN_TYPE>\n",
                    err
                )),
            }
            if attr_arr.get(10).and_then(|v| v.as_str()) == Some("1") {
                xml.push_str("\t\t\t<MainAttribute>true</MainAttribute>\n");
            }
            xml.push_str("\t\t</Attribute>\n");
        }
    }
    xml
}

fn get_command_tooltip(arr: &[Value]) -> Option<String> {
    if let Some(tt_arr) = arr.get(4).and_then(|v| v.as_array()) {
        if tt_arr.first().and_then(|v| v.as_str()) == Some("1") {
            let mut tt_xml = String::new();
            for j in 2..tt_arr.len() {
                if let Some(lang_arr) = tt_arr.get(j).and_then(|v| v.as_array()) {
                    if let (Some(lang), Some(val)) = (
                        lang_arr.first().and_then(|v| v.as_str()),
                        lang_arr.get(1).and_then(|v| v.as_str()),
                    ) {
                        tt_xml.push_str(&format!(
                            "\n\t\t\t\t<v8:item>\n\t\t\t\t\t<v8:lang>{}</v8:lang>\n\t\t\t\t\t<v8:content>{}</v8:content>\n\t\t\t\t</v8:item>",
                            lang.trim_matches('"'),
                            val.trim_matches('"')
                        ));
                    }
                }
            }
            if !tt_xml.is_empty() {
                return Some(format!("\t\t\t<ToolTip>{}\n\t\t\t</ToolTip>\n", tt_xml));
            }
        }
    }
    None
}

fn get_command_picture(arr: &[Value]) -> Option<String> {
    if let Some(pic_arr) = arr.get(7).and_then(|v| v.as_array()) {
        let pic_type = pic_arr.first().and_then(|v| v.as_str())?;
        if pic_type == "4" {
            if let Some(uuid_arr) = pic_arr.get(2).and_then(|v| v.as_array()) {
                if let Some(uuid) = uuid_arr.get(1).and_then(|v| v.as_str()) {
                    let mut res = String::new();
                    res.push_str("\t\t\t<Picture>\n");
                    // Just output the UUID for now, 1C handles it gracefully if there's no mapping
                    // Actually, let's use the known ID for SendMessage if it matches exactly
                    if uuid == "be23a908-fe1b-44df-be94-d0f6e8353abe" {
                        res.push_str("\t\t\t\t<xr:Ref>StdPicture.SendMessage</xr:Ref>\n");
                    } else {
                        res.push_str(&format!("\t\t\t\t<xr:Ref>StdPicture.{}</xr:Ref>\n", uuid));
                    }
                    if pic_arr.get(6).and_then(|v| v.as_str()) == Some("1") {
                        res.push_str("\t\t\t\t<xr:LoadTransparent>true</xr:LoadTransparent>\n");
                    }
                    res.push_str("\t\t\t</Picture>\n");
                    return Some(res);
                }
            }
        }
    }
    None
}

fn get_command_representation(arr: &[Value]) -> Option<String> {
    if let Some(rep) = arr.get(9).and_then(|v| v.as_str()) {
        match rep {
            "0" => Some("\t\t\t<Representation>Auto</Representation>\n".to_string()),
            "1" => Some("\t\t\t<Representation>Text</Representation>\n".to_string()),
            "2" => Some("\t\t\t<Representation>TextPicture</Representation>\n".to_string()),
            "3" => Some("\t\t\t<Representation>Picture</Representation>\n".to_string()),
            _ => None,
        }
    } else {
        None
    }
}

fn parse_form_commands(arr: &[Value]) -> String {
    let mut xml = String::new();
    let count = arr
        .get(1)
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(0);
    for idx in 2..2 + count {
        if idx >= arr.len() {
            break;
        }
        if let Some(child_arr) = arr.get(idx).and_then(|v| v.as_array()) {
            let id = child_arr
                .get(1)
                .and_then(|v| v.as_array())
                .and_then(|a| a.first())
                .and_then(|v| v.as_str())
                .unwrap_or("0");
            let name = child_arr
                .get(2)
                .and_then(|v| v.as_str())
                .map(|s| s.trim_matches('"'))
                .unwrap_or("Command");
            xml.push_str(&format!("\t\t<Command name=\"{}\" id=\"{}\">\n", name, id));
            if let Some(title) = get_title(child_arr) {
                xml.push_str(&title);
            }
            if let Some(tooltip) = get_command_tooltip(child_arr) {
                xml.push_str(&tooltip);
            }
            if let Some(picture) = get_command_picture(child_arr) {
                xml.push_str(&picture);
            }
            let action = child_arr
                .get(8)
                .and_then(|v| v.as_str())
                .map(|s| s.trim_matches('"'))
                .unwrap_or(name);
            xml.push_str(&format!("\t\t\t<Action>{}</Action>\n", action));
            if let Some(rep) = get_command_representation(child_arr) {
                xml.push_str(&rep);
            }
            xml.push_str("\t\t</Command>\n");
        }
    }
    xml
}

fn parse_form_parameters(arr: &[Value]) -> String {
    let mut xml = String::new();
    let count = arr
        .get(1)
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(0);
    for idx in 2..2 + count {
        if idx >= arr.len() {
            break;
        }
        if let Some(child_arr) = arr.get(idx).and_then(|v| v.as_array()) {
            let name = child_arr
                .get(1)
                .and_then(|v| v.as_str())
                .map(|s| s.trim_matches('"'))
                .unwrap_or("Param");
            xml.push_str(&format!("\t\t<Parameter name=\"{}\">\n", name));
            if let Some(title) = get_title(child_arr) {
                xml.push_str(&title);
            }
            match get_type(child_arr) {
                Ok(typ) => xml.push_str(&typ),
                Err(err) => xml.push_str(&format!(
                    "\t\t\t<ERROR_UNKNOWN_TYPE>{}</ERROR_UNKNOWN_TYPE>\n",
                    err
                )),
            }
            xml.push_str("\t\t</Parameter>\n");
        }
    }
    xml
}

fn parse_form_events(items_arr: &[Value]) -> String {
    let mut events_xml = String::new();
    for v in items_arr {
        if let Some(arr) = v.as_array() {
            if let Some(count_str) = arr.first().and_then(|v| v.as_str()) {
                if let Ok(_count) = count_str.parse::<usize>() {
                    let mut event_names = Vec::new();
                    let mut i = 1;
                    while i + 1 < arr.len() {
                        if let (Some(uuid), Some(name)) = (
                            arr.get(i).and_then(|v| v.as_str()),
                            arr.get(i + 1).and_then(|v| v.as_str()),
                        ) {
                            if name.starts_with('"')
                                && name.ends_with('"')
                                && uuid.len() == 36
                                && uuid.contains('-')
                            {
                                let clean_name = name.trim_matches('"').to_string();
                                if !clean_name.is_empty() {
                                    event_names.push(clean_name);
                                }
                                i += 2;
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                    if !event_names.is_empty() {
                        events_xml.push_str("\t<Events>\n");
                        for name in &event_names {
                            events_xml.push_str(&format!(
                                "\t\t<Event name=\"{}\">{}</Event>\n",
                                name, name
                            ));
                        }
                        events_xml.push_str("\t</Events>\n");
                        break;
                    }
                }
            }
        }
    }
    events_xml
}

// ---------------------------------------------------------------------------
// Form item generation
// ---------------------------------------------------------------------------

/// Generate an Addition element (SearchStringAddition, ViewStatusAddition, SearchControlAddition).
///
/// `table_name` is the name of the parent Table element that owns this addition.
/// It is passed explicitly from the caller — we do NOT attempt to derive it from
/// the addition element's own name (that would be fragile string-hacking).
fn gen_addition(arr: &[Value], _tc: &str, indent: &str, table_name: &str) -> String {
    let disc = arr.get(5).and_then(|v| v.as_str()).unwrap_or("0");
    let (tag, source_type) = match disc {
        "0" => ("SearchStringAddition", "SearchStringRepresentation"),
        "1" => ("ViewStatusAddition", "ViewStatusRepresentation"),
        "2" => ("SearchControlAddition", "SearchControl"),
        _ => ("SearchStringAddition", "SearchStringRepresentation"),
    };
    let id = get_id(arr).unwrap_or_else(|| "0".to_string());
    let name = get_name_heuristic(arr).unwrap_or_else(|| format!("Item{}", id));
    let mut xml = format!("{}\t<{} name=\"{}\" id=\"{}\">\n", indent, tag, name, id);
    xml.push_str(&format!("{}\t\t<AdditionSource>\n", indent));
    xml.push_str(&format!("{}\t\t\t<Item>{}</Item>\n", indent, table_name));
    xml.push_str(&format!("{}\t\t\t<Type>{}</Type>\n", indent, source_type));
    xml.push_str(&format!("{}\t\t</AdditionSource>\n", indent));
    for v in arr {
        if let Some(child_arr) = v.as_array() {
            if child_arr.len() > 10 && is_valid_form_item_id(child_arr) {
                if is_context_menu(child_arr) {
                    let cid = get_id(child_arr).unwrap_or_else(|| "0".to_string());
                    let cname =
                        get_name_heuristic(child_arr).unwrap_or_else(|| format!("Item{}", cid));
                    xml.push_str(&format!(
                        "{}\t\t<ContextMenu name=\"{}\" id=\"{}\"/>\n",
                        indent, cname, cid
                    ));
                } else if is_extended_tooltip_child(child_arr) {
                    let cid = get_id(child_arr).unwrap_or_else(|| "0".to_string());
                    let cname =
                        get_name_heuristic(child_arr).unwrap_or_else(|| format!("Item{}", cid));
                    xml.push_str(&format!(
                        "{}\t\t<ExtendedTooltip name=\"{}\" id=\"{}\"/>\n",
                        indent, cname, cid
                    ));
                }
            }
        }
    }
    xml.push_str(&format!("{}</{}>\n", indent, tag));
    xml
}

fn gen_form_item(
    item_arr: &[Value],
    tc: &str,
    indent: &str,
    attr_map: &HashMap<usize, String>,
    uuid_map: &HashMap<String, String>,
) -> Result<String, String> {
    let child_indent = format!("{}\t", indent);

    let is_table = tc == "77" || tc == "73";

    let tag = match tc {
        "34" => "Button",
        "22" => {
            let (group_type, _) = get_group_type_and_name_idx(item_arr);
            match group_type {
                "3" => "Pages",
                "4" => "Page",
                "9" => "AutoCommandBar",
                _ => "UsualGroup",
            }
        }
        "77" | "73" => "Table",
        "48" => {
            let disc = get_discriminator(item_arr);
            match disc {
                "1" => "LabelField",
                "2" => "InputField",
                "3" => "CheckBoxField",
                "5" => "RadioButtonField",
                "6" => "SpreadSheetDocumentField",
                _ => "InputField",
            }
        }
        "12" => {
            let disc = get_discriminator(item_arr);
            match disc {
                "0" => "LabelDecoration",
                "1" => "PictureDecoration",
                _ => "LabelDecoration",
            }
        }
        "6" => {
            let disc = item_arr.get(5).and_then(|v| v.as_str()).unwrap_or("0");
            match disc {
                "0" => "SearchStringAddition",
                "1" => "ViewStatusAddition",
                "2" => "SearchControlAddition",
                _ => "SearchStringAddition",
            }
        }
        "46" => "SpreadSheetDocumentField",
        _ => return Err(format!("UNKNOWN_TC:{}", tc)),
    };

    let (id, name) = if tc == "22" {
        let (_, name_idx) = get_group_type_and_name_idx(item_arr);
        let id = get_id(item_arr).unwrap_or_else(|| "0".to_string());
        let name = item_arr
            .get(name_idx)
            .and_then(|v| v.as_str())
            .map(|s| s.trim_matches('"').to_string())
            .unwrap_or_else(|| format!("Group{}", id));
        (id, name)
    } else {
        let id = get_id(item_arr).unwrap_or_else(|| "0".to_string());
        let name = get_name_heuristic(item_arr).unwrap_or_else(|| format!("Item{}", id));
        (id, name)
    };

    let mut xml = String::new();

    if tc == "34" {
        let cmd_name = if name.starts_with("Form") && name.len() > 4 {
            name[4..].to_string()
        } else {
            name.clone()
        };
        xml.push_str(&format!(
            "{}<Button name=\"{}\" id=\"{}\">\n",
            child_indent, name, id
        ));
        xml.push_str(&format!(
            "{}\t<Type>CommandBarButton</Type>\n",
            child_indent
        ));
        xml.push_str(&format!(
            "{}\t<CommandName>Form.Command.{}</CommandName>\n",
            child_indent, cmd_name
        ));
    } else {
        xml.push_str(&format!(
            "{}<{} name=\"{}\" id=\"{}\">\n",
            child_indent, tag, name, id
        ));
    }

    // Emit DataPath for items that have it
    if tc == "48" || tc == "73" || tc == "77" {
        let dp_idx = if tc == "73" || tc == "77" {
            12
        } else {
            let has_type_data = value_as_str(item_arr.get(4).unwrap_or(&Value::Null)) == "1";
            if has_type_data {
                12
            } else {
                11
            }
        };
        if let Some(dp_arr) = item_arr.get(dp_idx).and_then(|v| v.as_array()) {
            if let Some(dp_str) = resolve_datapath(dp_arr, attr_map, uuid_map) {
                xml.push_str(&format!(
                    "{}\t<DataPath>{}</DataPath>\n",
                    child_indent, dp_str
                ));
            }
        }
    }

    // We no longer hardcode RadioButtonType or EditMode/ExtendedEditMultipleValues here,
    // because it causes discrepancies when the actual value differs from the hardcoded one.

    if let Some(title) = get_title(item_arr) {
        let t = title.replace("\n\t\t\t", &format!("\n{}\t", child_indent));
        xml.push_str(&t);
    }

    let mut tooltip_xml = String::new();
    let mut context_menu_xml = String::new();
    let mut children_xml = String::new();
    let mut autocommandbar_xml = String::new();
    let mut additions_xml = String::new();
    let mut group_tooltip_xml = String::new();

    for v in item_arr {
        if let Some(child_arr) = v.as_array() {
            if let Some(child_tc) = child_arr.first().and_then(|v| v.as_str()) {
                if child_arr.len() > 10 && is_valid_form_item_id(child_arr) {
                    if is_extended_tooltip_child(child_arr) {
                        let cid = get_id(child_arr).unwrap_or_else(|| "0".to_string());
                        let cname =
                            get_name_heuristic(child_arr).unwrap_or_else(|| format!("Item{}", cid));
                        tooltip_xml.push_str(&format!(
                            "{}\t<ExtendedTooltip name=\"{}\" id=\"{}\"/>\n",
                            child_indent, cname, cid
                        ));
                    } else if is_context_menu(child_arr) {
                        let cid = get_id(child_arr).unwrap_or_else(|| "0".to_string());
                        let cname =
                            get_name_heuristic(child_arr).unwrap_or_else(|| format!("Item{}", cid));
                        context_menu_xml.push_str(&format!(
                            "{}\t<ContextMenu name=\"{}\" id=\"{}\"/>\n",
                            child_indent, cname, cid
                        ));
                    } else if !is_selected_rows_actions_panel(child_arr)
                        && is_form_item_tc(child_tc)
                    {
                        let is_autocommandbar_child = child_tc == "22" && {
                            let (gt, _) = get_group_type_and_name_idx(child_arr);
                            gt == "9"
                        };
                        let is_addition_child = child_tc == "6";

                        if is_table && is_autocommandbar_child {
                            let cid = get_id(child_arr).unwrap_or_else(|| "0".to_string());
                            let cname = get_name_heuristic(child_arr)
                                .unwrap_or_else(|| format!("Item{}", cid));
                            autocommandbar_xml.push_str(&format!(
                                "{}\t<AutoCommandBar name=\"{}\" id=\"{}\"/>\n",
                                child_indent, cname, cid
                            ));
                        } else if is_table && is_addition_child {
                            // Pass the table's name explicitly — no name string hacking
                            additions_xml.push_str(&gen_addition(
                                child_arr,
                                child_tc,
                                &child_indent,
                                &name,
                            ));
                        } else if let Ok(child_item) =
                            gen_form_item(child_arr, child_tc, &child_indent, attr_map, uuid_map)
                        {
                            children_xml.push_str(&child_item);
                        }
                    }
                }
            }
        }
    }

    if tc == "22" {
        let (group_type, _) = get_group_type_and_name_idx(item_arr);
        if matches!(group_type, "3" | "4" | "5" | "6" | "7") {
            if let Some(tt) = get_tooltip(item_arr) {
                let t = tt.replace("\n\t\t\t", &format!("\n{}\t", child_indent));
                group_tooltip_xml.push_str(&t);
            }
        }
    }

    xml.push_str(&context_menu_xml);
    xml.push_str(&group_tooltip_xml);

    if tc == "12" {
        // Extract TextColor (index 15)
        if let Some(color_arr) = item_arr.get(15).and_then(|v| v.as_array()) {
            if color_arr.len() > 3 {
                // Hardcoding typical SpecialTextColor mapping for now based on structure
                xml.push_str(&format!(
                    "{}\t<TextColor>style:SpecialTextColor</TextColor>\n",
                    child_indent
                ));
            }
        }
        // Extract Font (index 16)
        if let Some(font_arr) = item_arr.get(16).and_then(|v| v.as_array()) {
            if font_arr.len() > 4 {
                let scale = font_arr.get(4).and_then(|v| v.as_str()).unwrap_or("100");
                xml.push_str(&format!(
                    "{}\t<Font ref=\"style:NormalTextFont\" kind=\"StyleItem\" scale=\"{}\"/>\n",
                    child_indent, scale
                ));
            }
        }
        // Extract Picture for PictureDecoration (discriminator "1")
        if get_discriminator(item_arr) == "1" {
            if let Some(pic_wrapper) = item_arr.get(19).and_then(|v| v.as_array()) {
                if let Some(pic_arr) = pic_wrapper.get(1).and_then(|v| v.as_array()) {
                    if let Some(uuid_arr) = pic_arr.get(2).and_then(|v| v.as_array()) {
                        if let Some(uuid) = uuid_arr.get(1).and_then(|v| v.as_str()) {
                            xml.push_str(&format!("{}\t<Picture>\n", child_indent));
                            if uuid == "47f01799-7968-4f44-9acc-fe1bdde8beb2" {
                                xml.push_str(&format!(
                                    "{}\t\t<xr:Ref>StdPicture.ActiveUsers</xr:Ref>\n",
                                    child_indent
                                ));
                            } else {
                                xml.push_str(&format!(
                                    "{}\t\t<xr:Ref>StdPicture.{}</xr:Ref>\n",
                                    child_indent, uuid
                                ));
                            }
                            xml.push_str(&format!(
                                "{}\t\t<xr:LoadTransparent>true</xr:LoadTransparent>\n",
                                child_indent
                            ));
                            xml.push_str(&format!("{}\t</Picture>\n", child_indent));
                        }
                    }
                }
            }
        }
    }

    if is_table {
        xml.push_str(&autocommandbar_xml);
    }

    xml.push_str(&tooltip_xml);

    if is_table && !additions_xml.is_empty() {
        xml.push_str(&additions_xml);
    }

    if !children_xml.is_empty() {
        xml.push_str(&format!("{}\t<ChildItems>\n", child_indent));
        xml.push_str(&children_xml);
        xml.push_str(&format!("{}</ChildItems>\n", child_indent));
    }

    xml.push_str(&format!("{}</{}>\n", child_indent, tag));
    Ok(xml)
}

fn parse_form_items_filtered(
    arr: &[Value],
    indent: &str,
    exclude_autocommandbar: bool,
    attr_map: &HashMap<usize, String>,
    uuid_map: &HashMap<String, String>,
) -> String {
    let mut xml = String::new();
    let child_indent = format!("{}\t", indent);

    let mut i = 0;
    while i < arr.len() {
        if let Some(item_arr) = arr[i].as_array() {
            if let Some(tc) = item_arr.first().and_then(|v| v.as_str()) {
                if is_form_item_tc(tc)
                    && item_arr.len() > 10
                    && is_valid_form_item_id(item_arr)
                    && !is_selected_rows_actions_panel(item_arr)
                {
                    if exclude_autocommandbar && is_top_level_autocommandbar(item_arr) {
                        i += 1;
                        continue;
                    }
                    match gen_form_item(item_arr, tc, &child_indent, attr_map, uuid_map) {
                        Ok(child_item) => xml.push_str(&child_item),
                        Err(err) => xml.push_str(&format!(
                            "{}<ERROR_UNKNOWN_ITEM>{}</ERROR_UNKNOWN_ITEM>\n",
                            child_indent, err
                        )),
                    }
                }
            }
        }
        i += 1;
    }
    xml
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Convert form layout bracket data to Form XML.
/// Returns `(xml_content, optional_module_code)`.
pub fn bracket_to_formlayout_xml(
    bracket: &str,
    _parent_name: &str,
    _parent_type: &str,
    uuid_map: Option<&HashMap<String, String>>,
) -> Option<(String, Option<String>)> {
    let json_val = crate::base::bracket_json::parse_bracket_to_json(bracket.as_bytes()).ok()?;
    let root = json_val.as_array()?;

    // Clone the provided uuid_map and add any UUIDs defined inside the form (e.g. Form Attribute columns)
    let mut local_uuid_map = if let Some(m) = uuid_map {
        m.clone()
    } else {
        HashMap::new()
    };

    fn collect_form_uuids(val: &Value, map: &mut HashMap<String, String>) {
        if let Some(arr) = val.as_array() {
            for i in 0..arr.len() {
                if i + 1 < arr.len() {
                    if let (Some(uuid_arr), Some(name_val)) =
                        (arr[i].as_array(), arr[i + 1].as_str())
                    {
                        if uuid_arr.len() == 2 && uuid_arr[0].as_str() == Some("0") {
                            if let Some(uuid) = uuid_arr[1].as_str() {
                                if name_val.starts_with('"') && name_val.ends_with('"') {
                                    let name = name_val.trim_matches('"');
                                    map.insert(uuid.to_string(), name.to_string());
                                }
                            }
                        }
                    }
                }
                collect_form_uuids(&arr[i], map);
            }
        }
    }

    if let Some(attrs_arr) = root.get(3) {
        collect_form_uuids(attrs_arr, &mut local_uuid_map);
    }

    let uuid_map = &local_uuid_map;

    // Build attr_id → name map from FormAttributes (index 3)
    let mut attr_map: HashMap<usize, String> = HashMap::new();
    let mut attrs_xml = String::new();
    if let Some(attrs_arr) = root.get(3).and_then(|v| v.as_array()) {
        attrs_xml = parse_form_attributes(attrs_arr);
        let count = attrs_arr
            .get(1)
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0);
        for idx in 2..2 + count {
            if let Some(attr_arr) = attrs_arr.get(idx).and_then(|v| v.as_array()) {
                let id = attr_arr
                    .get(1)
                    .and_then(|v| v.as_array())
                    .and_then(|a| a.first())
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(0);
                let name =
                    get_name_heuristic(attr_arr).unwrap_or_else(|| format!("Attribute{}", id));
                if id > 0 {
                    attr_map.insert(id, name);
                }
            }
        }
    }

    // Commands (index 5)
    let mut cmds_xml = String::new();
    if let Some(arr) = root.get(5).and_then(|v| v.as_array()) {
        cmds_xml = parse_form_commands(arr);
    }

    // Parameters (index 4)
    let mut params_xml = String::new();
    if let Some(arr) = root.get(4).and_then(|v| v.as_array()) {
        params_xml = parse_form_parameters(arr);
    }

    let mut autocommandbar_xml = String::new();
    let mut child_items_xml = String::new();
    let mut events_xml = String::new();
    if let Some(items_arr) = root.get(1).and_then(|v| v.as_array()) {
        // Find the form-level AutoCommandBar (id="-1") and render it separately
        for v in items_arr {
            if let Some(item_arr) = v.as_array() {
                if let Some(tc) = item_arr.first().and_then(|v| v.as_str()) {
                    if tc == "22"
                        && is_valid_form_item_id(item_arr)
                        && is_top_level_autocommandbar(item_arr)
                    {
                        let id = get_id(item_arr).unwrap_or_default();
                        if id == "-1" {
                            if let Ok(item) = gen_form_item(item_arr, tc, "\t", &attr_map, uuid_map)
                            {
                                autocommandbar_xml = item;
                            }
                        }
                    }
                }
            }
        }

        child_items_xml = parse_form_items_filtered(items_arr, "\t\t", true, &attr_map, uuid_map);
        events_xml = parse_form_events(items_arr);
    }

    // Module (index 2)
    let mut module_content = None;
    if let Some(mod_str) = root.get(2).and_then(|v| v.as_str()) {
        let unquoted = mod_str.trim_matches('"').replace("\"\"", "\"");
        if has_real_content(unquoted.as_bytes()) {
            module_content = Some(unquoted);
        }
    }

    let mut xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<Form xmlns="http://v8.1c.ru/8.3/xcf/logform" xmlns:app="http://v8.1c.ru/8.2/managed-application/core" xmlns:cfg="http://v8.1c.ru/8.1/data/enterprise/current-config" xmlns:dcscor="http://v8.1c.ru/8.1/data-composition-system/core" xmlns:dcssch="http://v8.1c.ru/8.1/data-composition-system/schema" xmlns:dcsset="http://v8.1c.ru/8.1/data-composition-system/settings" xmlns:ent="http://v8.1c.ru/8.1/data/enterprise" xmlns:lf="http://v8.1c.ru/8.2/managed-application/logform" xmlns:pal="http://v8.1c.ru/8.1/data/ui/colors/palette" xmlns:style="http://v8.1c.ru/8.1/data/ui/style" xmlns:sys="http://v8.1c.ru/8.1/data/ui/fonts/system" xmlns:v8="http://v8.1c.ru/8.1/data/core" xmlns:v8ui="http://v8.1c.ru/8.1/data/ui" xmlns:web="http://v8.1c.ru/8.1/data/ui/colors/web" xmlns:win="http://v8.1c.ru/8.1/data/ui/colors/windows" xmlns:xr="http://v8.1c.ru/8.3/xcf/readable" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" version="2.21">
"#
    .to_string();

    if !autocommandbar_xml.is_empty() {
        xml.push_str(&autocommandbar_xml);
    }

    if !events_xml.is_empty() {
        xml.push_str(&events_xml);
    }

    if !child_items_xml.is_empty() {
        xml.push_str("\t<ChildItems>\n");
        xml.push_str(&child_items_xml);
        xml.push_str("\t</ChildItems>");
    }

    if !attrs_xml.is_empty() {
        xml.push_str("\n\t<Attributes>\n");
        xml.push_str(&attrs_xml);
        xml.push_str("\t</Attributes>");
    }

    if !cmds_xml.is_empty() {
        xml.push_str("\n\t<Commands>\n");
        xml.push_str(&cmds_xml);
        xml.push_str("\t</Commands>");
    }

    if !params_xml.is_empty() {
        xml.push_str("\n\t<Parameters>\n");
        xml.push_str(&params_xml);
        xml.push_str("\t</Parameters>");
    }

    xml.push_str("\n</Form>");

    Some((xml, module_content))
}

/// Public wrapper for `get_group_type_and_name_idx` (used in tests).
pub fn get_group_type_and_name_idx_pub(item_arr: &[Value]) -> (&str, usize) {
    get_group_type_and_name_idx(item_arr)
}
