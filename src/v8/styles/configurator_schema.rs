use crate::base::bracket_json::{parse_bracket_to_json, serialize_json_to_bracket};
use serde_json::Value;

/// Reference to a child object for the ChildObjects section in root XML.
pub struct ChildObjectRef {
    /// XML element type: "Form", "Template", etc.
    pub element_type: String,
    /// Name of the child object.
    pub name: String,
}

pub fn bracket_to_xml(bracket: &str, class_name: &str) -> Option<String> {
    let json_val = match parse_bracket_to_json(bracket.as_bytes()) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("bracket_to_xml parse error: {}", e);
            return None;
        }
    };

    // Detect class from class_id in bracket data
    let detected_class = if let Some(root) = json_val.as_array() {
        if let Some(inner) = root.get(3).and_then(|v| v.as_array()) {
            if let Some(cid) = inner.first().and_then(|s| s.as_str()) {
                if cid == "c3831ec8-d8d5-4f93-8a22-f9bfae07327f" {
                    Some("ExternalDataProcessor")
                } else if cid == "e41aff26-25cf-4bb6-b6c1-3f478a75f374" {
                    Some("ExternalReport")
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    // If not detected from class_id, try mapping from group name
    let target_class = detected_class
        .or_else(|| group_to_element_type(class_name))
        .unwrap_or(class_name);

    if target_class == "ExternalDataProcessor" || target_class == "ExternalReport" {
        return generate_ext_object(&json_val, target_class, &[]);
    }
    if target_class == "Configuration" {
        return generate_cfg_object(&json_val);
    }

    // Generic metadata object (Form, Language, Role, Template, etc.)
    if let Some(xml) = generate_generic_metadata(&json_val, target_class) {
        return Some(xml);
    }

    // Last resort: fallback with just UUID, no RawData
    Some(generate_fallback_xml(&json_val, target_class))
}

pub fn bracket_to_xml_with_children(
    bracket: &str,
    class_name: &str,
    children: &[ChildObjectRef],
) -> Option<String> {
    let json_val = match parse_bracket_to_json(bracket.as_bytes()) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("bracket_to_xml_with_children parse error: {}", e);
            return None;
        }
    };

    let detected_class = if let Some(root) = json_val.as_array() {
        if let Some(inner) = root.get(3).and_then(|v| v.as_array()) {
            if let Some(cid) = inner.first().and_then(|s| s.as_str()) {
                if cid == "c3831ec8-d8d5-4f93-8a22-f9bfae07327f" {
                    Some("ExternalDataProcessor")
                } else if cid == "e41aff26-25cf-4bb6-b6c1-3f478a75f374" {
                    Some("ExternalReport")
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let target_class = detected_class
        .or_else(|| group_to_element_type(class_name))
        .unwrap_or(class_name);

    if target_class == "ExternalDataProcessor" || target_class == "ExternalReport" {
        return generate_ext_object(&json_val, target_class, children);
    }
    if target_class == "Configuration" {
        return generate_cfg_object(&json_val);
    }

    if let Some(xml) = generate_generic_metadata(&json_val, target_class) {
        return Some(xml);
    }

    Some(generate_fallback_xml(&json_val, target_class))
}

/// Map group display names to XML element type names.
/// E.g., "Forms" -> "Form", "Templates" -> "Template"
pub fn group_to_element_type(group_name: &str) -> Option<&'static str> {
    match group_name {
        "Forms" => Some("Form"),
        "Templates" => Some("Template"),
        "Attributes" => Some("Attribute"),
        "TabularSections" => Some("TabularSection"),
        "Commands" => Some("Command"),
        "FormAttributes" => Some("FormAttribute"),
        "FormLayout" => Some("Form"),
        "TemplateLayout" => Some("Template"),
        "Languages" => Some("Language"),
        "Roles" => Some("Role"),
        "DataProcessors" => Some("DataProcessor"),
        "Reports" => Some("Report"),
        "Catalogs" => Some("Catalog"),
        "Documents" => Some("Document"),
        "Constants" => Some("Constant"),
        "Enums" => Some("Enum"),
        "CommonModules" => Some("CommonModule"),
        "CommonForms" => Some("CommonForm"),
        "CommonTemplates" => Some("CommonTemplate"),
        "Subsystems" => Some("Subsystem"),
        _ if group_name.ends_with('s') && group_name.len() > 3 => {
            // Generic plural→singular for known patterns
            let singular = &group_name[..group_name.len() - 1];
            match singular {
                "Constant" => Some("Constant"),
                "Subsystem" => Some("Subsystem"),
                _ => None,
            }
        }
        _ => None,
    }
}

fn generate_ext_object(json_val: &Value, class_name: &str, children: &[ChildObjectRef]) -> Option<String> {
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

    let mut lang = "ru";
    let mut synonym = "";
    if let Some(syn_arr) = props.get(3)?.as_array() {
        lang = syn_arr
            .get(1)?
            .as_str()
            .unwrap_or("\"ru\"")
            .trim_matches('"');
        synonym = syn_arr.get(2)?.as_str().unwrap_or("\"\"").trim_matches('"');
    }

    let comment_raw = props.get(4)?.as_str().unwrap_or("\"\"");
    let comment = comment_raw.trim_matches('"');

    // Build ChildObjects section
    let child_objects = if children.is_empty() {
        "\t\t<ChildObjects/>".to_string()
    } else {
        let mut lines = vec!["\t\t<ChildObjects>".to_string()];
        for child in children {
            lines.push(format!(
                "\t\t\t<{0}>{1}</{0}>",
                child.element_type, child.name
            ));
        }
        lines.push("\t\t</ChildObjects>".to_string());
        lines.join("\n")
    };

    // Extra properties for ExternalReport
    let extra_properties = if class_name == "ExternalReport" {
        "\n\t\t\t<MainDataCompositionSchema/>\n\t\t\t<DefaultSettingsForm/>\n\t\t\t<AuxiliarySettingsForm/>\n\t\t\t<DefaultVariantForm/>\n\t\t\t<AuxiliaryVariantForm/>\n\t\t\t<VariantsStorage/>\n\t\t\t<SettingsStorage/>"
    } else {
        ""
    };

    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<MetaDataObject xmlns="http://v8.1c.ru/8.3/MDClasses" version="2.21">
	<{class_name} uuid="{uuid}">
		<InternalInfo>
			<xr:ContainedObject xmlns:xr="http://v8.1c.ru/8.3/xcf/readable">
				<xr:ClassId>{class_id}</xr:ClassId>
				<xr:ObjectId>{object_id}</xr:ObjectId>
			</xr:ContainedObject>
			<xr:GeneratedType name="{class_name}Object.{name}" category="Object" xmlns:xr="http://v8.1c.ru/8.3/xcf/readable">
				<xr:TypeId>{type_id}</xr:TypeId>
				<xr:ValueId>{value_id}</xr:ValueId>
			</xr:GeneratedType>
		</InternalInfo>
		<Properties>
			<Name>{name}</Name>
			<Synonym>
				<v8:item xmlns:v8="http://v8.1c.ru/8.1/data/core">
					<v8:lang>{lang}</v8:lang>
					<v8:content>{synonym}</v8:content>
				</v8:item>
			</Synonym>
			<Comment>{comment}</Comment>
			<DefaultForm/>
			<AuxiliaryForm/>{extra_properties}
		</Properties>
{child_objects}
	</{class_name}>
</MetaDataObject>"#,
        class_name = class_name,
        uuid = uuid,
        class_id = class_id,
        object_id = object_id,
        name = name,
        type_id = type_id,
        value_id = value_id,
        lang = lang,
        synonym = synonym,
        comment = comment,
        child_objects = child_objects,
        extra_properties = extra_properties,
    );

    Some(xml)
}

fn generate_cfg_object(json_val: &Value) -> Option<String> {
    let root = json_val.as_array()?;
    let uuid = root.get(1)?.as_array()?.first()?.as_str()?;

    // Try to extract Name, Synonym, Comment from the configuration data.
    // The configuration structure differs from EPF — try the generic props first,
    // then fall back to UUID-only output.
    let props_html = if let Some((_, name, lang, synonym)) = extract_metadata_props(json_val) {
        format!(
            r#"		<Properties>
			<Name>{name}</Name>
			<Synonym>
				<v8:item xmlns:v8="http://v8.1c.ru/8.1/data/core">
					<v8:lang>{lang}</v8:lang>
					<v8:content>{synonym}</v8:content>
				</v8:item>
			</Synonym>
			<Comment/>
		</Properties>"#,
            name = name,
            lang = lang,
            synonym = synonym,
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

/// Extract a quoted string value, stripping surrounding quotes.
fn unquote(val: &str) -> &str {
    val.trim_matches('"')
}

/// Recursively search JSON array for a UUID-format string.
fn find_uuid_in_json(val: &Value) -> Option<String> {
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

/// Extract metadata properties (uuid, name, synonym, comment) from the standard
/// bracket format used by subordinate metadata objects (Form, Language, Role, etc.).
///
/// Expected JSON structure (from bracket `{1,{ver},0,{type_code,{props...}}}`):
/// `["1", [...], "0", [type_code, ["3", ["1","0",uuid], name, [syn_count, lang, syn], comment, ...]]]`
/// UUID at `[3][1][1][2]`, name at `[3][1][2]`, synonym array at `[3][1][3]`
fn extract_metadata_props(json_val: &Value) -> Option<(String, String, String, String)> {
    let root = json_val.as_array()?;
    // Ensure this is a type-1 structure (metadata object)
    if root.first().and_then(|v| v.as_str()) != Some("1") {
        return None;
    }
    let props_branch = root.get(3)?.as_array()?.get(1)?.as_array()?;
    // props_branch should be ["3", ["1","0",uuid], name, synonym, comment, ...]
    let uuid = props_branch
        .get(1)?
        .as_array()?
        .get(2)?
        .as_str()
        .map(|s| unquote(s).to_string())?;
    let name = props_branch
        .get(2)?
        .as_str()
        .map(|s| unquote(s).to_string())
        .unwrap_or_default();
    let mut lang = "ru".to_string();
    let mut synonym = String::new();
    if let Some(syn_arr) = props_branch.get(3)?.as_array() {
        lang = syn_arr
            .get(1)?
            .as_str()
            .map(|s| unquote(s).to_string())
            .unwrap_or_else(|| "ru".to_string());
        synonym = syn_arr
            .get(2)?
            .as_str()
            .map(|s| unquote(s).to_string())
            .unwrap_or_default();
    }
    Some((uuid, name, lang, synonym))
}

/// Generate XML for a generic metadata object (Form, Language, Role, Template, etc.).
fn generate_generic_metadata(json_val: &Value, element_name: &str) -> Option<String> {
    if let Some((uuid, name, lang, synonym)) = extract_metadata_props(json_val) {
        return Some(format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<MetaDataObject xmlns="http://v8.1c.ru/8.3/MDClasses" version="2.21">
	<{elem} uuid="{uuid}">
		<Properties>
			<Name>{name}</Name>
			<Synonym>
				<v8:item xmlns:v8="http://v8.1c.ru/8.1/data/core">
					<v8:lang>{lang}</v8:lang>
					<v8:content>{synonym}</v8:content>
				</v8:item>
			</Synonym>
			<Comment>{comment}</Comment>
		</Properties>
	</{elem}>
</MetaDataObject>"#,
            elem = element_name,
            uuid = uuid,
            name = name,
            lang = lang,
            synonym = synonym,
            comment = ""
        ));
    }
    // Fallback: try to at least extract the UUID
    let uuid = find_uuid_in_json(json_val)
        .unwrap_or_else(|| "00000000-0000-0000-0000-000000000000".to_string());
    Some(format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<MetaDataObject xmlns="http://v8.1c.ru/8.3/MDClasses" version="2.21">
	<{elem} uuid="{uuid}">
	</{elem}>
</MetaDataObject>"#,
        elem = element_name,
        uuid = uuid,
    ))
}

/// Fallback: generate minimal XML without RawData/CDATA.
fn generate_fallback_xml(json_val: &Value, element_name: &str) -> String {
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
        if let Some(close_bracket) = xml[start..].find(">") {
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
        if let Some(close_bracket) = xml[start..].find(">") {
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

    // Parse ExternalDataProcessor or ExternalReport
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
    let lang = extract_xml_tag(xml, "v8:lang").unwrap_or_else(|| "ru".to_string());
    let synonym = extract_xml_tag(xml, "v8:content").unwrap_or_default();
    let comment = extract_xml_tag(xml, "Comment").unwrap_or_default();

    // Reconstruct the bracket format for Ext Object
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
