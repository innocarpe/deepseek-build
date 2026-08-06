//! Skills index (stable prefix) + on-demand body load (volatile).
//!
//! Index entries are name + short description only — full SKILL.md bodies are
//! **not** embedded in the stable prefix so loading a skill mid-session does not
//! thrash the cache epoch.

use std::fs;
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::prefix::SkillIndexEntry;

#[derive(Debug, Error)]
pub enum SkillError {
    #[error("skill not found: {0}")]
    NotFound(String),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

/// Discover skill index entries from well-known roots (stable, sorted).
///
/// Search order (later roots override same name for **body** load priority,
/// but index is a union sorted by name):
/// 1. `{workspace}/skills/*/SKILL.md`
/// 2. `{workspace}/.deepseek-build/skills/*/SKILL.md`
/// 3. `{user_home}/skills/*/SKILL.md` when provided
pub fn discover_skills_index(
    workspace: &Path,
    user_skills_root: Option<&Path>,
) -> Result<Vec<SkillIndexEntry>, SkillError> {
    let mut roots: Vec<PathBuf> = vec![
        workspace.join("skills"),
        workspace.join(".deepseek-build").join("skills"),
    ];
    if let Some(u) = user_skills_root {
        roots.push(u.to_path_buf());
    }
    let mut by_name: std::collections::BTreeMap<String, SkillIndexEntry> =
        std::collections::BTreeMap::new();
    for root in roots {
        if !root.is_dir() {
            continue;
        }
        for ent in fs::read_dir(&root)? {
            let ent = ent?;
            let path = ent.path();
            if !path.is_dir() {
                continue;
            }
            let skill_md = path.join("SKILL.md");
            if !skill_md.is_file() {
                continue;
            }
            let name = path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
            if name.is_empty() || name.starts_with('.') {
                continue;
            }
            let raw = fs::read_to_string(&skill_md)?;
            if skill_opts_out_of_index(&raw) {
                // Still overridable: an later root without opt-out may re-insert.
                by_name.remove(&name);
                continue;
            }
            let description = extract_description(&raw);
            by_name.insert(name.clone(), SkillIndexEntry { name, description });
        }
    }
    Ok(by_name.into_values().collect())
}

/// Frontmatter `disable-model-invocation` / `disable_model_invocation` truthy → omit from index.
fn skill_opts_out_of_index(raw: &str) -> bool {
    let text = raw.trim();
    if !text.starts_with("---") {
        return false;
    }
    let Some(rest) = text.strip_prefix("---") else {
        return false;
    };
    let Some(end) = rest.find("\n---") else {
        return false;
    };
    let fm = &rest[..end];
    for line in fm.lines() {
        let line = line.trim();
        let lower = line.to_ascii_lowercase();
        if lower.starts_with("disable-model-invocation:")
            || lower.starts_with("disable_model_invocation:")
        {
            let v = line
                .split_once(':')
                .map(|(_, v)| {
                    v.trim()
                        .trim_matches('"')
                        .trim_matches('\'')
                        .to_ascii_lowercase()
                })
                .unwrap_or_default();
            return matches!(v.as_str(), "true" | "yes" | "1");
        }
    }
    false
}

/// Load full skill body by name (on-demand; does not affect stable prefix).
pub fn load_skill_body(
    workspace: &Path,
    user_skills_root: Option<&Path>,
    name: &str,
) -> Result<String, SkillError> {
    if name.is_empty() || name.contains("..") || name.contains('/') || name.contains('\\') {
        return Err(SkillError::NotFound(name.to_string()));
    }
    let mut candidates = vec![
        workspace.join("skills").join(name).join("SKILL.md"),
        workspace
            .join(".deepseek-build")
            .join("skills")
            .join(name)
            .join("SKILL.md"),
    ];
    if let Some(u) = user_skills_root {
        candidates.push(u.join(name).join("SKILL.md"));
    }
    // Prefer user override last in list — search reverse so user wins
    for path in candidates.into_iter().rev() {
        if path.is_file() {
            return Ok(fs::read_to_string(path)?);
        }
    }
    Err(SkillError::NotFound(name.to_string()))
}

/// Short description for index: YAML frontmatter `description:` or first non-heading prose line.
fn extract_description(raw: &str) -> String {
    let text = raw.trim();
    if text.starts_with("---")
        && let Some(rest) = text.strip_prefix("---")
        && let Some(end) = rest.find("\n---")
    {
        let fm = &rest[..end];
        for line in fm.lines() {
            let line = line.trim();
            if let Some(v) = line.strip_prefix("description:") {
                let v = v.trim().trim_matches('"').trim_matches('\'');
                if !v.is_empty() {
                    return truncate(v, 200);
                }
            }
        }
    }
    for line in text.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('#') || t.starts_with("---") {
            continue;
        }
        return truncate(t, 200);
    }
    "(no description)".into()
}

fn truncate(s: &str, max: usize) -> String {
    let mut out = String::new();
    for (i, ch) in s.chars().enumerate() {
        if i >= max {
            out.push('…');
            break;
        }
        out.push(ch);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn discovers_index_without_body_in_entry() {
        let dir = tempdir().unwrap();
        let skill_dir = dir.path().join("skills").join("demo-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            "---\nname: demo-skill\ndescription: \"Does a thing\"\n---\n\n# Demo\n\nLong body that must not live in the stable prefix index.\n",
        )
        .unwrap();
        let idx = discover_skills_index(dir.path(), None).unwrap();
        assert_eq!(idx.len(), 1);
        assert_eq!(idx[0].name, "demo-skill");
        assert!(idx[0].description.contains("Does a thing"));
        assert!(!idx[0].description.contains("Long body"));
    }

    #[test]
    fn load_body_on_demand() {
        let dir = tempdir().unwrap();
        let skill_dir = dir.path().join("skills").join("x");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(skill_dir.join("SKILL.md"), "# X\n\nBODY_MARKER\n").unwrap();
        let body = load_skill_body(dir.path(), None, "x").unwrap();
        assert!(body.contains("BODY_MARKER"));
        assert!(load_skill_body(dir.path(), None, "../etc").is_err());
    }

    #[test]
    fn opt_out_excluded_from_index() {
        let dir = tempdir().unwrap();
        let hidden = dir.path().join("skills").join("hidden");
        let visible = dir.path().join("skills").join("visible");
        fs::create_dir_all(&hidden).unwrap();
        fs::create_dir_all(&visible).unwrap();
        fs::write(
            hidden.join("SKILL.md"),
            "---\ndescription: secret\ndisable-model-invocation: true\n---\n\n# Hidden\n",
        )
        .unwrap();
        fs::write(
            visible.join("SKILL.md"),
            "---\ndescription: public\n---\n\n# Visible\n",
        )
        .unwrap();
        let idx = discover_skills_index(dir.path(), None).unwrap();
        assert_eq!(idx.len(), 1);
        assert_eq!(idx[0].name, "visible");
    }

    #[test]
    fn user_root_overrides_project_body() {
        let dir = tempdir().unwrap();
        let user = tempdir().unwrap();
        let proj = dir.path().join("skills").join("shared");
        let usr = user.path().join("shared");
        fs::create_dir_all(&proj).unwrap();
        fs::create_dir_all(&usr).unwrap();
        fs::write(proj.join("SKILL.md"), "# Project\nPROJECT_BODY\n").unwrap();
        fs::write(usr.join("SKILL.md"), "# User\nUSER_BODY\n").unwrap();
        let body = load_skill_body(dir.path(), Some(user.path()), "shared").unwrap();
        assert!(body.contains("USER_BODY"));
        assert!(!body.contains("PROJECT_BODY"));
    }

    #[test]
    fn index_sorted_by_name() {
        let dir = tempdir().unwrap();
        for name in ["zeta", "alpha", "mid"] {
            let p = dir.path().join("skills").join(name);
            fs::create_dir_all(&p).unwrap();
            fs::write(
                p.join("SKILL.md"),
                format!("---\ndescription: {name}\n---\n"),
            )
            .unwrap();
        }
        let idx = discover_skills_index(dir.path(), None).unwrap();
        let names: Vec<_> = idx.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, vec!["alpha", "mid", "zeta"]);
    }
}
