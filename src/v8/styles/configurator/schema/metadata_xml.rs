//! XML generation for 1C metadata objects (EPF, ERF, CF, Language, Role, etc.)
//!
//! All synonym handling is language-agnostic — we read whatever language pairs
//! are stored in the bracket data instead of hardcoding "ru"/"en".

use super::synonyms::{
    extract_all_synonyms, extract_tc27_synonyms, extract_tc27_synonyms_from_sub,
    render_synonym_items,
};
use crate::base::bracket_json::{parse_bracket_to_json, serialize_json_to_bracket};
use crate::v8::uuids;
use serde_json::Value;
use std::collections::HashMap;

/// Reference to a child object for the ChildObjects section in root XML.
pub struct ChildObjectRef {
    /// XML element type: "Form", "Template", etc.
    pub element_type: String,
    /// Name of the child object.
    pub name: String,
    /// If set, use this raw XML instead of generating a simple `<Type>Name</Type>` element.
    pub raw_xml: Option<String>,
}

/// Known type UUIDs for subordinate groups in EPF headers.
pub const EPF_FORM_ATTRIBUTES_UUID: &str = "ec6bb5e5-b7a8-4d75-bec9-658107a699cf";
pub const EPF_TABULAR_SECTIONS_UUID: &str = "2bcef0d1-0981-11d6-b9b8-0050bae0a95d";

// ---------------------------------------------------------------------------
// Common XML header (shared namespace declarations)
// ---------------------------------------------------------------------------

const METADATA_XML_HEADER: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<MetaDataObject xmlns="http://v8.1c.ru/8.3/MDClasses" xmlns:app="http://v8.1c.ru/8.2/managed-application/core" xmlns:cfg="http://v8.1c.ru/8.1/data/enterprise/current-config" xmlns:cmi="http://v8.1c.ru/8.2/managed-application/cmi" xmlns:ent="http://v8.1c.ru/8.1/data/enterprise" xmlns:lf="http://v8.1c.ru/8.2/managed-application/logform" xmlns:pal="http://v8.1c.ru/8.1/data/ui/colors/palette" xmlns:style="http://v8.1c.ru/8.1/data/ui/style" xmlns:sys="http://v8.1c.ru/8.1/data/ui/fonts/system" xmlns:v8="http://v8.1c.ru/8.1/data/core" xmlns:v8ui="http://v8.1c.ru/8.1/data/ui" xmlns:web="http://v8.1c.ru/8.1/data/ui/colors/web" xmlns:win="http://v8.1c.ru/8.1/data/ui/colors/windows" xmlns:xen="http://v8.1c.ru/8.3/xcf/enums" xmlns:xpr="http://v8.1c.ru/8.3/xcf/predef" xmlns:xr="http://v8.1c.ru/8.3/xcf/readable" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" version="2.21">"#;

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Extract a string value from a JSON Value, trimming quotes.
pub fn json_str(v: &Value) -> Option<String> {
    v.as_str().map(|s| s.trim_matches('"').to_string())
}

/// Extract a quoted string value, stripping surrounding quotes.
pub fn unquote(val: &str) -> &str {
    val.trim_matches('"')
}

/// Recursively search JSON array for a UUID-format string.
pub fn find_uuid_in_json(val: &Value) -> Option<String> {
    match val {
        Value::String(s) => {
            let clean = s.trim_matches('"');
            if clean.len() == 36 && clean.chars().filter(|&c| c == '-').count() == 4 {
                let parts: Vec<&str> = clean.split('-').collect();
                if parts.len() == 5
                    && parts[0].len() == 8
                    && parts[1].len() == 4
                    && parts[2].len() == 4
                    && parts[3].len() == 4
                    && parts[4].len() == 12
                    && parts
                        .iter()
                        .all(|p| p.chars().all(|c| c.is_ascii_hexdigit()))
                {
                    return Some(clean.to_string());
                }
            }
            None
        }
        Value::Array(arr) => {
            for item in arr {
                if let Some(uuid) = find_uuid_in_json(item) {
                    return Some(uuid);
                }
            }
            None
        }
        _ => None,
    }
}

/// Extract the UUID from a tc=27 attribute definition's property sub-array.
/// Path: `item_arr[1][1][1][2]`
pub fn extract_tc27_uuid(item_arr: &[Value]) -> Option<String> {
    let props = item_arr.get(1)?.as_array()?;
    let sub = props.get(1)?.as_array()?;
    let uuid_arr = sub.get(1)?.as_array()?;
    json_str(uuid_arr.get(2)?)
}

/// Extract the name from a tc=27 attribute definition.
/// Path: `item_arr[1][1][2]`
pub fn extract_tc27_name(item_arr: &[Value]) -> Option<String> {
    let props = item_arr.get(1)?.as_array()?;
    let sub = props.get(1)?.as_array()?;
    json_str(sub.get(2)?)
}

/// Extract the comment from a tc=27 attribute definition.
/// Path: `item_arr[1][1][4]`
pub fn extract_tc27_comment(item_arr: &[Value]) -> String {
    item_arr
        .get(1)
        .and_then(|v| v.as_array())
        .and_then(|a| a.get(1))
        .and_then(|v| v.as_array())
        .and_then(|a| a.get(4))
        .and_then(|v| v.as_str())
        .map(|s| s.trim_matches('"').to_string())
        .unwrap_or_default()
}

/// Map a tc=27 type pattern to the v8 XML type representation.
pub fn tc27_type_to_xml(type_arr: &[Value]) -> String {
    let pattern = type_arr
        .first()
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim_matches('"');

    match pattern {
        "S" => {
            let len = type_arr.get(1).and_then(|v| v.as_str()).unwrap_or("0");
            let allowed = type_arr.get(2).and_then(|v| v.as_str());
            let allowed_str = match allowed {
                Some("0") => "Fixed",
                Some("1") => "Variable",
                None => "Variable",
                _ => "Variable",
            };
            format!(
                "<v8:Type>xs:string</v8:Type>\n\t\t\t\t\t<v8:StringQualifiers>\n\t\t\t\t\t\t<v8:Length>{}</v8:Length>\n\t\t\t\t\t\t<v8:AllowedLength>{}</v8:AllowedLength>\n\t\t\t\t\t</v8:StringQualifiers>",
                len, allowed_str
            )
        }
        "N" => {
            let digits = type_arr.get(1).and_then(|v| v.as_str()).unwrap_or("0");
            let fraction = type_arr.get(2).and_then(|v| v.as_str()).unwrap_or("0");
            let non_negative = type_arr.get(3).and_then(|v| v.as_str()).unwrap_or("0");
            let nn_str = if non_negative == "1" { "true" } else { "false" };
            format!(
                "<v8:Type>xs:decimal</v8:Type>\n\t\t\t\t\t<v8:NumberQualifiers>\n\t\t\t\t\t\t<v8:Digits>{}</v8:Digits>\n\t\t\t\t\t\t<v8:FractionDigits>{}</v8:FractionDigits>\n\t\t\t\t\t\t<v8:NonNegative>{}</v8:NonNegative>\n\t\t\t\t\t</v8:NumberQualifiers>",
                digits, fraction, nn_str
            )
        }
        "B" => "<v8:Type>xs:boolean</v8:Type>".to_string(),
        "D" => {
            let date_fracs = type_arr.get(1).and_then(|v| v.as_str());
            let df_str = match date_fracs {
                Some("1") => "Date",
                Some("2") => "Time",
                Some("3") => "DateTime",
                // By default, if not specified (or 0), 1C uses DateTime
                _ => "DateTime",
            };
            format!(
                "<v8:Type>xs:dateTime</v8:Type>\n\t\t\t\t\t<v8:DateQualifiers>\n\t\t\t\t\t\t<v8:DateFractions>{}</v8:DateFractions>\n\t\t\t\t\t</v8:DateQualifiers>",
                df_str
            )
        }
        "#" => {
            let type_uuid = type_arr
                .get(1)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim_matches('"');
            match type_uuid {
                "2fdc88ec-7c9b-43cd-8ba5-873f043bdd88" => {
                    "<v8:Type>v8:StandardPeriod</v8:Type>".to_string()
                }
                "e603103e-a318-4edc-a014-b1c6cf94d49f" => {
                    "<v8:Type xmlns:mxl=\"http://v8.1c.ru/8.2/data/spreadsheet\">mxl:SpreadsheetDocument</v8:Type>".to_string()
                }
                _ => format!(
                    "<v8:Type>{}Ref.{}</v8:Type>",
                    "",
                    uuids::metadata_type_name(type_uuid).unwrap_or("Unknown")
                ),
            }
        }
        _ => "<v8:Type>xs:string</v8:Type>".to_string(),
    }
}

/// Generate full `<Attribute>` XML from a tc=27 bracket data array.
pub fn gen_attribute_xml(tc27_arr: &[Value], is_ts_child: bool) -> Option<String> {
    let uuid = extract_tc27_uuid(tc27_arr)?;
    let name = extract_tc27_name(tc27_arr)?;
    let synonyms = extract_tc27_synonyms(tc27_arr);
    let comment = extract_tc27_comment(tc27_arr);

    let type_arr = tc27_arr
        .get(1)
        .and_then(|v| v.as_array())
        .and_then(|a| a.get(2))
        .and_then(|v| v.as_array())?;

    let type_inner = if type_arr
        .first()
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim_matches('"')
        == "Pattern"
    {
        type_arr.get(1).and_then(|v| v.as_array())?
    } else {
        type_arr
    };

    let type_xml = tc27_type_to_xml(type_inner);

    let syn_xml = render_synonym_items(&synonyms, "\t\t\t\t\t\t");

    let comment_xml = if comment.is_empty() {
        "\t\t\t\t\t<Comment/>".to_string()
    } else {
        format!("\t\t\t\t\t<Comment>{}</Comment>", comment)
    };

    let type_pattern = type_inner
        .first()
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim_matches('"');
    let fill_val = match type_pattern {
        "S" => "<FillValue xsi:type=\"xs:string\"/>".to_string(),
        "N" => "<FillValue xsi:type=\"xs:decimal\">0</FillValue>".to_string(),
        "D" => "<FillValue xsi:type=\"xs:dateTime\">0001-01-01T00:00:00</FillValue>".to_string(),
        "B" => "<FillValue xsi:type=\"xs:boolean\">false</FillValue>".to_string(),
        "#" => {
            let uuid = type_inner
                .get(1)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim_matches('"');
            if uuid == "e603103e-a318-4edc-a014-b1c6cf94d49f" {
                "<FillValue xmlns:mxl=\"http://v8.1c.ru/8.2/data/spreadsheet\" xsi:type=\"mxl:SpreadsheetDocument\"/>".to_string()
            } else if uuid == "2fdc88ec-7c9b-43cd-8ba5-873f043bdd88" {
                "<FillValue xsi:type=\"v8:StandardPeriod\"/>".to_string()
            } else {
                "<FillValue xsi:nil=\"true\"/>".to_string()
            }
        }
        _ => "<FillValue xsi:nil=\"true\"/>".to_string(),
    };

    let fill_prefix = if is_ts_child {
        format!(
            "\t\t\t\t\t<FillFromFillingValue>false</FillFromFillingValue>\n\t\t\t\t\t{}\n",
            fill_val
        )
    } else {
        "".to_string()
    };

    let xml = format!(
        "\t\t\t<Attribute uuid=\"{uuid}\">\n\t\t\t\t<Properties>\n\t\t\t\t\t<Name>{name}</Name>\n\t\t\t\t\t<Synonym>\n{syn_xml}\t\t\t\t\t</Synonym>\n{comment_xml}\n\t\t\t\t\t<Type>\n\t\t\t\t\t\t{type_xml}\n\t\t\t\t\t</Type>\n\t\t\t\t\t<PasswordMode>false</PasswordMode>\n\t\t\t\t\t<Format/>\n\t\t\t\t\t<EditFormat/>\n\t\t\t\t\t<ToolTip/>\n\t\t\t\t\t<MarkNegatives>false</MarkNegatives>\n\t\t\t\t\t<Mask/>\n\t\t\t\t\t<MultiLine>false</MultiLine>\n\t\t\t\t\t<ExtendedEdit>false</ExtendedEdit>\n\t\t\t\t\t<MinValue xsi:nil=\"true\"/>\n\t\t\t\t\t<MaxValue xsi:nil=\"true\"/>\n{fill_prefix}\t\t\t\t\t<FillChecking>DontCheck</FillChecking>\n\t\t\t\t\t<ChoiceFoldersAndItems>Items</ChoiceFoldersAndItems>\n\t\t\t\t\t<ChoiceParameterLinks/>\n\t\t\t\t\t<ChoiceParameters/>\n\t\t\t\t\t<QuickChoice>Auto</QuickChoice>\n\t\t\t\t\t<CreateOnInput>Auto</CreateOnInput>\n\t\t\t\t\t<ChoiceForm/>\n\t\t\t\t\t<LinkByType/>\n\t\t\t\t\t<ChoiceHistoryOnInput>Auto</ChoiceHistoryOnInput>\n\t\t\t\t</Properties>\n\t\t\t</Attribute>",
        uuid = uuid,
        name = name,
        syn_xml = syn_xml,
        comment_xml = comment_xml,
        type_xml = type_xml,
        fill_prefix = fill_prefix,
    );

    Some(xml)
}

/// Extract the name from a tabular section definition wrapper.
pub fn extract_tabular_section_name(inst: &[Value]) -> Option<String> {
    let inner_arr = inst.first()?.as_array()?;
    let tc_data = inner_arr.get(1)?.as_array()?;
    extract_tc27_name(tc_data)
}

/// Generate `<TabularSection>` XML from an inline definition wrapper.
pub fn gen_tabular_section_xml(
    inst: &[Value],
    _class_name: &str,
    obj_name: &str,
) -> Option<String> {
    let inner_arr = inst.first()?.as_array()?;
    let tc_data = inner_arr.get(1)?.as_array()?;
    let tc = tc_data.first()?.as_str()?;
    if tc != "11" {
        return None;
    }

    let ts_type_id = tc_data.get(1)?.as_str()?;
    let ts_value_id = tc_data.get(2)?.as_str()?;
    let ts_row_type_id = tc_data.get(3)?.as_str()?;
    let ts_row_value_id = tc_data.get(4)?.as_str()?;

    let definition = tc_data.get(5)?.as_array()?;
    let props = definition.get(1)?.as_array()?;
    let uuid_data = props.get(1)?.as_array()?;
    let uuid = uuid_data.get(2)?.as_str()?.trim_matches('"');
    let name = props.get(2)?.as_str()?.trim_matches('"').to_string();

    let synonyms = extract_tc27_synonyms_from_sub(props);
    let comment = props
        .get(4)
        .and_then(|v| v.as_str())
        .unwrap_or("\"\"")
        .trim_matches('"')
        .to_string();

    let syn_xml = render_synonym_items(&synonyms, "\t\t\t\t\t\t");

    let comment_xml = if comment.is_empty() {
        "\t\t\t\t\t<Comment/>".to_string()
    } else {
        format!("\t\t\t\t\t<Comment>{}</Comment>", comment)
    };

    // Extract child attributes from inst[2]
    let mut child_attrs = Vec::new();
    if let Some(child_group) = inst.get(2).and_then(|v| v.as_array()) {
        for i in 2..child_group.len() {
            if let Some(attr_inst) = child_group.get(i).and_then(|v| v.as_array()) {
                if let Some(attr_inner) = attr_inst.first().and_then(|v| v.as_array()) {
                    if let Some(attr_tc27) = attr_inner.get(1).and_then(|v| v.as_array()) {
                        let attr_tc = attr_tc27.first().and_then(|v| v.as_str()).unwrap_or("");
                        if attr_tc == "27" {
                            if let Some(attr_xml) = gen_attribute_xml(attr_tc27, true) {
                                child_attrs.push(attr_xml);
                            }
                        }
                    }
                }
            }
        }
    }

    let children_xml = if child_attrs.is_empty() {
        String::new()
    } else {
        let mut xml = "\t\t\t\t<ChildObjects>\n".to_string();
        for attr in &child_attrs {
            xml.push_str(attr);
            xml.push('\n');
        }
        xml.push_str("\t\t\t\t</ChildObjects>");
        xml
    };

    let standard_attributes = "\t\t\t\t\t<StandardAttributes>\n\t\t\t\t\t\t<xr:StandardAttribute name=\"LineNumber\">\n\t\t\t\t\t\t\t<xr:LinkByType/>\n\t\t\t\t\t\t\t<xr:FillChecking>DontCheck</xr:FillChecking>\n\t\t\t\t\t\t\t<xr:MultiLine>false</xr:MultiLine>\n\t\t\t\t\t\t\t<xr:FillFromFillingValue>false</xr:FillFromFillingValue>\n\t\t\t\t\t\t\t<xr:CreateOnInput>Auto</xr:CreateOnInput>\n\t\t\t\t\t\t\t<xr:TypeReductionMode>TransformValues</xr:TypeReductionMode>\n\t\t\t\t\t\t\t<xr:MaxValue xsi:nil=\"true\"/>\n\t\t\t\t\t\t\t<xr:ToolTip/>\n\t\t\t\t\t\t\t<xr:ExtendedEdit>false</xr:ExtendedEdit>\n\t\t\t\t\t\t\t<xr:Format/>\n\t\t\t\t\t\t\t<xr:ChoiceForm/>\n\t\t\t\t\t\t\t<xr:QuickChoice>Auto</xr:QuickChoice>\n\t\t\t\t\t\t\t<xr:ChoiceHistoryOnInput>Auto</xr:ChoiceHistoryOnInput>\n\t\t\t\t\t\t\t<xr:EditFormat/>\n\t\t\t\t\t\t\t<xr:PasswordMode>false</xr:PasswordMode>\n\t\t\t\t\t\t\t<xr:DataHistory>Use</xr:DataHistory>\n\t\t\t\t\t\t\t<xr:MarkNegatives>false</xr:MarkNegatives>\n\t\t\t\t\t\t\t<xr:MinValue xsi:nil=\"true\"/>\n\t\t\t\t\t\t\t<xr:Synonym/>\n\t\t\t\t\t\t\t<xr:Comment/>\n\t\t\t\t\t\t\t<xr:FullTextSearch>Use</xr:FullTextSearch>\n\t\t\t\t\t\t\t<xr:ChoiceParameterLinks/>\n\t\t\t\t\t\t\t<xr:FillValue xsi:nil=\"true\"/>\n\t\t\t\t\t\t\t<xr:Mask/>\n\t\t\t\t\t\t\t<xr:ChoiceParameters/>\n\t\t\t\t\t\t</xr:StandardAttribute>\n\t\t\t\t\t</StandardAttributes>";

    let ts_prefix = "DataProcessorTabularSection";
    let ts_row_prefix = "DataProcessorTabularSectionRow";
    let ts_generated_name = format!("{}.{}.{}", ts_prefix, obj_name, name);
    let ts_row_generated_name = format!("{}.{}.{}", ts_row_prefix, obj_name, name);

    let xml = format!(
        "\t\t\t<TabularSection uuid=\"{uuid}\">\n\t\t\t\t<InternalInfo>\n\t\t\t\t\t<xr:GeneratedType name=\"{ts_generated_name}\" category=\"TabularSection\">\n\t\t\t\t\t\t<xr:TypeId>{ts_type_id}</xr:TypeId>\n\t\t\t\t\t\t<xr:ValueId>{ts_value_id}</xr:ValueId>\n\t\t\t\t\t</xr:GeneratedType>\n\t\t\t\t\t<xr:GeneratedType name=\"{ts_row_generated_name}\" category=\"TabularSectionRow\">\n\t\t\t\t\t\t<xr:TypeId>{ts_row_type_id}</xr:TypeId>\n\t\t\t\t\t\t<xr:ValueId>{ts_row_value_id}</xr:ValueId>\n\t\t\t\t\t</xr:GeneratedType>\n\t\t\t\t</InternalInfo>\n\t\t\t\t<Properties>\n\t\t\t\t\t<Name>{name}</Name>\n\t\t\t\t\t<Synonym>\n{syn_xml}\t\t\t\t\t</Synonym>\n{comment_xml}\n\t\t\t\t\t<ToolTip/>\n\t\t\t\t\t<FillChecking>DontCheck</FillChecking>\n{standard_attributes}\n\t\t\t\t</Properties>\n{children_xml}\n\t\t\t</TabularSection>",
        uuid = uuid,
        name = name,
        syn_xml = syn_xml,
        comment_xml = comment_xml,
        ts_type_id = ts_type_id,
        ts_value_id = ts_value_id,
        ts_row_type_id = ts_row_type_id,
        ts_row_value_id = ts_row_value_id,
        children_xml = children_xml,
        standard_attributes = standard_attributes,
        ts_generated_name = ts_generated_name,
        ts_row_generated_name = ts_row_generated_name,
    );

    Some(xml)
}

/// Extract child objects (Attributes, TabularSections) from EPF header bracket data.
pub fn extract_epf_child_objects(
    json_val: &Value,
    class_name: &str,
    obj_name: &str,
) -> Vec<ChildObjectRef> {
    let root = match json_val.as_array() {
        Some(r) => r,
        None => return vec![],
    };
    let inner = match root.get(3).and_then(|v| v.as_array()) {
        Some(a) => a,
        None => return vec![],
    };
    let data = match inner.get(1).and_then(|v| v.as_array()) {
        Some(a) => a,
        None => return vec![],
    };

    let mut children = Vec::new();

    let count_str = match data.get(2).and_then(|v| v.as_str()) {
        Some(c) => c,
        None => return children,
    };
    let _count: usize = count_str.parse().unwrap_or(0);

    for idx in 3..data.len() {
        let group = match data.get(idx).and_then(|v| v.as_array()) {
            Some(g) => g,
            None => continue,
        };
        let type_uuid = match group.first().and_then(|v| v.as_str()) {
            Some(u) => u,
            None => continue,
        };
        match type_uuid {
            EPF_FORM_ATTRIBUTES_UUID => {
                for i in 2..group.len() {
                    if let Some(inst) = group.get(i).and_then(|v| v.as_array()) {
                        if let Some(inner_arr) = inst.first().and_then(|v| v.as_array()) {
                            if let Some(tc27) = inner_arr.get(1).and_then(|v| v.as_array()) {
                                let tc = tc27.first().and_then(|v| v.as_str()).unwrap_or("");
                                if tc == "27" {
                                    if let Some(xml) = gen_attribute_xml(tc27, false) {
                                        let name = extract_tc27_name(tc27)
                                            .unwrap_or_else(|| "Attribute".to_string());
                                        children.push(ChildObjectRef {
                                            element_type: "Attribute".to_string(),
                                            name,
                                            raw_xml: Some(xml),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
            EPF_TABULAR_SECTIONS_UUID => {
                for i in 2..group.len() {
                    if let Some(inst) = group.get(i).and_then(|v| v.as_array()) {
                        if let Some(xml) = gen_tabular_section_xml(inst, class_name, obj_name) {
                            let ts_name = extract_tabular_section_name(inst)
                                .unwrap_or_else(|| "TabularSection".to_string());
                            children.push(ChildObjectRef {
                                element_type: "TabularSection".to_string(),
                                name: ts_name,
                                raw_xml: Some(xml),
                            });
                        }
                    }
                }
            }
            _ => {}
        }
    }

    children
}

fn collect_uuid_names(val: &Value, map: &mut HashMap<String, String>) {
    if let Some(arr) = val.as_array() {
        if let Some(tc) = arr.first().and_then(|v| v.as_str()) {
            if tc == "27" {
                if let Some(uuid) = extract_tc27_uuid(arr) {
                    let name = extract_tc27_name(arr).unwrap_or_else(|| "Attribute".to_string());
                    if !uuid.is_empty() {
                        map.insert(uuid, name);
                    }
                }
            } else if tc == "11" {
                if let Some(definition) = arr.get(5).and_then(|v| v.as_array()) {
                    if let Some(props) = definition.get(1).and_then(|v| v.as_array()) {
                        if let Some(uuid_data) = props.get(1).and_then(|v| v.as_array()) {
                            if let Some(uuid) = uuid_data.get(2).and_then(|v| v.as_str()) {
                                let uuid = uuid.trim_matches('"').to_string();
                                let name = props
                                    .get(2)
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.trim_matches('"').to_string())
                                    .unwrap_or_else(|| "TabularSection".to_string());
                                if !uuid.is_empty() {
                                    map.insert(uuid, name);
                                }
                            }
                        }
                    }
                }
            }
        }
        for child in arr {
            collect_uuid_names(child, map);
        }
    }
}

/// Extract a UUID→name map from EPF header bracket data.
pub fn extract_epf_uuid_name_map(json_val: &Value) -> HashMap<String, String> {
    let mut map = HashMap::new();
    collect_uuid_names(json_val, &mut map);
    map
}

// ---------------------------------------------------------------------------
// XML generators for metadata objects
// ---------------------------------------------------------------------------

/// Generate XML for an ExternalDataProcessor or ExternalReport object.
pub fn generate_ext_object(
    json_val: &Value,
    class_name: &str,
    children: &[ChildObjectRef],
    default_form: Option<&str>,
) -> Option<String> {
    let root = json_val.as_array()?;
    let uuid = root.get(1)?.as_array()?.first()?.as_str()?;

    let inner = root.get(3)?.as_array()?;
    let class_id = inner.first()?.as_str()?;

    let data = inner.get(1)?.as_array()?;
    let type_data = data.get(1)?.as_array()?;
    let type_id = type_data.get(1)?.as_str()?;
    let value_id = type_data.get(2)?.as_str()?;

    let props = type_data.get(3)?.as_array()?.get(1)?.as_array()?;

    let object_id = props.get(1)?.as_array()?.get(2)?.as_str()?;
    let name_raw = props.get(2)?.as_str()?;
    let name = name_raw.trim_matches('"');

    // Extract ALL synonym pairs — language-agnostic
    let synonyms = extract_all_synonyms(json_val);

    let comment_raw = props.get(4).and_then(|v| v.as_str()).unwrap_or("\"\"");
    let comment = comment_raw.trim_matches('"');

    // Build ChildObjects section
    let child_objects = if children.is_empty() {
        "\t\t<ChildObjects/>".to_string()
    } else {
        let mut attrs = Vec::new();
        let mut tss = Vec::new();
        let mut others = Vec::new();
        for child in children {
            match child.element_type.as_str() {
                "Attribute" => attrs.push(child),
                "TabularSection" => tss.push(child),
                _ => others.push(child),
            }
        }
        let mut lines = vec!["\t\t<ChildObjects>".to_string()];
        for child in attrs.into_iter().chain(tss).chain(others) {
            if let Some(ref raw) = child.raw_xml {
                lines.push(raw.clone());
            } else {
                lines.push(format!(
                    "\t\t\t<{0}>{1}</{0}>",
                    child.element_type, child.name
                ));
            }
        }
        lines.push("\t\t</ChildObjects>".to_string());
        lines.join("\n")
    };

    let extra_properties = if class_name == "ExternalReport" {
        "\n\t\t\t<MainDataCompositionSchema/>\n\t\t\t<DefaultSettingsForm/>\n\t\t\t<AuxiliarySettingsForm/>\n\t\t\t<DefaultVariantForm/>\n\t\t\t<AuxiliaryVariantForm/>\n\t\t\t<VariantsStorage/>\n\t\t\t<SettingsStorage/>"
    } else {
        ""
    };

    let comment_xml = if comment.is_empty() {
        "\t\t\t<Comment/>".to_string()
    } else {
        format!("\t\t\t<Comment>{}</Comment>", comment)
    };

    // Render synonym XML from actual data
    let syn_items = render_synonym_items(&synonyms, "\t\t\t\t");
    let synonym_xml = if syn_items.is_empty() {
        "\t\t\t<Synonym/>".to_string()
    } else {
        format!("\t\t\t<Synonym>\n{}\t\t\t</Synonym>", syn_items)
    };

    let xml = format!(
        r#"{header}
	<{class_name} uuid="{uuid}">
		<InternalInfo>
			<xr:ContainedObject>
				<xr:ClassId>{class_id}</xr:ClassId>
				<xr:ObjectId>{object_id}</xr:ObjectId>
			</xr:ContainedObject>
			<xr:GeneratedType name="{class_name}Object.{name}" category="Object">
				<xr:TypeId>{type_id}</xr:TypeId>
				<xr:ValueId>{value_id}</xr:ValueId>
			</xr:GeneratedType>
		</InternalInfo>
		<Properties>
			<Name>{name}</Name>
{synonym_xml}
{comment_xml}
			<DefaultForm>{default_form_val}</DefaultForm>
			<AuxiliaryForm/>{extra_properties}
		</Properties>
{child_objects}
	</{class_name}>
</MetaDataObject>"#,
        header = METADATA_XML_HEADER,
        class_name = class_name,
        uuid = uuid,
        class_id = class_id,
        object_id = object_id,
        name = name,
        type_id = type_id,
        value_id = value_id,
        synonym_xml = synonym_xml,
        comment_xml = comment_xml,
        default_form_val = default_form.unwrap_or(""),
        child_objects = child_objects,
        extra_properties = extra_properties,
    );

    Some(xml)
}

/// Generate XML for a Configuration (CF/CFE root object).
pub fn generate_cfg_object(json_val: &Value) -> Option<String> {
    let root = json_val.as_array()?;
    let uuid = root.get(1)?.as_array()?.first()?.as_str()?;

    let props_html = if let Some(props_branch) = root
        .get(3)
        .and_then(|v| v.as_array())
        .and_then(|a| a.get(1))
        .and_then(|v| v.as_array())
    {
        let name = props_branch
            .get(2)
            .and_then(|v| v.as_str())
            .map(|s| unquote(s).to_string())
            .unwrap_or_default();
        let synonyms = extract_all_synonyms(json_val);
        let syn_items = render_synonym_items(&synonyms, "\t\t\t\t");
        let synonym_xml = if syn_items.is_empty() {
            "\t\t\t<Synonym/>".to_string()
        } else {
            format!("\t\t\t<Synonym>\n{}\t\t\t</Synonym>", syn_items)
        };

        format!(
            "\t\t<Properties>\n\t\t\t<Name>{name}</Name>\n{synonym_xml}\n\t\t\t<Comment/>\n\t\t</Properties>",
            name = name,
            synonym_xml = synonym_xml,
        )
    } else {
        String::new()
    };

    Some(format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<MetaDataObject xmlns="http://v8.1c.ru/8.3/MDClasses" version="2.21">
	<Configuration uuid="{uuid}">
{props}
	</Configuration>
</MetaDataObject>"#,
        uuid = uuid,
        props = props_html
    ))
}

/// Generate XML for a generic metadata object (Form, Language, Role, Template, etc.).
pub fn generate_generic_metadata(
    json_val: &Value,
    element_name: &str,
    group_name: &str,
    override_name: Option<&str>,
) -> Option<String> {
    // Try to extract props from the 3-element subordinate header format:
    // root = ["1", ["1", ["0", [tc, [props_branch]]]], "0"]
    // root[1][1][1][1] = props_branch = [marker, uuid_data, name, synonyms, ...]
    let props_from_3elem = json_val.as_array().and_then(|root| {
        if root.len() == 3 {
            root.get(1)
                .and_then(|v| v.as_array())
                .and_then(|a| a.get(1))
                .and_then(|v| v.as_array())
                .and_then(|a| a.get(1))
                .and_then(|v| v.as_array())
                .and_then(|a| a.get(1))
                .and_then(|v| v.as_array())
        } else {
            None
        }
    });

    // Also try the standard 4-element EPF/CF header: root[3][1]
    let props_from_4elem = json_val.as_array().and_then(|root| {
        root.get(3)
            .and_then(|v| v.as_array())
            .and_then(|a| a.get(1))
            .and_then(|v| v.as_array())
    });

    let props_branch = props_from_3elem.or(props_from_4elem);

    // Extract UUID — try both paths
    let uuid = props_branch
        .and_then(|pb| pb.get(1))
        .and_then(|v| v.as_array())
        .and_then(|u| u.get(2))
        .and_then(|v| v.as_str())
        .map(|s| unquote(s).to_string())
        .or_else(|| find_uuid_in_json(json_val))
        .unwrap_or_else(|| "00000000-0000-0000-0000-000000000000".to_string());

    if let Some(pb) = props_branch {
        let name = pb
            .get(2)
            .and_then(|v| v.as_str())
            .map(|s| unquote(s).to_string())
            .filter(|n| !n.is_empty())
            .or_else(|| override_name.map(|n| n.to_string()))
            .unwrap_or_else(|| group_name.to_string());

        let synonyms = extract_all_synonyms(json_val);
        let syn_items = render_synonym_items(&synonyms, "\t\t\t\t");
        let synonym_xml = if syn_items.is_empty() {
            "\t\t\t<Synonym/>".to_string()
        } else {
            format!("\t\t\t<Synonym>\n{}\t\t\t</Synonym>", syn_items)
        };

        if element_name == "Form" {
            return Some(format!(
                r#"{header}
	<{elem} uuid="{uuid}">
		<Properties>
			<Name>{name}</Name>
{synonym_xml}
			<Comment/>
			<FormType>Managed</FormType>
			<IncludeHelpInContents>false</IncludeHelpInContents>
			<UsePurposes>
				<v8:Value xsi:type="app:ApplicationUsePurpose">PlatformApplication</v8:Value>
				<v8:Value xsi:type="app:ApplicationUsePurpose">MobilePlatformApplication</v8:Value>
			</UsePurposes>
			<UseInInterfaceCompatibilityMode>Any</UseInInterfaceCompatibilityMode>
			<ExtendedPresentation/>
		</Properties>
	</{elem}>
</MetaDataObject>"#,
                header = METADATA_XML_HEADER,
                elem = element_name,
                uuid = uuid,
                name = name,
                synonym_xml = synonym_xml,
            ));
        }

        return Some(format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<MetaDataObject xmlns="http://v8.1c.ru/8.3/MDClasses" version="2.21">
	<{elem} uuid="{uuid}">
		<Properties>
			<Name>{name}</Name>
{synonym_xml}
			<Comment/>
		</Properties>
	</{elem}>
</MetaDataObject>"#,
            elem = element_name,
            uuid = uuid,
            name = name,
            synonym_xml = synonym_xml,
        ));
    }

    // Complete fallback — no parseable props in bracket data
    let name = override_name.unwrap_or(group_name);
    // For Form elements, always produce full Form XML even in fallback
    if element_name == "Form" {
        let synonym_xml = if name.is_empty() {
            "\t\t\t<Synonym/>".to_string()
        } else {
            format!("\t\t\t<Synonym>\n\t\t\t\t<v8:item>\n\t\t\t\t\t<v8:lang>ru</v8:lang>\n\t\t\t\t\t<v8:content>{}</v8:content>\n\t\t\t\t</v8:item>\n\t\t\t</Synonym>", name)
        };
        return Some(format!(
            r#"{header}
	<{elem} uuid="{uuid}">
		<Properties>
			<Name>{name}</Name>
{synonym_xml}
			<Comment/>
			<FormType>Managed</FormType>
			<IncludeHelpInContents>false</IncludeHelpInContents>
			<UsePurposes>
				<v8:Value xsi:type="app:ApplicationUsePurpose">PlatformApplication</v8:Value>
				<v8:Value xsi:type="app:ApplicationUsePurpose">MobilePlatformApplication</v8:Value>
			</UsePurposes>
			<UseInInterfaceCompatibilityMode>Any</UseInInterfaceCompatibilityMode>
			<ExtendedPresentation/>
		</Properties>
	</{elem}>
</MetaDataObject>"#,
            header = METADATA_XML_HEADER,
            elem = element_name,
            uuid = uuid,
            name = name,
            synonym_xml = synonym_xml,
        ));
    }

    None
}

/// Fallback: generate minimal XML without RawData/CDATA.
pub fn generate_fallback_xml(json_val: &Value, element_name: &str) -> String {
    let uuid = find_uuid_in_json(json_val)
        .unwrap_or_else(|| "00000000-0000-0000-0000-000000000000".to_string());
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<MetaDataObject xmlns="http://v8.1c.ru/8.3/MDClasses" version="2.21">
	<{elem} uuid="{uuid}">
	</{elem}>
</MetaDataObject>"#,
        elem = element_name,
        uuid = uuid,
    )
}

// ---------------------------------------------------------------------------
// XML → bracket round-trip
// ---------------------------------------------------------------------------

fn extract_xml_tag(xml: &str, tag: &str) -> Option<String> {
    let start_tag = format!("<{}>", tag);
    let end_tag = format!("</{}>", tag);
    if let Some(start) = xml.find(&start_tag) {
        if let Some(end) = xml[start..].find(&end_tag) {
            return Some(xml[start + start_tag.len()..start + end].to_string());
        }
    }
    let start_tag_space = format!("<{} ", tag);
    if let Some(start) = xml.find(&start_tag_space) {
        if let Some(close_bracket) = xml[start..].find('>') {
            if let Some(end) = xml[start + close_bracket..].find(&end_tag) {
                return Some(
                    xml[start + close_bracket + 1..start + close_bracket + end].to_string(),
                );
            }
        }
    }
    None
}

fn extract_xml_attr(xml: &str, tag: &str, attr: &str) -> Option<String> {
    let start_tag_space = format!("<{} ", tag);
    if let Some(start) = xml.find(&start_tag_space) {
        if let Some(close_bracket) = xml[start..].find('>') {
            let tag_content = &xml[start..start + close_bracket];
            let attr_matcher = format!("{}=", attr);
            if let Some(attr_start) = tag_content.find(&attr_matcher) {
                let val_start = attr_start + attr_matcher.len() + 1;
                if let Some(val_end) = tag_content[val_start..].find('"') {
                    return Some(tag_content[val_start..val_start + val_end].to_string());
                }
            }
        }
    }
    None
}

/// Convert an XML metadata object back to bracket format bytes (for writing).
pub fn xml_to_bracket(xml: &str) -> Option<Vec<u8>> {
    if let Some(cdata_start) = xml.find("<![CDATA[") {
        if let Some(cdata_end) = xml.find("]]>") {
            let json_str = &xml[cdata_start + 9..cdata_end];
            if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(json_str) {
                if let Ok(bracket_bytes) = serialize_json_to_bracket(&json_val) {
                    return Some(bracket_bytes);
                }
            }
        }
    }

    let class_name = if xml.contains("<ExternalDataProcessor") {
        "ExternalDataProcessor"
    } else if xml.contains("<ExternalReport") {
        "ExternalReport"
    } else {
        return None;
    };

    let uuid = extract_xml_attr(xml, class_name, "uuid")?;
    let class_id = extract_xml_tag(xml, "xr:ClassId")?;
    let object_id = extract_xml_tag(xml, "xr:ObjectId")?;
    let type_id = extract_xml_tag(xml, "xr:TypeId")?;
    let value_id = extract_xml_tag(xml, "xr:ValueId")?;
    let name = extract_xml_tag(xml, "Name").unwrap_or_default();

    // Extract the first synonym pair present in the XML
    let lang = extract_xml_tag(xml, "v8:lang").unwrap_or_else(|| "ru".to_string());
    let synonym = extract_xml_tag(xml, "v8:content").unwrap_or_default();
    let comment = extract_xml_tag(xml, "Comment").unwrap_or_default();

    let mut bracket_str = "{1,\n{{uuid}},1,\n{{class_id},\n{1,\n{4,{type_id},{value_id},\n{0,\n{3,\n{1,0,{object_id}},\"{name}\",\n{1,\"{lang}\",\"{synonym}\"},\"{comment}\",0,0,00000000-0000-0000-0000-000000000000,0}\n},00000000-0000-0000-0000-000000000000,\"\",00000000-0000-0000-0000-000000000000},4,\n{2bcef0d1-0981-11d6-b9b8-0050bae0a95d,0},\n{3daea016-69b7-4ed4-9453-127911372fe6,0},\n{d5b0e5ed-256d-401c-9c36-f630cafd8a62,0},\n{ec6bb5e5-b7a8-4d75-bec9-658107a699cf,0}\n}\n}\n}".to_string();
    bracket_str = bracket_str.replace("{uuid}", &uuid);
    bracket_str = bracket_str.replace("{class_id}", &class_id);
    bracket_str = bracket_str.replace("{type_id}", &type_id);
    bracket_str = bracket_str.replace("{value_id}", &value_id);
    bracket_str = bracket_str.replace("{object_id}", &object_id);
    bracket_str = bracket_str.replace("{name}", &name);
    bracket_str = bracket_str.replace("{lang}", &lang);
    bracket_str = bracket_str.replace("{synonym}", &synonym);
    bracket_str = bracket_str.replace("{comment}", &comment);

    Some(bracket_str.into_bytes())
}

/// Detect ExternalDataProcessor/ExternalReport class from bracket JSON structure.
pub fn detect_ext_class(json_val: &Value) -> Option<&'static str> {
    let root = json_val.as_array()?;
    let inner = root.get(3)?.as_array()?;
    let cid = inner.first()?.as_str()?;
    match cid {
        "c3831ec8-d8d5-4f93-8a22-f9bfae07327f" => Some("ExternalDataProcessor"),
        "e41aff26-25cf-4bb6-b6c1-3f478a75f374" => Some("ExternalReport"),
        _ => None,
    }
}

/// Parse bracket bytes to JSON value, logging errors.
pub fn parse_bracket_for_xml(bracket: &str) -> Option<Value> {
    match parse_bracket_to_json(bracket.as_bytes()) {
        Ok(v) => Some(v),
        Err(e) => {
            eprintln!("bracket_to_xml parse error: {}", e);
            None
        }
    }
}
