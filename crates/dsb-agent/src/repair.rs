//! Tool-call argument repair (spec 15).

use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RepairError {
    #[error("invalid tool arguments JSON after repair: {0}")]
    InvalidJson(String),
    #[error("missing required argument(s): {0}")]
    MissingRequired(String),
    #[error("arguments not an object")]
    NotObject,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RepairOutcome {
    pub arguments: Value,
    pub repair_applied: bool,
    /// Truncated original for debug logs only.
    pub original_snippet: String,
}

/// Max auto-repair attempts per tool call (spec 15): 1 then error.
const MAX_REPAIR_ATTEMPTS: u8 = 1;

/// Repair model-produced tool arguments JSON.
///
/// Allowed repairs:
/// - trailing commas
/// - single-quoted strings (unambiguous)
/// - unescaped control chars in strings
/// - args as JSON string containing object (unwrap once)
/// - strip unknown fields when schema says `additionalProperties: false`
/// - fill defaults for optional fields when schema provides default
///
/// Never: change tool name, invent required args, execute partial required sets.
pub fn repair_tool_arguments(
    raw: &str,
    schema: Option<&Value>,
) -> Result<RepairOutcome, RepairError> {
    let original_snippet = truncate(raw, 200);
    let mut repair_applied = false;
    let mut current = raw.trim().to_string();

    // Unwrap JSON string containing object once.
    if let Ok(Value::String(inner)) = serde_json::from_str::<Value>(&current) {
        let inner_trim = inner.trim();
        if inner_trim.starts_with('{') || inner_trim.starts_with('[') {
            current = inner;
            repair_applied = true;
        }
    }

    match try_parse_object(&current) {
        Ok(mut obj) => {
            if let Some(schema) = schema {
                apply_schema(&mut obj, schema, &mut repair_applied)?;
            }
            return Ok(RepairOutcome {
                arguments: obj,
                repair_applied,
                original_snippet,
            });
        }
        Err(_) => {}
    }

    // One repair pass.
    let mut attempts = 0u8;
    while attempts < MAX_REPAIR_ATTEMPTS {
        attempts += 1;
        let repaired = apply_repairs(&current);
        if repaired != current {
            repair_applied = true;
            current = repaired;
        }
        match try_parse_object(&current) {
            Ok(mut obj) => {
                if let Some(schema) = schema {
                    apply_schema(&mut obj, schema, &mut repair_applied)?;
                }
                return Ok(RepairOutcome {
                    arguments: obj,
                    repair_applied,
                    original_snippet,
                });
            }
            Err(e) => {
                if attempts >= MAX_REPAIR_ATTEMPTS {
                    return Err(RepairError::InvalidJson(e));
                }
            }
        }
    }

    Err(RepairError::InvalidJson("unrepairable".into()))
}

fn try_parse_object(s: &str) -> Result<Value, String> {
    let v: Value = serde_json::from_str(s).map_err(|e| e.to_string())?;
    match v {
        Value::Object(_) => Ok(v),
        _ => Err("not an object".into()),
    }
}

fn apply_repairs(input: &str) -> String {
    let mut s = input.to_string();
    s = strip_trailing_commas(&s);
    s = convert_single_quotes(&s);
    s = escape_control_chars_in_strings(&s);
    s
}

/// Strip trailing commas before `}` or `]`.
fn strip_trailing_commas(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == ',' {
            let mut j = i + 1;
            while j < chars.len() && chars[j].is_whitespace() {
                j += 1;
            }
            if j < chars.len() && (chars[j] == '}' || chars[j] == ']') {
                // skip comma
                i += 1;
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

/// Convert single-quoted strings to double-quoted when unambiguous.
fn convert_single_quotes(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    let mut in_double = false;
    while i < chars.len() {
        let c = chars[i];
        if in_double {
            out.push(c);
            if c == '\\' && i + 1 < chars.len() {
                out.push(chars[i + 1]);
                i += 2;
                continue;
            }
            if c == '"' {
                in_double = false;
            }
            i += 1;
            continue;
        }
        if c == '"' {
            in_double = true;
            out.push(c);
            i += 1;
            continue;
        }
        if c == '\'' {
            // parse single-quoted string
            out.push('"');
            i += 1;
            while i < chars.len() {
                let ch = chars[i];
                if ch == '\\' && i + 1 < chars.len() {
                    out.push(ch);
                    out.push(chars[i + 1]);
                    i += 2;
                    continue;
                }
                if ch == '\'' {
                    out.push('"');
                    i += 1;
                    break;
                }
                if ch == '"' {
                    out.push('\\');
                    out.push('"');
                    i += 1;
                    continue;
                }
                out.push(ch);
                i += 1;
            }
            continue;
        }
        out.push(c);
        i += 1;
    }
    out
}

fn escape_control_chars_in_strings(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_string = false;
    let mut escape = false;
    for c in s.chars() {
        if escape {
            out.push(c);
            escape = false;
            continue;
        }
        if c == '\\' && in_string {
            out.push(c);
            escape = true;
            continue;
        }
        if c == '"' {
            in_string = !in_string;
            out.push(c);
            continue;
        }
        if in_string {
            match c {
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\t' => out.push_str("\\t"),
                c if (c as u32) < 0x20 => {
                    out.push_str(&format!("\\u{:04x}", c as u32));
                }
                _ => out.push(c),
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn apply_schema(
    obj: &mut Value,
    schema: &Value,
    repair_applied: &mut bool,
) -> Result<(), RepairError> {
    let Some(map) = obj.as_object_mut() else {
        return Err(RepairError::NotObject);
    };

    let properties = schema.get("properties").and_then(|p| p.as_object());
    let required: Vec<&str> = schema
        .get("required")
        .and_then(|r| r.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    // Fill defaults for optional properties.
    if let Some(props) = properties {
        for (key, prop_schema) in props {
            if !map.contains_key(key) {
                if let Some(default) = prop_schema.get("default") {
                    map.insert(key.clone(), default.clone());
                    *repair_applied = true;
                }
            }
        }
    }

    // Strip unknown when additionalProperties is false.
    let additional = schema
        .get("additionalProperties")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    if !additional {
        if let Some(props) = properties {
            let unknown: Vec<String> = map
                .keys()
                .filter(|k| !props.contains_key(k.as_str()))
                .cloned()
                .collect();
            for k in unknown {
                map.remove(&k);
                *repair_applied = true;
            }
        }
    }

    // Required keys must exist — never invent.
    let mut missing = Vec::new();
    for r in required {
        if !map.contains_key(r) {
            missing.push(r.to_string());
        }
    }
    if !missing.is_empty() {
        return Err(RepairError::MissingRequired(missing.join(", ")));
    }

    Ok(())
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        s.chars().take(max).collect::<String>() + "…"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn repair_trailing_comma() {
        let out = repair_tool_arguments(r#"{"path":"a.rs",}"#, None).unwrap();
        assert_eq!(out.arguments["path"], "a.rs");
        assert!(out.repair_applied);
    }

    #[test]
    fn repair_single_quotes() {
        let out = repair_tool_arguments(r#"{'path':'a.rs'}"#, None).unwrap();
        assert_eq!(out.arguments["path"], "a.rs");
        assert!(out.repair_applied);
    }

    #[test]
    fn repair_json_string_wrapper() {
        let raw = serde_json::to_string(r#"{"path":"x"}"#).unwrap();
        let out = repair_tool_arguments(&raw, None).unwrap();
        assert_eq!(out.arguments["path"], "x");
        assert!(out.repair_applied);
    }

    #[test]
    fn repair_does_not_invent_required() {
        let schema = json!({
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "optional": {"type": "string", "default": "y"}
            },
            "required": ["path"],
            "additionalProperties": false
        });
        let err = repair_tool_arguments(r#"{"optional":"z"}"#, Some(&schema)).unwrap_err();
        assert!(matches!(err, RepairError::MissingRequired(_)));
    }

    #[test]
    fn fills_default_optional() {
        let schema = json!({
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "mode": {"type": "string", "default": "r"}
            },
            "required": ["path"]
        });
        let out = repair_tool_arguments(r#"{"path":"a"}"#, Some(&schema)).unwrap();
        assert_eq!(out.arguments["mode"], "r");
        assert!(out.repair_applied);
    }

    #[test]
    fn strips_unknown_when_additional_false() {
        let schema = json!({
            "type": "object",
            "properties": {"path": {"type": "string"}},
            "required": ["path"],
            "additionalProperties": false
        });
        let out = repair_tool_arguments(r#"{"path":"a","extra":1}"#, Some(&schema)).unwrap();
        assert!(out.arguments.get("extra").is_none());
        assert!(out.repair_applied);
    }

    #[test]
    fn valid_json_no_repair() {
        let out = repair_tool_arguments(r#"{"path":"a"}"#, None).unwrap();
        assert!(!out.repair_applied);
        assert_eq!(out.arguments["path"], "a");
    }

    #[test]
    fn unrepairable_errors() {
        let err = repair_tool_arguments("not-json-at-all", None).unwrap_err();
        assert!(matches!(err, RepairError::InvalidJson(_)));
    }
}
