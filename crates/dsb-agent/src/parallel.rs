//! Parallel independent tool scheduling (spec 50 / G4).

use dsb_tools::ToolName;
use serde_json::Value;

/// Whether a tool call is treated as mutating for scheduling (fail-closed).
pub fn is_mutating_tool(name: &str, arguments: &Value) -> bool {
    if name.starts_with("mcp__") {
        return true;
    }
    match ToolName::parse(name) {
        Some(ToolName::Read)
        | Some(ToolName::Grep)
        | Some(ToolName::Skill)
        | Some(ToolName::BashCollect) => false,
        Some(ToolName::Plan) => {
            // Only `get` is read-only; missing action → mutating.
            !matches!(
                arguments.get("action").and_then(|v| v.as_str()),
                Some("get")
            )
        }
        Some(ToolName::Edit)
        | Some(ToolName::Write)
        | Some(ToolName::Bash)
        | Some(ToolName::Subagent)
        | None => true,
    }
}

/// Max concurrent read-only tools per turn (product default).
pub const MAX_PARALLEL_READONLY: usize = 8;

/// Partition tool call indices into read-only vs mutating (preserve order within each).
pub fn partition_indices(names_and_args: &[(String, Value)]) -> (Vec<usize>, Vec<usize>) {
    let mut ro = Vec::new();
    let mut mu = Vec::new();
    for (i, (name, args)) in names_and_args.iter().enumerate() {
        if is_mutating_tool(name, args) {
            mu.push(i);
        } else {
            ro.push(i);
        }
    }
    (ro, mu)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn classifier_readonly_vs_mutating() {
        assert!(!is_mutating_tool("read", &json!({"path": "a"})));
        assert!(!is_mutating_tool("grep", &json!({"pattern": "x"})));
        assert!(!is_mutating_tool("skill", &json!({"name": "s"})));
        assert!(!is_mutating_tool("plan", &json!({"action": "get"})));
        assert!(is_mutating_tool(
            "plan",
            &json!({"action": "set", "items": ["a"]})
        ));
        assert!(is_mutating_tool("edit", &json!({})));
        assert!(is_mutating_tool("write", &json!({})));
        assert!(is_mutating_tool("bash", &json!({})));
        assert!(is_mutating_tool("mcp__demo__ping", &json!({})));
        assert!(is_mutating_tool("unknown_xyz", &json!({})));
    }

    #[test]
    fn partition_preserves_order() {
        let batch = vec![
            ("read".into(), json!({})),
            ("edit".into(), json!({})),
            ("grep".into(), json!({})),
            ("write".into(), json!({})),
        ];
        let (ro, mu) = partition_indices(&batch);
        assert_eq!(ro, vec![0, 2]);
        assert_eq!(mu, vec![1, 3]);
    }

    /// Path A / G010 stamp maps product tool names → short ToolName before
    /// partition. Keep this mapping honest for Spec 50 RO parallel / mutate serial.
    #[test]
    fn product_path_a_names_partition_like_stamp() {
        let batch = vec![
            ("read_file".into(), json!({"target_file": "a.txt"})),
            ("search_replace".into(), json!({"file_path": "a.txt"})),
            ("run_terminal_command".into(), json!({"command": "echo x"})),
            ("mcp__demo__ping".into(), json!({})),
            ("grep".into(), json!({"pattern": "x"})),
            ("unknown_tool_xyz".into(), json!({})),
        ];
        let class_input: Vec<(String, Value)> = batch
            .iter()
            .map(|(n, a): &(String, Value)| {
                let short = match n.as_str() {
                    "read_file" => "read",
                    "search_replace" => "edit",
                    "run_terminal_command" => "bash",
                    other => other,
                };
                (short.to_string(), a.clone())
            })
            .collect();
        let (ro, mu) = partition_indices(&class_input);
        // read + grep → RO; edit + bash + mcp + unknown → mutating
        assert_eq!(ro, vec![0, 4]);
        assert_eq!(mu, vec![1, 2, 3, 5]);
        assert!(is_mutating_tool("bash", &json!({"command": "true"})));
        assert!(!is_mutating_tool("read", &json!({})));
        // bash_collect remains read-only for scheduling (collect does not mutate workspace)
        assert!(!is_mutating_tool("bash_collect", &json!({"job_id": "j1"})));
    }
}
