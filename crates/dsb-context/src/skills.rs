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
            let description = extract_description(&raw);
            by_name.insert(
                name.clone(),
                SkillIndexEntry {
                    name,
                    description,
                },
            );
        }
    }
    Ok(by_name.into_values().collect())
}

/// Load full skill body by name (on-demand; does not affect stable prefix).
pub fn load_skill_body(
    workspace: &Path,
    user_skills_root: Option<&Path>,
    name: &str,
) -> Result<String, SkillError> {
    if name.is_empty()
        || name.contains("..")
        || name.contains('/')
        || name.contains('\\')
    {
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
    if text.starts_with("---") {
        if let Some(rest) = text.strip_prefix("---") {
            if let Some(end) = rest.find("\n---") {
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
}
