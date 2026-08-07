pub fn try_extract_concatenated_json_objects(arguments: &str) -> Option<Vec<serde_json::Value>> {
    let trimmed = arguments.trim();

    // Quick check: must start with '{'.
    if !trimmed.starts_with('{') {
        return None;
    }

    // If it parses as valid JSON already, no recovery needed.
    if serde_json::from_str::<serde_json::Value>(trimmed).is_ok() {
        return None;
    }

    // Use serde_json::StreamDeserializer to parse concatenated JSON objects.
    // This handles nested braces correctly (unlike naive string splitting on "}{").
    let stream = serde_json::Deserializer::from_str(trimmed).into_iter::<serde_json::Value>();

    let mut objects = Vec::new();
    for result in stream {
        match result {
            Ok(value) if value.is_object() => objects.push(value),
            _ => break,
        }
    }

    // Need at least 2 objects for this to be concatenated JSON.
    if objects.len() >= 2 {
        Some(objects)
    } else {
        None
    }
}

/// Normalize empty tool call arguments to `"{}"`.
///
/// Zero-arg MCP tools (e.g. `get_me`) sometimes receive `""` from the model
/// instead of `"{}"`, which fails JSON parsing. This normalizes empty/whitespace
/// strings to `"{}"` so downstream parsing succeeds.
pub fn normalize_empty_arguments(arguments: &str) -> &str {
    if arguments.trim().is_empty() {
        "{}"
    } else {
        arguments
    }
}

/// Spec 15 one-pass repair result for Path A / Grok tool dispatch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spec15RepairOutcome {
    /// Argument string after empty-normalize + at most one repair pass.
    pub arguments: String,
    /// True if any repair transformation was applied.
    pub repair_applied: bool,
}

/// Spec 15 spirit (DeepSeek Build owner-bar G007): repair tool arguments **once**
/// before execute on the Grok dispatch path.
///
/// Allowed: trailing commas, single-quoted strings, control-char escapes,
/// unwrap JSON-string-wrapped object once, empty → `{}`.
/// Never invent required fields or rename tools (name handled by caller).
pub fn repair_tool_arguments_one_pass(raw: &str) -> Spec15RepairOutcome {
    let mut repair_applied = false;
    let mut current = normalize_empty_arguments(raw).trim().to_string();

    // Unwrap JSON string containing object once.
    if let Ok(serde_json::Value::String(inner)) = serde_json::from_str::<serde_json::Value>(&current)
    {
        let inner_trim = inner.trim();
        if inner_trim.starts_with('{') || inner_trim.starts_with('[') {
            current = inner;
            repair_applied = true;
        }
    }

    if serde_json::from_str::<serde_json::Value>(&current).is_ok() {
        return Spec15RepairOutcome {
            arguments: current,
            repair_applied,
        };
    }

    // Exactly one repair pass (Spec 15).
    let repaired = apply_spec15_repairs(&current);
    if repaired != current {
        repair_applied = true;
        current = repaired;
    }

    Spec15RepairOutcome {
        arguments: current,
        repair_applied,
    }
}

fn apply_spec15_repairs(input: &str) -> String {
    let mut s = strip_trailing_commas(input);
    s = convert_single_quotes(&s);
    s = escape_control_chars_in_strings(&s);
    s
}

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
                i += 1;
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_objects() {
        let args = r#"{"target_file": "a.java"}{"target_file": "b.java"}{"target_file": "c.java"}"#;
        let objects = try_extract_concatenated_json_objects(args).unwrap();
        assert_eq!(objects.len(), 3);
        assert_eq!(objects[0]["target_file"], "a.java");
    }

    #[test]
    fn test_no_extract_for_valid_single_object() {
        assert!(
            try_extract_concatenated_json_objects(r#"{"target_file": "src/main.rs"}"#).is_none()
        );
    }

    #[test]
    fn test_no_extract_for_valid_object_with_braces_in_value() {
        assert!(
            try_extract_concatenated_json_objects(r#"{"command": "echo '}{' && ls"}"#).is_none()
        );
    }

    #[test]
    fn test_no_extract_for_array() {
        assert!(
            try_extract_concatenated_json_objects(
                r#"[{"target_file": "a.java"}, {"target_file": "b.java"}]"#
            )
            .is_none()
        );
    }

    #[test]
    fn test_no_extract_for_empty_or_non_json() {
        assert!(try_extract_concatenated_json_objects("").is_none());
        assert!(try_extract_concatenated_json_objects("not json").is_none());
    }

    #[test]
    fn test_extract_with_nested_braces() {
        let args = r#"{"file": "a.rs", "opts": {"line": 1}}{"file": "b.rs", "opts": {"line": 2}}"#;
        let objects = try_extract_concatenated_json_objects(args).unwrap();
        assert_eq!(objects.len(), 2);
        assert_eq!(objects[0]["opts"]["line"], 1);
    }

    #[test]
    fn test_extract_with_whitespace_between_objects() {
        let objects = try_extract_concatenated_json_objects(r#"{"a": 1} {"b": 2}"#).unwrap();
        assert_eq!(objects.len(), 2);
    }

    #[test]
    fn test_extract_real_world_20_files() {
        let mut args = String::new();
        for i in 0..20 {
            args.push_str(&format!(r#"{{"target_file": "src/File{i}.java"}}"#));
        }
        let objects = try_extract_concatenated_json_objects(&args).unwrap();
        assert_eq!(objects.len(), 20);
    }

    #[test]
    fn test_no_extract_for_truncated_json() {
        assert!(try_extract_concatenated_json_objects(r#"{"a": 1} garbage"#).is_none());
    }

    /// Parse after normalizing — mirrors the production pattern in handle_tool_call.
    fn normalize_and_parse(arguments: &str) -> serde_json::Value {
        let normalized = normalize_empty_arguments(arguments);
        serde_json::from_str(normalized).unwrap_or_else(|_| serde_json::json!({"raw": arguments}))
    }

    #[test]
    fn empty_string_becomes_empty_object() {
        assert_eq!(normalize_and_parse(""), serde_json::json!({}));
    }

    #[test]
    fn whitespace_only_becomes_empty_object() {
        assert_eq!(normalize_and_parse("   "), serde_json::json!({}));
        assert_eq!(normalize_and_parse("\n\t"), serde_json::json!({}));
    }

    #[test]
    fn valid_json_unchanged() {
        assert_eq!(
            normalize_and_parse(r#"{"query": "test"}"#),
            serde_json::json!({"query": "test"})
        );
    }

    #[test]
    fn empty_object_string_unchanged() {
        assert_eq!(normalize_and_parse("{}"), serde_json::json!({}));
    }

    #[test]
    fn invalid_json_falls_back_to_raw() {
        let result = normalize_and_parse("not json");
        assert_eq!(result["raw"], "not json");
    }

    #[test]
    fn complex_args_with_arrays_unchanged() {
        let args = r#"{"pages": [{"title": "Test"}], "limit": 10}"#;
        let result = normalize_and_parse(args);
        assert!(result["pages"].is_array());
        assert_eq!(result["limit"], 10);
    }

    #[test]
    fn normalize_empty_returns_braces() {
        assert_eq!(normalize_empty_arguments(""), "{}");
        assert_eq!(normalize_empty_arguments("   "), "{}");
        assert_eq!(normalize_empty_arguments("\n\t"), "{}");
    }

    #[test]
    fn normalize_non_empty_passthrough() {
        assert_eq!(normalize_empty_arguments(r#"{"a":1}"#), r#"{"a":1}"#);
        assert_eq!(normalize_empty_arguments("not json"), "not json");
    }

    #[test]
    fn spec15_trailing_comma_repairs_once() {
        let raw = r#"{"file_path":"a.rs","old_string":"x","new_string":"y",}"#;
        let out = repair_tool_arguments_one_pass(raw);
        assert!(out.repair_applied);
        let v: serde_json::Value = serde_json::from_str(&out.arguments).unwrap();
        assert_eq!(v["file_path"], "a.rs");
        assert_eq!(v["new_string"], "y");
    }

    #[test]
    fn spec15_single_quotes_repair() {
        let raw = r#"{'file_path':'a.rs'}"#;
        let out = repair_tool_arguments_one_pass(raw);
        assert!(out.repair_applied);
        let v: serde_json::Value = serde_json::from_str(&out.arguments).unwrap();
        assert_eq!(v["file_path"], "a.rs");
    }

    #[test]
    fn spec15_valid_json_no_repair() {
        let raw = r#"{"file_path":"a.rs"}"#;
        let out = repair_tool_arguments_one_pass(raw);
        assert!(!out.repair_applied);
        assert_eq!(out.arguments, raw);
    }

    #[test]
    fn spec15_unrepairable_stays_unparseable() {
        let out = repair_tool_arguments_one_pass("not-json-at-all");
        assert!(serde_json::from_str::<serde_json::Value>(&out.arguments).is_err());
    }
}
