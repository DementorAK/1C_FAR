//! Schema module — public API for converting 1C bracket data to XML and back.
//!
//! Internal implementation is split across sub-modules:
//! - [`synonyms`] — language-agnostic synonym extraction and rendering
//! - [`metadata_xml`] — XML generators for metadata objects (EPF, ERF, CF, etc.)
//! - [`form_layout`] — form layout XML generation

pub mod form_layout;
pub mod metadata_xml;
pub mod synonyms;

use metadata_xml::{
    detect_ext_class, generate_cfg_object, generate_ext_object, generate_fallback_xml,
    generate_generic_metadata, parse_bracket_for_xml,
};
use std::collections::HashMap;

// Re-export types used by external callers
pub use form_layout::get_group_type_and_name_idx_pub;
pub use metadata_xml::{
    extract_epf_child_objects, extract_epf_uuid_name_map, gen_attribute_xml, xml_to_bracket, ChildObjectRef,
};

// ---------------------------------------------------------------------------
// group_to_element_type — maps folder/group names to XML element type names
// ---------------------------------------------------------------------------

/// Map group display names to XML element type names.
/// E.g., "Forms" → "Form", "Templates" → "Template"
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
            // Generic plural→singular fallback
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

// ---------------------------------------------------------------------------
// bracket_to_xml — top-level dispatcher (single object, no children)
// ---------------------------------------------------------------------------

pub fn bracket_to_xml(
    bracket: &str,
    class_name: &str,
    override_name: Option<&str>,
) -> Option<String> {
    let json_val = parse_bracket_for_xml(bracket)?;

    let detected_class = detect_ext_class(&json_val);
    let target_class = detected_class
        .or_else(|| group_to_element_type(class_name))
        .unwrap_or(class_name);

    if target_class == "ExternalDataProcessor" || target_class == "ExternalReport" {
        return generate_ext_object(&json_val, target_class, &[], None);
    }
    if target_class == "Configuration" {
        return generate_cfg_object(&json_val);
    }

    if let Some(xml) = generate_generic_metadata(&json_val, target_class, class_name, override_name)
    {
        return Some(xml);
    }

    Some(generate_fallback_xml(&json_val, target_class))
}

// ---------------------------------------------------------------------------
// bracket_to_xml_with_children — dispatcher with child object list
// ---------------------------------------------------------------------------

pub fn bracket_to_xml_with_children(
    bracket: &str,
    class_name: &str,
    children: &[ChildObjectRef],
    default_form: Option<&str>,
) -> Option<String> {
    let json_val = parse_bracket_for_xml(bracket)?;

    let detected_class = detect_ext_class(&json_val);
    let target_class = detected_class
        .or_else(|| group_to_element_type(class_name))
        .unwrap_or(class_name);

    if target_class == "ExternalDataProcessor" || target_class == "ExternalReport" {
        return generate_ext_object(&json_val, target_class, children, default_form);
    }
    if target_class == "Configuration" {
        return generate_cfg_object(&json_val);
    }

    if let Some(xml) = generate_generic_metadata(&json_val, target_class, class_name, None) {
        return Some(xml);
    }

    Some(generate_fallback_xml(&json_val, target_class))
}

// ---------------------------------------------------------------------------
// bracket_to_formlayout_xml — delegate to form_layout module
// ---------------------------------------------------------------------------

pub fn bracket_to_formlayout_xml(
    bracket: &str,
    parent_name: &str,
    parent_type: &str,
    uuid_map: Option<&HashMap<String, String>>,
) -> Option<(String, Option<String>)> {
    form_layout::bracket_to_formlayout_xml(bracket, parent_name, parent_type, uuid_map)
}
