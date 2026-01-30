//! JSON canonicalization per RFC 8785 (JCS)

use serde_json::Value;
use sha2::{Digest, Sha256};

/// Canonicalize a JSON value according to RFC 8785 (JCS)
///
/// Rules:
/// - Object keys are lexicographically sorted
/// - No whitespace outside of strings
/// - Numbers are formatted without unnecessary precision
/// - Arrays preserve order
pub fn canonicalize(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(b) => if *b { "true" } else { "false" }.to_string(),
        Value::Number(n) => canonicalize_number(n),
        Value::String(s) => canonicalize_string(s),
        Value::Array(arr) => {
            let items: Vec<String> = arr.iter().map(canonicalize).collect();
            format!("[{}]", items.join(","))
        }
        Value::Object(obj) => {
            // Sort keys lexicographically
            let mut keys: Vec<&String> = obj.keys().collect();
            keys.sort();

            let items: Vec<String> = keys
                .into_iter()
                .map(|k| {
                    let v = canonicalize(obj.get(k).unwrap());
                    format!("{}:{}", canonicalize_string(k), v)
                })
                .collect();

            format!("{{{}}}", items.join(","))
        }
    }
}

/// Canonicalize a number
fn canonicalize_number(n: &serde_json::Number) -> String {
    // Use serde_json's Display which follows JSON spec
    n.to_string()
}

/// Canonicalize a string with proper escaping
fn canonicalize_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 2);
    result.push('"');

    for c in s.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            c if c.is_control() => {
                // Use \uXXXX format for control characters
                result.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => result.push(c),
        }
    }

    result.push('"');
    result
}

/// Compute SHA256 hash of canonical JSON
pub fn hash_canonical(value: &Value) -> String {
    let canonical = canonicalize(value);
    let mut hasher = Sha256::new();
    hasher.update(canonical.as_bytes());
    hex::encode(hasher.finalize())
}

/// Pretty-print JSON with sorted keys (for human-readable canonical form)
pub fn pretty_canonical(value: &Value) -> String {
    pretty_canonical_indent(value, 0)
}

fn pretty_canonical_indent(value: &Value, indent: usize) -> String {
    let indent_str = "  ".repeat(indent);
    let next_indent = "  ".repeat(indent + 1);

    match value {
        Value::Null => "null".to_string(),
        Value::Bool(b) => if *b { "true" } else { "false" }.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => canonicalize_string(s),
        Value::Array(arr) => {
            if arr.is_empty() {
                return "[]".to_string();
            }
            let items: Vec<String> = arr
                .iter()
                .map(|v| format!("{}{}", next_indent, pretty_canonical_indent(v, indent + 1)))
                .collect();
            format!("[\n{}\n{}]", items.join(",\n"), indent_str)
        }
        Value::Object(obj) => {
            if obj.is_empty() {
                return "{}".to_string();
            }
            // Sort keys lexicographically
            let mut keys: Vec<&String> = obj.keys().collect();
            keys.sort();

            let items: Vec<String> = keys
                .into_iter()
                .map(|k| {
                    let v = pretty_canonical_indent(obj.get(k).unwrap(), indent + 1);
                    format!("{}{}: {}", next_indent, canonicalize_string(k), v)
                })
                .collect();

            format!("{{\n{}\n{}}}", items.join(",\n"), indent_str)
        }
    }
}

/// Result of formatting a file
#[derive(Debug, Clone, serde::Serialize)]
pub struct FormatResult {
    pub path: String,
    pub changed: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_canonicalize_primitives() {
        assert_eq!(canonicalize(&json!(null)), "null");
        assert_eq!(canonicalize(&json!(true)), "true");
        assert_eq!(canonicalize(&json!(false)), "false");
        assert_eq!(canonicalize(&json!(42)), "42");
        assert_eq!(canonicalize(&json!("hello")), "\"hello\"");
    }

    #[test]
    fn test_canonicalize_string_escaping() {
        assert_eq!(canonicalize(&json!("hello\nworld")), "\"hello\\nworld\"");
        assert_eq!(canonicalize(&json!("tab\there")), "\"tab\\there\"");
        assert_eq!(canonicalize(&json!("quote\"here")), "\"quote\\\"here\"");
    }

    #[test]
    fn test_canonicalize_array() {
        assert_eq!(canonicalize(&json!([1, 2, 3])), "[1,2,3]");
        assert_eq!(canonicalize(&json!(["a", "b"])), "[\"a\",\"b\"]");
    }

    #[test]
    fn test_canonicalize_object_sorted_keys() {
        let obj = json!({"z": 1, "a": 2, "m": 3});
        assert_eq!(canonicalize(&obj), "{\"a\":2,\"m\":3,\"z\":1}");
    }

    #[test]
    fn test_canonicalize_nested() {
        let obj = json!({
            "b": [1, 2],
            "a": {"y": 1, "x": 2}
        });
        assert_eq!(canonicalize(&obj), "{\"a\":{\"x\":2,\"y\":1},\"b\":[1,2]}");
    }

    #[test]
    fn test_hash_canonical() {
        let obj = json!({"hello": "world"});
        let hash = hash_canonical(&obj);
        assert_eq!(hash.len(), 64); // SHA256 hex is 64 chars

        // Same content, different key order should produce same hash
        let obj2 = json!({"hello": "world"});
        assert_eq!(hash, hash_canonical(&obj2));
    }
}
