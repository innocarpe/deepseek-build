//! Prefix shape — component-level attribution for cache-epoch changes
//! (spec 10 §1.5.1).
//!
//! An epoch ([`crate::PrefixEpoch`]) answers *whether* the stable prefix moved.
//! A [`PrefixShape`] answers **what** moved: one hash per §1.1 component, taken
//! over the exact document strings the builder emits.
//!
//! The shape is observational. It is computed from the same strings that
//! compose the prefix, so it cannot change `stable_prefix_bytes` — and it uses
//! the same document functions, so a component hash cannot drift from the bytes
//! it names.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Hex chars kept from a sub-component hash. Enough to name a change; it is not
/// an authenticator (spec 10 §5).
const SUB_HASH_HEX: usize = 16;

/// SHA-256 of `bytes`, truncated to [`SUB_HASH_HEX`] hex chars.
pub fn sub_hash(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hex::encode(hasher.finalize());
    digest[..SUB_HASH_HEX].to_string()
}

/// One §1.1 component of the stable prefix.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PrefixAxis {
    System,
    Tools,
    Skills,
    Environment,
    ProjectInstructions,
}

impl PrefixAxis {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Tools => "tools",
            Self::Skills => "skills",
            Self::Environment => "environment",
            Self::ProjectInstructions => "project_instructions",
        }
    }

    /// Log order: the §1.1 order, so two logs of the same change read alike.
    pub const ALL: [Self; 5] = [
        Self::System,
        Self::Tools,
        Self::Skills,
        Self::Environment,
        Self::ProjectInstructions,
    ];
}

/// Per-component hashes over exactly the documents `PrefixBuilder::build` emits.
///
/// All fields are hashes or identifiers — never instruction content, never a
/// cwd value (spec 10 §1.5.1 rule 3). An absent/empty shape means "no shape",
/// not "everything changed".
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrefixShape {
    pub system: String,
    pub tools: String,
    pub skills: String,
    pub environment: String,
    pub project_instructions: String,
    /// tool name → hash of that tool's own canonical document (sorted by name).
    #[serde(default)]
    pub tool_names: BTreeMap<String, String>,
    /// skill name → hash of that skill's index entry (sorted by name).
    #[serde(default)]
    pub skill_names: BTreeMap<String, String>,
    /// `os_family` | `cwd` → hash of that value alone.
    #[serde(default)]
    pub environment_axes: BTreeMap<String, String>,
}

impl PrefixShape {
    /// True when every component hash matches. Two equal shapes for one set of
    /// inputs are the same shape; the epoch is separately compared.
    pub fn components_equal(&self, other: &Self) -> bool {
        self.system == other.system
            && self.tools == other.tools
            && self.skills == other.skills
            && self.environment == other.environment
            && self.project_instructions == other.project_instructions
    }
}

/// What changed between two prefix shapes (spec 10 §1.5.1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrefixChange {
    /// Components that moved, in §1.1 order.
    pub axes: Vec<PrefixAxis>,
    /// `tools.added=write`, `environment.cwd`, … in deterministic order.
    pub detail: Vec<String>,
    /// Epoch moved but no component hash did — a coverage bug in §1.5.1,
    /// never a provider event.
    pub unattributed: bool,
}

impl PrefixChange {
    /// Compare two shapes. Call this only when a previous shape exists
    /// (spec 10 §1.5.1 rule 4); `epoch_moved` is the epoch comparison, which is
    /// what decides whether the provider saw different prefix bytes.
    pub fn between(prev: &PrefixShape, cur: &PrefixShape, epoch_moved: bool) -> Self {
        let mut axes: Vec<PrefixAxis> = PrefixAxis::ALL
            .into_iter()
            .filter(|axis| match axis {
                PrefixAxis::System => prev.system != cur.system,
                PrefixAxis::Tools => prev.tools != cur.tools,
                PrefixAxis::Skills => prev.skills != cur.skills,
                PrefixAxis::Environment => prev.environment != cur.environment,
                PrefixAxis::ProjectInstructions => {
                    prev.project_instructions != cur.project_instructions
                }
            })
            .collect();

        let mut detail = Vec::new();
        if axes.contains(&PrefixAxis::Tools) {
            collect_named_detail("tools", &prev.tool_names, &cur.tool_names, &mut detail);
            // The document moved but every named entry matches: the tool list
            // was reordered. Tool order is byte-affecting (§1.1 item 2), so this
            // is a real axis and names nothing at entry level.
            if detail.is_empty() {
                detail.push("tools.reordered".to_string());
            }
        }
        if axes.contains(&PrefixAxis::Skills) {
            collect_named_detail("skills", &prev.skill_names, &cur.skill_names, &mut detail);
        }
        if axes.contains(&PrefixAxis::Environment) {
            // Fixed axis order: os_family then cwd.
            for (key, axis_name) in [
                ("os_family", "environment.os_family"),
                ("cwd", "environment.cwd"),
            ] {
                let before = prev.environment_axes.get(key);
                let after = cur.environment_axes.get(key);
                if before != after {
                    detail.push(axis_name.to_string());
                }
            }
        }

        let unattributed = epoch_moved && axes.is_empty();
        if unattributed {
            axes = Vec::new();
        }
        Self {
            axes,
            detail,
            unattributed,
        }
    }

    /// `prefix_change=` value: axis names, `none`, or `unattributed`.
    pub fn label(&self) -> String {
        if self.unattributed {
            return "unattributed".to_string();
        }
        if self.axes.is_empty() {
            return "none".to_string();
        }
        self.axes
            .iter()
            .map(|a| a.as_str())
            .collect::<Vec<_>>()
            .join(",")
    }

    /// `prefix_change_detail=` value, when the change has finer axes.
    pub fn detail_label(&self) -> Option<String> {
        if self.detail.is_empty() {
            None
        } else {
            Some(self.detail.join(","))
        }
    }

    /// Log block for a comparison (spec 10 §1.5.1): the change line, then the
    /// detail line only when there is detail. `prev`/`cur` are the short epoch
    /// hashes, so one grep shows what moved and between which builds.
    pub fn log_block(&self, prev_epoch_short: &str, cur_epoch_short: &str) -> String {
        let mut out = format!(
            "prefix_change={} prev={prev_epoch_short} cur={cur_epoch_short}",
            self.label()
        );
        if let Some(detail) = self.detail_label() {
            out.push('\n');
            out.push_str(&format!("prefix_change_detail={detail}"));
        }
        out
    }
}

fn collect_named_detail(
    prefix: &str,
    prev: &BTreeMap<String, String>,
    cur: &BTreeMap<String, String>,
    out: &mut Vec<String>,
) {
    // Deterministic: added, removed, changed; each sorted by name (BTreeMap).
    for (name, hash) in cur {
        match prev.get(name) {
            None => out.push(format!("{prefix}.added={name}")),
            Some(before) if before != hash => out.push(format!("{prefix}.changed={name}")),
            Some(_) => {}
        }
    }
    for name in prev.keys() {
        if !cur.contains_key(name) {
            out.push(format!("{prefix}.removed={name}"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shape(system: &str, tools: &str) -> PrefixShape {
        PrefixShape {
            system: sub_hash(system.as_bytes()),
            tools: sub_hash(tools.as_bytes()),
            ..PrefixShape::default()
        }
    }

    #[test]
    fn unchanged_shape_reports_none() {
        let a = shape("sys", "tools");
        let c = PrefixChange::between(&a, &a.clone(), false);
        assert_eq!(c.label(), "none");
        assert!(c.detail_label().is_none());
        assert!(!c.unattributed);
    }

    #[test]
    fn system_change_names_system_only() {
        let prev = shape("sys-a", "tools");
        let cur = shape("sys-b", "tools");
        let c = PrefixChange::between(&prev, &cur, true);
        assert_eq!(c.label(), "system");
        assert!(c.detail_label().is_none(), "system has no finer axis");
        assert!(!c.unattributed);
    }

    #[test]
    fn multiple_axes_sorted_in_spec_order() {
        let mut prev = shape("s", "t");
        prev.project_instructions = sub_hash(b"old");
        prev.skills = sub_hash(b"sk");
        let mut cur = shape("s2", "t2");
        cur.project_instructions = sub_hash(b"new");
        cur.skills = sub_hash(b"sk");
        let c = PrefixChange::between(&prev, &cur, true);
        assert_eq!(c.label(), "system,tools,project_instructions");
    }

    #[test]
    fn epoch_moved_without_component_change_is_unattributed() {
        let a = shape("sys", "tools");
        let c = PrefixChange::between(&a, &a.clone(), true);
        assert!(c.unattributed);
        assert_eq!(c.label(), "unattributed");
    }

    #[test]
    fn tool_names_added_removed_changed() {
        let mut prev = shape("s", "doc-a");
        prev.tool_names.insert("read".into(), sub_hash(b"read-v1"));
        prev.tool_names
            .insert("write".into(), sub_hash(b"write-v1"));
        prev.tool_names.insert("grep".into(), sub_hash(b"grep-v1"));
        let mut cur = shape("s", "doc-b");
        cur.tool_names.insert("read".into(), sub_hash(b"read-v1"));
        cur.tool_names.insert("grep".into(), sub_hash(b"grep-v2"));
        cur.tool_names.insert("bash".into(), sub_hash(b"bash-v1"));
        let c = PrefixChange::between(&prev, &cur, true);
        assert_eq!(c.label(), "tools");
        assert_eq!(
            c.detail_label().unwrap(),
            "tools.added=bash,tools.changed=grep,tools.removed=write"
        );
    }

    #[test]
    fn tool_reorder_names_reordered() {
        let mut prev = shape("s", "doc-order-a");
        prev.tool_names.insert("read".into(), sub_hash(b"read"));
        prev.tool_names.insert("write".into(), sub_hash(b"write"));
        let mut cur = shape("s", "doc-order-b");
        cur.tool_names.insert("read".into(), sub_hash(b"read"));
        cur.tool_names.insert("write".into(), sub_hash(b"write"));
        let c = PrefixChange::between(&prev, &cur, true);
        assert_eq!(c.label(), "tools");
        assert_eq!(c.detail_label().unwrap(), "tools.reordered");
    }

    #[test]
    fn skill_names_added_removed_changed() {
        let mut prev = shape("s", "t");
        prev.skills = sub_hash(b"sk-a");
        prev.skill_names.insert("alpha".into(), sub_hash(b"a1"));
        prev.skill_names.insert("beta".into(), sub_hash(b"b1"));
        let mut cur = shape("s", "t");
        cur.skills = sub_hash(b"sk-b");
        cur.skill_names.insert("alpha".into(), sub_hash(b"a1"));
        cur.skill_names.insert("gamma".into(), sub_hash(b"g1"));
        let c = PrefixChange::between(&prev, &cur, true);
        assert_eq!(c.label(), "skills");
        assert_eq!(
            c.detail_label().unwrap(),
            "skills.added=gamma,skills.removed=beta"
        );
    }

    #[test]
    fn environment_axes_are_named_not_valued() {
        let mut prev = shape("s", "t");
        prev.environment = sub_hash(b"env-a");
        prev.environment_axes.insert("cwd".into(), sub_hash(b"/a"));
        prev.environment_axes
            .insert("os_family".into(), sub_hash(b"macos"));
        let mut cur = shape("s", "t");
        cur.environment = sub_hash(b"env-b");
        cur.environment_axes.insert("cwd".into(), sub_hash(b"/b"));
        cur.environment_axes
            .insert("os_family".into(), sub_hash(b"macos"));
        let c = PrefixChange::between(&prev, &cur, true);
        assert_eq!(c.label(), "environment");
        assert_eq!(c.detail_label().unwrap(), "environment.cwd");
        // The value itself never appears in the label.
        assert!(!c.detail_label().unwrap().contains("/a"));
        assert!(!c.detail_label().unwrap().contains("/b"));
    }

    #[test]
    fn log_block_carries_epochs_and_detail_only_when_present() {
        let prev = shape("sys-a", "t");
        let cur = shape("sys-b", "t");
        let c = PrefixChange::between(&prev, &cur, true);
        assert_eq!(
            c.log_block("aaaa", "bbbb"),
            "prefix_change=system prev=aaaa cur=bbbb"
        );
        let mut prev2 = shape("s", "t-doc");
        prev2.tool_names.insert("read".into(), sub_hash(b"read"));
        let mut cur2 = shape("s", "t-doc-2");
        cur2.tool_names.insert("read".into(), sub_hash(b"read"));
        cur2.tool_names.insert("write".into(), sub_hash(b"write"));
        let c2 = PrefixChange::between(&prev2, &cur2, true);
        assert_eq!(
            c2.log_block("aaaa", "bbbb"),
            "prefix_change=tools prev=aaaa cur=bbbb\nprefix_change_detail=tools.added=write"
        );
    }

    #[test]
    fn sub_hash_is_16_hex_chars() {
        let h = sub_hash(b"anything");
        assert_eq!(h.len(), 16);
        assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
