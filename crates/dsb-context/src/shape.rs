//! 접두 모양 — 안정 접두 귀속과 요청 단위 메타데이터
//! (spec 10 §1.5.1).
//!
//! epoch([`crate::PrefixEpoch`])은 안정 접두가 바뀌었는지를 나타낸다.
//! [`PrefixShape`]는 실제 출력된 안정 접두 구성요소의 해시를 기록한다.
//! 도구·스킬 해시는 요청 메타데이터로 남지만 해당 문서는 안정 시스템 메시지 밖에
//! 있으므로 epoch를 바꾸지 않는다.
//!
//! 모양은 관측값이다. 접두를 구성하는 문자열에서 계산하므로 `stable_prefix_bytes`를
//! 바꾸지 않는다. 요청 단위 메타데이터 해시도 관측값이다.

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

/// 출력된 안정 접두 구성요소 하나다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PrefixAxis {
    System,
    Environment,
    ProjectInstructions,
}

impl PrefixAxis {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Environment => "environment",
            Self::ProjectInstructions => "project_instructions",
        }
    }

    /// 안정 시스템 메시지 바이트의 로그 순서다.
    pub const ALL: [Self; 3] = [Self::System, Self::Environment, Self::ProjectInstructions];
}

/// 안정 접두 구성요소와 요청 단위 도구·스킬 메타데이터의 해시다.
///
/// All fields are hashes or identifiers — never instruction content, never a
/// cwd value (spec 10 §1.5.1 rule 3). An absent/empty shape means "no shape",
/// 이 뜻은 "모든 항목이 바뀜"이 아니다. `tools`, `skills`, `tool_names`,
/// `skill_names`는 안정 시스템 메시지 밖의 요청 입력을 나타낸다.
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
    /// 안정 접두 구성요소의 해시가 모두 같으면 참이다. 요청 메타데이터가 달라도
    /// 안정 접두 바이트는 같을 수 있다.
    pub fn components_equal(&self, other: &Self) -> bool {
        self.system == other.system
            && self.environment == other.environment
            && self.project_instructions == other.project_instructions
    }
}

/// What changed between two prefix shapes (spec 10 §1.5.1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrefixChange {
    /// 바뀐 안정 접두 구성요소를 Spec 10 순서로 담는다.
    pub axes: Vec<PrefixAxis>,
    /// `environment.cwd` 등 안정 메시지 세부 정보를 결정적 순서로 담는다.
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
                PrefixAxis::Environment => prev.environment != cur.environment,
                PrefixAxis::ProjectInstructions => {
                    prev.project_instructions != cur.project_instructions
                }
            })
            .collect();

        let mut detail = Vec::new();
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
        assert_eq!(c.label(), "system,project_instructions");
    }

    #[test]
    fn epoch_moved_without_component_change_is_unattributed() {
        let a = shape("sys", "tools");
        let c = PrefixChange::between(&a, &a.clone(), true);
        assert!(c.unattributed);
        assert_eq!(c.label(), "unattributed");
    }

    #[test]
    fn request_tool_and_skill_changes_do_not_move_the_prefix() {
        let mut prev = shape("s", "doc-a");
        let mut cur = shape("s", "doc-b");
        prev.tool_names.insert("read".into(), sub_hash(b"read-v1"));
        prev.skill_names
            .insert("pr-authoring".into(), sub_hash(b"Write PRs"));
        cur.tool_names.insert("write".into(), sub_hash(b"write-v1"));
        cur.skill_names
            .insert("new-skill".into(), sub_hash(b"new description"));
        assert!(prev.components_equal(&cur));
        let c = PrefixChange::between(&prev, &cur, false);
        assert_eq!(c.label(), "none");
        assert!(c.detail_label().is_none());
        assert!(!c.unattributed);
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
        let prev2 = shape("s", "t-doc");
        let cur2 = shape("s", "t-doc-2");
        let c2 = PrefixChange::between(&prev2, &cur2, false);
        assert_eq!(
            c2.log_block("aaaa", "bbbb"),
            "prefix_change=none prev=aaaa cur=bbbb"
        );
    }

    #[test]
    fn sub_hash_is_16_hex_chars() {
        let h = sub_hash(b"anything");
        assert_eq!(h.len(), 16);
        assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
