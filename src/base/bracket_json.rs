use crate::base::parser::Node;
use crate::base::parser::StructParser;
use serde_json::Value;

pub fn parse_bracket_to_json(data: &[u8]) -> Result<Value, String> {
    let bom = [0xEF, 0xBB, 0xBF];
    let mut offset = 0;
    if data.starts_with(&bom) {
        offset = 3;
    }

    let string_data =
        std::str::from_utf8(&data[offset..]).map_err(|e| format!("Invalid UTF-8: {}", e))?;

    let parser = StructParser::new(string_data.to_string())
        .map_err(|e| format!("Bracket parse error: {:?}", e))?;

    Ok(node_to_json(parser.root(), parser.source()))
}

fn node_to_json(node: &Node, source: &str) -> Value {
    match node {
        Node::Leaf(leaf) => {
            if leaf.is_empty() {
                Value::String(String::new())
            } else {
                Value::String(source[leaf.begin..leaf.end].to_string())
            }
        }
        Node::Branch(children) => {
            let mut arr = Vec::new();
            for child in children {
                arr.push(node_to_json(child, source));
            }
            Value::Array(arr)
        }
    }
}

pub fn serialize_json_to_bracket(value: &Value) -> Result<Vec<u8>, String> {
    let mut out = String::new();
    json_to_bracket_str(value, &mut out)?;

    let mut result = vec![0xEF, 0xBB, 0xBF]; // add BOM
    result.extend_from_slice(out.as_bytes());
    Ok(result)
}

fn json_to_bracket_str(value: &Value, out: &mut String) -> Result<(), String> {
    match value {
        Value::String(s) => {
            out.push_str(s);
            Ok(())
        }
        Value::Array(arr) => {
            out.push('{');
            for (i, v) in arr.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                json_to_bracket_str(v, out)?;
            }
            out.push('}');
            Ok(())
        }
        _ => Err("Invalid JSON structure: expected Array or String".to_string()),
    }
}
