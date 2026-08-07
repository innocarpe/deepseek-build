//! Path A (default Grok agent) permissions matrix — Spec 90 spirit.
//!
//! Product entry (`dsb` → `deepseek-build-agent`) uses Grok capability modes +
//! UI `yolo` + reverse-request prompts. This module is the **product contract**
//! that maps those surfaces to Spec 90 `allow` / `deny` / `ask` and enforces
//! **headless fail-closed** (Ask → Deny when non-interactive).
//!
//! Thin Path B already implements [`PermissionPolicy`] in `permissions.rs`.
//! Heart fusion (G005) requires the same matrix to be the default story for
//! Path A configuration and tests — not YOLO-only product defaults.
//!
//! See: `docs/architecture/HEART_3X_SPEC_BINDING.md`,
//! `docs/product/HEART_3X_P0_TEST_PLAN.md` (H90.*).

use crate::permissions::{Decision, PermissionPolicy, Scope, decide, default_coding_policy};

/// Grok-aligned capability mode names (wire/docs). Product maps these to
/// tool filtering; permission **decisions** still use Spec 90 scopes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathACapabilityMode {
    /// Read/search only — no edit, no shell.
    ReadOnly,
    /// Read + edit; no shell.
    ReadWrite,
    /// Read + shell; no edit.
    Execute,
    /// Full tool surface (root session default in Grok).
    All,
}

impl PathACapabilityMode {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "read_only" | "readonly" | "read-only" => Some(Self::ReadOnly),
            "read_write" | "readwrite" | "read-write" => Some(Self::ReadWrite),
            "execute" => Some(Self::Execute),
            "all" => Some(Self::All),
            _ => None,
        }
    }

    /// Whether edit/write tools are present under this capability mode.
    pub fn allows_edit(self) -> bool {
        matches!(self, Self::ReadWrite | Self::All)
    }

    /// Whether shell/bash tools are present under this capability mode.
    pub fn allows_shell(self) -> bool {
        matches!(self, Self::Execute | Self::All)
    }
}

/// Path A product permission settings (config + runtime).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PathAPermissionSettings {
    /// Grok `[ui].yolo` — product default **false** (not YOLO-only).
    pub yolo: bool,
    /// Interactive TTY that can answer ask prompts.
    pub interactive: bool,
    /// Capability mode applied to the session toolset.
    pub capability: PathACapabilityMode,
}

impl PathAPermissionSettings {
    /// Product defaults for interactive DeepSeek TUI.
    pub const fn product_tty_default() -> Self {
        Self {
            yolo: false,
            interactive: true,
            capability: PathACapabilityMode::All,
        }
    }

    /// Product defaults for headless agent (`-p` / CI / scripts).
    pub const fn product_headless_default() -> Self {
        Self {
            yolo: false,
            interactive: false,
            capability: PathACapabilityMode::All,
        }
    }

    /// Spec 90 policy for Path A under these settings.
    ///
    /// - YOLO is **not** the product default. When `yolo` is true (explicit
    ///   user/CLI override), workspace write/delete are allowed like dogfood.
    /// - Headless always maps Ask → Deny via [`PermissionPolicy::headless`].
    pub fn policy(self) -> PermissionPolicy {
        let headless = !self.interactive;
        if self.yolo {
            // Explicit override only — still deny out-of-cwd write/delete.
            crate::permissions::dogfood_coding_policy(headless)
        } else {
            default_coding_policy(headless)
        }
    }

    /// Decide a side-effect after capability filter.
    ///
    /// If capability mode strips the tool class, result is **Deny** even when
    /// the scope policy would ask/allow (no silent elevate past capability).
    pub fn decide_action(self, scopes: &[Scope], action: PathAActionClass) -> Decision {
        match action {
            PathAActionClass::Edit | PathAActionClass::Write => {
                if !self.capability.allows_edit() {
                    return Decision::Deny;
                }
            }
            PathAActionClass::Shell => {
                if !self.capability.allows_shell() {
                    return Decision::Deny;
                }
            }
            PathAActionClass::Read => {}
        }
        decide(&self.policy(), scopes)
    }
}

/// Coarse tool class for capability × permission matrix.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathAActionClass {
    Read,
    Edit,
    Write,
    Shell,
}

/// Product seed must not enable YOLO.
pub fn product_yolo_default() -> bool {
    false
}

/// True when config body sets yolo to an explicit true.
pub fn config_enables_yolo(config_toml: &str) -> bool {
    for line in config_toml.lines() {
        let t = line.trim();
        if t.starts_with('#') {
            continue;
        }
        if let Some(rest) = t.strip_prefix("yolo") {
            let rest = rest.trim_start();
            if let Some(rest) = rest.strip_prefix('=') {
                let v = rest.trim().trim_matches('"');
                return matches!(v, "true" | "1" | "yes");
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn h90_1_headless_ask_is_deny() {
        let s = PathAPermissionSettings::product_headless_default();
        assert_eq!(
            s.decide_action(&[Scope::WriteInCwd], PathAActionClass::Edit),
            Decision::Deny
        );
        assert_eq!(
            s.decide_action(&[Scope::DeleteInCwd], PathAActionClass::Write),
            Decision::Deny
        );
        assert_eq!(
            s.decide_action(&[Scope::Network], PathAActionClass::Shell),
            Decision::Deny
        );
    }

    #[test]
    fn h90_2_tty_ask_for_write() {
        let s = PathAPermissionSettings::product_tty_default();
        assert_eq!(
            s.decide_action(&[Scope::WriteInCwd], PathAActionClass::Edit),
            Decision::Ask
        );
        assert_eq!(
            s.decide_action(&[Scope::ReadInCwd], PathAActionClass::Read),
            Decision::Allow
        );
    }

    #[test]
    fn h90_3_explicit_deny_out_of_cwd() {
        let tty = PathAPermissionSettings::product_tty_default();
        let headless = PathAPermissionSettings::product_headless_default();
        for s in [tty, headless] {
            assert_eq!(
                s.decide_action(&[Scope::WriteOutCwd], PathAActionClass::Write),
                Decision::Deny
            );
            assert_eq!(
                s.decide_action(&[Scope::DeleteOutCwd], PathAActionClass::Write),
                Decision::Deny
            );
        }
    }

    #[test]
    fn h90_4_product_default_not_yolo() {
        assert!(!product_yolo_default());
        assert!(!PathAPermissionSettings::product_tty_default().yolo);
        assert!(!PathAPermissionSettings::product_headless_default().yolo);
        // Seed-like toml
        let seed = r#"
[ui]
theme = "deepseeknight-neutral"
yolo = false
"#;
        assert!(!config_enables_yolo(seed));
        assert!(config_enables_yolo("yolo = true\n"));
    }

    #[test]
    fn h90_capability_readonly_blocks_edit_even_if_yolo() {
        let s = PathAPermissionSettings {
            yolo: true,
            interactive: true,
            capability: PathACapabilityMode::ReadOnly,
        };
        assert_eq!(
            s.decide_action(&[Scope::WriteInCwd], PathAActionClass::Edit),
            Decision::Deny
        );
    }

    #[test]
    fn h90_yolo_explicit_allows_workspace_write_still_denies_out() {
        let s = PathAPermissionSettings {
            yolo: true,
            interactive: false,
            capability: PathACapabilityMode::All,
        };
        assert_eq!(
            s.decide_action(&[Scope::WriteInCwd], PathAActionClass::Edit),
            Decision::Allow
        );
        assert_eq!(
            s.decide_action(&[Scope::WriteOutCwd], PathAActionClass::Write),
            Decision::Deny
        );
    }

    #[test]
    fn h90_capability_execute_blocks_edit() {
        let s = PathAPermissionSettings {
            yolo: false,
            interactive: true,
            capability: PathACapabilityMode::Execute,
        };
        assert_eq!(
            s.decide_action(&[Scope::WriteInCwd], PathAActionClass::Edit),
            Decision::Deny
        );
        // Shell still goes through Spec 90 (ask on TTY for network/mutate).
        assert_eq!(
            s.decide_action(&[Scope::Network], PathAActionClass::Shell),
            Decision::Ask
        );
    }
}
