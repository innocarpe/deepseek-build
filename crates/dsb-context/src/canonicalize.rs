//! Canonical serialization for byte-stable prefixes (spec 10 §1.3).

use serde_json::Value;

/// Recursively sort object keys lexicographically.
pub fn sort_keys(value: Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut keys: Vec<_> = map.keys().cloned().collect();
            keys.sort();
            let mut out = serde_json::Map::new();
            for k in keys {
                if let Some(v) = map.get(&k) {
                    out.insert(k, sort_keys(v.clone()));
                }
            }
            Value::Object(out)
        }
        Value::Array(items) => Value::Array(items.into_iter().map(sort_keys).collect()),
        other => other,
    }
}

/// Canonical JSON bytes: sorted keys, compact separators via `serde_json::to_vec`.
pub fn canonicalize_json(value: &Value) -> Result<Vec<u8>, serde_json::Error> {
    let sorted = sort_keys(value.clone());
    serde_json::to_vec(&sorted)
}

/// UTF-8 encoding of the canonical form of stable prefix messages.
///
/// Messages are serialized as a JSON array with sorted keys on every object.
pub fn stable_prefix_bytes(messages_json: &Value) -> Result<Vec<u8>, serde_json::Error> {
    canonicalize_json(messages_json)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn sorted_tool_schema_keys() {
        let a = json!({"b": 1, "a": {"z": 1, "y": 2}});
        let b = json!({"a": {"y": 2, "z": 1}, "b": 1});
        assert_eq!(
            canonicalize_json(&a).unwrap(),
            canonicalize_json(&b).unwrap()
        );
    }
}
