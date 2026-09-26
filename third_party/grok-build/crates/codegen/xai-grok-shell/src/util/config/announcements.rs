use toml::Value as TomlValue;
use xai_grok_config_types::RemoteSettings;

/// Announcement entry received from cli-chat-proxy `/v1/settings`.
pub use xai_grok_announcements::RemoteAnnouncement;

/// Removes the remote announcements from settings before they enter product state.
///
/// DeepSeek Build does not carry xAI/Grok announcements, so every writer of
/// `cfg.remote_settings` funnels through this: the stored settings never hold
/// an announcement and the announcements push gate has nothing to seed clients
/// with. The display side holds the same line in [`resolve_announcements`];
/// `GROK_ANNOUNCEMENTS_OVERRIDE` and the config TOML layers are untouched.
/// See docs/architecture/GROK_VENDOR.md.
pub fn strip_remote_announcements(settings: &mut RemoteSettings) {
    settings.announcements = None;
}

// ---------------------------------------------------------------------------
// Announcements & tips from TOML
// ---------------------------------------------------------------------------

/// Parse `announcements` from a TOML value (inline tables or array-of-tables).
pub(crate) fn announcements_from_toml(root: &TomlValue) -> Vec<RemoteAnnouncement> {
    root.get("announcements")
        .and_then(|v| v.clone().try_into::<Vec<RemoteAnnouncement>>().ok())
        .unwrap_or_default()
}

/// Merge announcement slices in priority order. Dedup by `id`; first wins.
pub(crate) fn merge_announcements(sources: &[&[RemoteAnnouncement]]) -> Vec<RemoteAnnouncement> {
    let mut seen = std::collections::HashSet::<String>::new();
    let mut out = Vec::new();
    for source in sources {
        for a in *source {
            if let Some(ref id) = a.id
                && !seen.insert(id.clone())
            {
                continue;
            }
            out.push(a.clone());
        }
    }
    out
}

/// Dev/test override for announcements via `GROK_ANNOUNCEMENTS_OVERRIDE` (a JSON array of announcements).
/// Returns `Some` only when the env var holds valid JSON; an empty array (`[]`) suppresses all announcements.
/// Every announcement resolution path honors this so it works for testing regardless of source.
pub(crate) fn announcements_override() -> Option<Vec<RemoteAnnouncement>> {
    let raw = std::env::var("GROK_ANNOUNCEMENTS_OVERRIDE").ok()?;
    match serde_json::from_str::<Vec<RemoteAnnouncement>>(&raw) {
        Ok(list) => Some(list),
        Err(_) => {
            tracing::warn!("invalid GROK_ANNOUNCEMENTS_OVERRIDE JSON; ignoring");
            None
        }
    }
}

/// Priority order: requirements, then user config, then managed config.
/// The `GROK_ANNOUNCEMENTS_OVERRIDE` env var overrides everything (dev only).
///
/// The `remote` layer is deliberately not merged: DeepSeek Build does not put
/// remote xAI announcements on screen, so a backend push can never reintroduce
/// them. The parameter stays accepted for callers; the receive boundary strips
/// them first (`strip_remote_announcements`).
/// See docs/architecture/GROK_VENDOR.md.
pub fn resolve_announcements(
    requirements: Option<&TomlValue>,
    user: Option<&TomlValue>,
    managed: Option<&TomlValue>,
    _remote: Option<&[RemoteAnnouncement]>,
) -> Vec<RemoteAnnouncement> {
    if let Some(list) = announcements_override() {
        return list;
    }

    let req = requirements
        .map(announcements_from_toml)
        .unwrap_or_default();
    let usr = user.map(announcements_from_toml).unwrap_or_default();
    let mgd = managed.map(announcements_from_toml).unwrap_or_default();

    merge_announcements(&[&req, &usr, &mgd])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ann(id: &str) -> RemoteAnnouncement {
        RemoteAnnouncement {
            id: Some(id.to_string()),
            message: Some(format!("{id}-msg")),
            ..Default::default()
        }
    }

    /// The remote layer never merges: a backend push must not reach the display set.
    /// Serial: a concurrently running override test would leak its env into this one.
    #[test]
    #[serial_test::serial]
    fn resolve_announcements_ignores_the_remote_layer() {
        let user: TomlValue = toml::from_str(
            r#"
            [[announcements]]
            id = "local"
            message = "from user config"
            "#,
        )
        .unwrap();
        let resolved = resolve_announcements(None, Some(&user), None, Some(&[ann("remote")]));
        let ids: Vec<_> = resolved.iter().filter_map(|a| a.id.as_deref()).collect();
        assert_eq!(ids, ["local"], "the remote layer must not merge in");
    }

    /// The dev/test override still beats everything, including a remote list.
    #[test]
    #[serial_test::serial]
    fn resolve_announcements_override_seed_still_wins() {
        let _env = crate::env::EnvVarGuard::set(
            "GROK_ANNOUNCEMENTS_OVERRIDE",
            r#"[{"id":"seed","message":"from override"}]"#,
        );
        let resolved = resolve_announcements(None, None, None, Some(&[ann("remote")]));
        let ids: Vec<_> = resolved.iter().filter_map(|a| a.id.as_deref()).collect();
        assert_eq!(ids, ["seed"], "the override seed must keep working");
    }

    #[test]
    fn strip_remote_announcements_clears_the_field() {
        let mut settings = RemoteSettings {
            announcements: Some(vec![ann("grok-4.7")]),
            ..Default::default()
        };
        strip_remote_announcements(&mut settings);
        assert_eq!(settings.announcements, None);
    }
}
