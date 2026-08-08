//! Path A V2-cache signal stamp (VC009).
//!
//! When product Path A launches with `DEEPSEEK_BUILD_HOME`, each turn that
//! receives provider usage writes a fail-soft stamp file so hermetic R0A can
//! assert a **loggable** cache-hit signal without driving the TUI status row.
//!
//! User-visible path remains the pager `format_cache_hit_pct` chip (shipped).

use xai_grok_sampling_types::TokenUsage;

/// Best-effort stamp of DeepSeek/prompt-cache usage under product home.
///
/// Writes `path_a_cache_signal.txt` when `DEEPSEEK_BUILD_HOME` (or an explicit
/// `stamp_dir`) is set. Failures never block the turn.
pub fn stamp_path_a_cache_signal(usage: &TokenUsage, stamp_dir: Option<&std::path::Path>) {
    let dir = stamp_dir
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::var_os("DEEPSEEK_BUILD_HOME").map(std::path::PathBuf::from));
    let Some(dir) = dir else {
        return;
    };

    let prompt = u64::from(usage.prompt_tokens);
    let cached = u64::from(usage.cached_prompt_tokens);
    let completion = u64::from(usage.completion_tokens);
    let pct = if prompt == 0 {
        None
    } else {
        let clamped = cached.min(prompt);
        Some(((clamped * 100) + (prompt / 2)) / prompt)
    };
    let chip = pct
        .map(|p| format!("cache {p}%"))
        .unwrap_or_else(|| "cache n/a".to_string());

    let path = dir.join("path_a_cache_signal.txt");
    let body = format!(
        "path_a_cache_signal=present\n\
prompt_tokens={prompt}\n\
cached_prompt_tokens={cached}\n\
completion_tokens={completion}\n\
cache_hit_pct={pct}\n\
cache_chip={chip}\n\
source=path_a_turn_usage\n",
        pct = pct
            .map(|p| p.to_string())
            .unwrap_or_else(|| "none".to_string()),
    );
    let _ = std::fs::create_dir_all(&dir);
    let _ = std::fs::write(path, body);
}

#[cfg(test)]
mod tests {
    use super::*;
    use xai_grok_sampling_types::TokenUsage;

    #[test]
    fn stamp_writes_cache_chip_for_partial_hit() {
        let dir = tempfile::tempdir().expect("tempdir");
        let usage = TokenUsage {
            prompt_tokens: 100,
            completion_tokens: 5,
            total_tokens: 105,
            reasoning_tokens: 0,
            cached_prompt_tokens: 80,
            cache_creation_prompt_tokens: 0,
        };
        stamp_path_a_cache_signal(&usage, Some(dir.path()));
        let body = std::fs::read_to_string(dir.path().join("path_a_cache_signal.txt"))
            .expect("stamp written");
        assert!(body.contains("path_a_cache_signal=present"));
        assert!(body.contains("cached_prompt_tokens=80"));
        assert!(body.contains("prompt_tokens=100"));
        assert!(body.contains("cache_hit_pct=80"));
        assert!(body.contains("cache_chip=cache 80%"));
    }

    #[test]
    fn stamp_hides_pct_when_prompt_zero() {
        let dir = tempfile::tempdir().expect("tempdir");
        let usage = TokenUsage {
            prompt_tokens: 0,
            completion_tokens: 1,
            total_tokens: 1,
            reasoning_tokens: 0,
            cached_prompt_tokens: 0,
            cache_creation_prompt_tokens: 0,
        };
        stamp_path_a_cache_signal(&usage, Some(dir.path()));
        let body = std::fs::read_to_string(dir.path().join("path_a_cache_signal.txt"))
            .expect("stamp written");
        assert!(body.contains("cache_hit_pct=none"));
        assert!(body.contains("cache_chip=cache n/a"));
    }
}
