//! Session-local Spec 45 snippet table for Path A (Grok tool path).
//!
//! ## Ownership (fail-close)
//!
//! Inserted only into a **per-session** [`super::resources::Resources`] bag
//! (typically behind that session's [`super::resources::SharedResources`] =
//! `Arc<Mutex<Resources>>`) via `get_or_default`. There is **no** `static`,
//! `lazy_static`, process-global map, or `register_state` persistence path —
//! the table dies with the session resources and must never enter Spec 10
//! stable-prefix bytes.
//!
//! - **VC003:** issue on successful **UTF-8 text** `read_file` only.
//! - **VC004:** edit require against this store (Path A `search_replace`).
//! - **VC005:** `expire_path` / `expire_all` + bash mutation classification.
//! - **VC006:** resume/fork restore remains out of scope.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Opaque session snippet record (ADR 0010 §2 shape; Path A host).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionSnippet {
    pub snippet_id: String,
    pub path: PathBuf,
    pub start_line: usize,
    pub end_line: usize,
    /// Full-file `hex(sha256(bytes))` at issue time (alias of tool `file_version`).
    pub version: String,
    /// `lines` or `whole_file`.
    pub scope: String,
    pub preview: String,
    pub encoding: String,
    /// Audit-only issuance counter within this store (not Spec 10 / cache).
    pub issued_at_turn: u64,
}

/// Session-owned snippet table. Ephemeral; discarded with the session Resources.
///
/// Constructed per `Resources` instance (one bag per agent session /
/// `SharedResources`). Not process-global.
#[derive(Debug, Default)]
pub struct SessionSnippetStore {
    by_id: HashMap<String, SessionSnippet>,
    /// Monotonic issuance counter used as `issued_at_turn` until a real turn
    /// index is plumbed (audit only).
    mint_seq: u64,
}

/// Preview cap matching thin `dsb-tools` / ADR 0010 (Unicode scalars + ellipsis).
pub const SNIPPET_PREVIEW_MAX_SCALARS: usize = 200;

/// Crockford Base32 alphabet (ULID / ADR 0010 §2). No I, L, O, U.
/// Exact character set required for the 26-char ULID suffix after `snp_`.
pub const CROCKFORD_ALPHABET: &str = "0123456789ABCDEFGHJKMNPQRSTVWXYZ";
const CROCKFORD: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

impl SessionSnippetStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, id: &str) -> Option<&SessionSnippet> {
        self.by_id.get(id)
    }

    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }

    pub fn contains(&self, id: &str) -> bool {
        self.by_id.contains_key(id)
    }

    /// Expire (remove) all snippets bound to `path` (ADR 0010 §6).
    ///
    /// Path compare is exact PathBuf equality first, then host
    /// canonicalize when both sides resolve (same spirit as Path A edit
    /// path binding).
    pub fn expire_path(&mut self, path: &Path) {
        self.by_id
            .retain(|_, s| !snippet_paths_equal(path, &s.path));
    }

    /// Expire every session file snippet (unknown bash mutation / M2 default).
    pub fn expire_all(&mut self) {
        self.by_id.clear();
    }

    /// Apply a bash mutation invalidation plan (VC005 / ADR 0010 §6.2).
    pub fn apply_bash_expire_plan(&mut self, plan: &BashSnippetExpirePlan) {
        match plan {
            BashSnippetExpirePlan::None => {}
            BashSnippetExpirePlan::Paths(paths) => {
                for p in paths {
                    self.expire_path(p);
                }
            }
            BashSnippetExpirePlan::All => self.expire_all(),
        }
    }

    /// Mint a new opaque `snippet_id` for a successful UTF-8 text read.
    ///
    /// Repeated calls always insert a **new** id (ADR 0010 §2 Multiple IDs).
    pub fn issue(
        &mut self,
        path: &Path,
        start_line: usize,
        end_line: usize,
        version: impl Into<String>,
        scope_text: &str,
        total_lines: usize,
    ) -> SessionSnippet {
        let start_line = start_line.max(1);
        let end_line = end_line.max(start_line);
        let version = version.into();
        let whole_file = start_line == 1 && end_line >= total_lines.max(1);
        let scope = if whole_file {
            "whole_file".to_string()
        } else {
            "lines".to_string()
        };
        self.mint_seq = self.mint_seq.saturating_add(1);
        let snippet = SessionSnippet {
            snippet_id: new_snippet_id(),
            path: path.to_path_buf(),
            start_line,
            end_line,
            version,
            scope,
            preview: truncate_preview(scope_text, SNIPPET_PREVIEW_MAX_SCALARS),
            encoding: "utf-8".to_string(),
            issued_at_turn: self.mint_seq,
        };
        self.by_id
            .insert(snippet.snippet_id.clone(), snippet.clone());
        snippet
    }
}

/// ADR 0010 §2: `snp_` + Crockford-base32 ULID (26 chars).
///
/// Local encoder (no extra crate): 48-bit ms timestamp + 80-bit entropy from
/// existing workspace `uuid` randomness, packed as a 128-bit ULID and encoded
/// with the Crockford alphabet.
pub fn new_snippet_id() -> String {
    format!("snp_{}", new_ulid_crockford())
}

/// Generate a 26-character Crockford Base32 ULID string.
pub fn new_ulid_crockford() -> String {
    let ts_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
        & 0x0000_FFFF_FFFF_FFFF; // 48 bits
    // 10 bytes entropy via existing uuid dependency (no new crates).
    let entropy = {
        let b = uuid::Uuid::new_v4().into_bytes();
        let mut e = [0u8; 10];
        e.copy_from_slice(&b[0..10]);
        e
    };
    encode_ulid_crockford(ts_ms, &entropy)
}

/// Encode `(timestamp_ms_48bit, entropy_80bit)` as a 26-char Crockford ULID.
pub fn encode_ulid_crockford(timestamp_ms: u64, entropy10: &[u8; 10]) -> String {
    let mut bytes = [0u8; 16];
    // Big-endian 48-bit timestamp into first 6 bytes.
    bytes[0] = ((timestamp_ms >> 40) & 0xff) as u8;
    bytes[1] = ((timestamp_ms >> 32) & 0xff) as u8;
    bytes[2] = ((timestamp_ms >> 24) & 0xff) as u8;
    bytes[3] = ((timestamp_ms >> 16) & 0xff) as u8;
    bytes[4] = ((timestamp_ms >> 8) & 0xff) as u8;
    bytes[5] = (timestamp_ms & 0xff) as u8;
    bytes[6..16].copy_from_slice(entropy10);

    // Standard ULID encode: 128-bit BE integer → 26×5-bit Crockford chars
    // (26*5=130; top 2 bits are zero padding on the left).
    let mut value = u128::from_be_bytes(bytes);
    let mut chars = [0u8; 26];
    for i in (0..26).rev() {
        chars[i] = CROCKFORD[(value & 0x1f) as usize];
        value >>= 5;
    }
    // SAFETY: CROCKFORD is ASCII.
    String::from_utf8(chars.to_vec()).expect("crockford alphabet is ascii")
}

/// True when `id` matches ADR 0010 Path A shape: `snp_` + 26 Crockford chars.
pub fn is_valid_snippet_id(id: &str) -> bool {
    let Some(rest) = id.strip_prefix("snp_") else {
        return false;
    };
    rest.len() == 26
        && rest.bytes().all(|b| {
            let u = b.to_ascii_uppercase();
            CROCKFORD.contains(&u)
        })
}

fn truncate_preview(text: &str, max_scalars: usize) -> String {
    let mut out = String::new();
    let mut count = 0usize;
    for ch in text.chars() {
        if count >= max_scalars {
            out.push('…');
            break;
        }
        out.push(ch);
        count += 1;
    }
    out
}

/// Inclusive 1-based line range covered by a read window.
pub fn snippet_line_range(
    total_lines: usize,
    offset: Option<usize>,
    limit: Option<usize>,
) -> (usize, usize) {
    if total_lines == 0 {
        return (1, 1);
    }
    let start = offset.unwrap_or(1).max(1);
    let start = start.min(total_lines);
    let end = match limit {
        Some(lim) if lim > 0 => start.saturating_add(lim).saturating_sub(1).min(total_lines),
        _ => total_lines,
    };
    let end = end.max(start);
    (start, end)
}

/// Compare two paths the same way Path A edit authorization does.
///
/// Prefer live `canonicalize` when the path still exists. When either side is
/// already gone (post-`rm` expire), fall back to a stable absolute/lex form so
/// mint-time `/private/tmp/…` still matches extract-time `/tmp/…` on macOS.
pub fn snippet_paths_equal(a: &Path, b: &Path) -> bool {
    if a == b {
        return true;
    }
    let ca = canonicalize_or_absolute(a);
    let cb = canonicalize_or_absolute(b);
    if ca == cb {
        return true;
    }
    // Last resort: strip known macOS private prefix asymmetry.
    normalize_path_key(&ca) == normalize_path_key(&cb)
}

/// Absolute path suitable for snippet binding / expire after the file may be gone.
pub fn canonicalize_or_absolute(path: &Path) -> PathBuf {
    if let Ok(c) = std::fs::canonicalize(path) {
        return c;
    }
    if path.is_absolute() {
        return path.to_path_buf();
    }
    std::env::current_dir()
        .map(|cwd| cwd.join(path))
        .unwrap_or_else(|_| path.to_path_buf())
}

fn normalize_path_key(path: &Path) -> String {
    let s = path.to_string_lossy();
    // macOS: /var -> /private/var, /tmp -> /private/tmp when canonicalized.
    let stripped = s
        .strip_prefix("/private/tmp")
        .map(|rest| format!("/tmp{rest}"))
        .or_else(|| {
            s.strip_prefix("/private/var")
                .map(|rest| format!("/var{rest}"))
        })
        .unwrap_or_else(|| s.into_owned());
    stripped
}

/// VC005 / ADR 0010 §6.2 plan after a dispatched bash (or equivalent) command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BashSnippetExpirePlan {
    /// Read-only / non-mutating — leave the table alone.
    None,
    /// Known touched paths — expire only those.
    Paths(Vec<PathBuf>),
    /// May mutate files but path set is unknown — expire all session snippets.
    All,
}

/// Classify a shell command for session-snippet invalidation (Path A M2).
///
/// Heuristic only (not a full shell parser). Fail-closed: when mutation is
/// possible but paths cannot be extracted, returns [`BashSnippetExpirePlan::All`].
///
/// `cwd` is used to resolve relative path tokens found in the command.
pub fn bash_snippet_expire_plan(command: &str, cwd: &Path) -> BashSnippetExpirePlan {
    if !bash_command_may_mutate_files(command) {
        return BashSnippetExpirePlan::None;
    }
    let paths = extract_bash_touched_paths(command, cwd);
    if paths.is_empty() {
        BashSnippetExpirePlan::All
    } else {
        BashSnippetExpirePlan::Paths(paths)
    }
}

/// True when the command is treated as potentially file-mutating for Spec 45.
pub fn bash_command_may_mutate_files(command: &str) -> bool {
    let lower = command.to_ascii_lowercase();
    let trimmed = lower.trim();
    if trimmed.is_empty() {
        return false;
    }

    // Explicit write / delete / move / permission mutators.
    if contains_any(
        &lower,
        &[
            "rm ",
            "rm\t",
            "rmdir ",
            "unlink ",
            "mv ",
            "cp ",
            "tee ",
            "touch ",
            "mkdir ",
            "chmod ",
            "chown ",
            "truncate ",
            "install ",
            "sed -i",
            "perl -i",
            "dd ",
        ],
    ) {
        return true;
    }
    // File-writing redirects only — ignore fd-only forms (`2>&1`, `>&2`) and
    // redirects into `/dev/null` so harmless stderr capture does not expire_all.
    if command_has_file_writing_redirect(command) {
        return true;
    }
    // Git mutations.
    if lower.contains("git ")
        && contains_any(
            &lower,
            &[
                "git commit",
                "git push",
                "git rebase",
                "git reset",
                "git tag ",
                "git merge",
                "git cherry-pick",
                "git clean",
                "git checkout",
                "git restore",
                "git stash",
                "git apply",
                "git am ",
            ],
        )
    {
        return true;
    }
    if lower.contains("sudo ") {
        return true;
    }

    // Known-safe read-ish tools (no redirect handled above).
    if is_bash_read_only_command(trimmed) {
        return false;
    }

    // Unrecognized → unknown mutation set → fail closed (caller expire_all).
    true
}

fn is_bash_read_only_command(trimmed_lower: &str) -> bool {
    // First token only. Redirects / mutator phrases are handled by the caller.
    let first = trimmed_lower
        .split(|c: char| c.is_whitespace() || c == '|')
        .find(|t| !t.is_empty())
        .unwrap_or("");
    matches!(
        first,
        "ls" | "cat"
            | "head"
            | "tail"
            | "pwd"
            | "echo"
            | "true"
            | "false"
            | "which"
            | "type"
            | "file"
            | "stat"
            | "wc"
            | "rg"
            | "grep"
            | "find"
            | "git"
            | "sleep"
            | "date"
            | "whoami"
            | "id"
            | "env"
            | "printenv"
            | "uname"
            | "basename"
            | "dirname"
            | "realpath"
            | "readlink"
            | "test"
            | "["
    )
}

fn contains_any(hay: &str, needles: &[&str]) -> bool {
    needles.iter().any(|n| hay.contains(n))
}

/// True when `command` has a redirect that may write a real filesystem path.
///
/// Non-mutating: `2>&1`, `>&2`, `>/dev/null`, `2>/dev/null`.
fn command_has_file_writing_redirect(command: &str) -> bool {
    let bytes = command.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] != b'>' {
            i += 1;
            continue;
        }
        // `N>` forms: optional leading digits already before `>`.
        let mut j = i + 1;
        if j < bytes.len() && bytes[j] == b'>' {
            // `>>`
            j += 1;
        } else if j < bytes.len() && bytes[j] == b'&' {
            // `>&1` / `2>&1` — fd dup, not a file write.
            i = j + 1;
            continue;
        }
        while j < bytes.len() && bytes[j].is_ascii_whitespace() {
            j += 1;
        }
        if let Some((tok, _next)) = take_shell_token(&command[j..]) {
            let t = tok.trim().trim_matches(|c| c == '\'' || c == '"');
            if is_non_file_redirect_target(t) {
                i = j + 1;
                continue;
            }
            // Real path target (or unknown token) — treat as file-writing.
            return true;
        }
        // Bare `>` with no token — fail closed as potential mutation.
        return true;
    }
    false
}

fn is_non_file_redirect_target(tok: &str) -> bool {
    matches!(
        tok,
        "/dev/null" | "/dev/stdout" | "/dev/stderr" | "/dev/fd/1" | "/dev/fd/2" | "1" | "2" | "0"
    )
}

/// Best-effort path token extraction for expire_path (known set).
///
/// Returns absolute or cwd-joined paths. Empty means "unknown set".
pub fn extract_bash_touched_paths(command: &str, cwd: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = Vec::new();
    let mut seen = std::collections::HashSet::new();

    // Capture targets of `> file`, `>> file`, `2> file` (not fd dups).
    let bytes = command.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'>' {
            let mut j = i + 1;
            if j < bytes.len() && bytes[j] == b'>' {
                j += 1;
            } else if j < bytes.len() && bytes[j] == b'&' {
                // fd dup — skip
                i = j + 1;
                continue;
            }
            while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            if let Some((tok, next)) = take_shell_token(&command[j..]) {
                if !is_non_file_redirect_target(tok.trim().trim_matches(|c| c == '\'' || c == '"'))
                {
                    push_path_token(tok, cwd, &mut out, &mut seen);
                }
                i = j + next;
                continue;
            }
        }
        i += 1;
    }

    // Mutator verbs: take following path-like tokens.
    for (idx, tok) in shell_tokens(command).into_iter().enumerate() {
        let t = tok.as_str();
        let tl = t.to_ascii_lowercase();
        let is_mutator = matches!(
            tl.as_str(),
            "rm" | "rmdir"
                | "unlink"
                | "mv"
                | "cp"
                | "tee"
                | "touch"
                | "mkdir"
                | "chmod"
                | "chown"
                | "truncate"
                | "install"
        );
        if !is_mutator {
            continue;
        }
        // Collect subsequent non-flag tokens as candidate paths.
        for later in shell_tokens(command).into_iter().skip(idx + 1) {
            if later.starts_with('-') {
                continue;
            }
            if later.contains('=') && !later.contains('/') {
                continue;
            }
            // Stop at next obvious verb-ish token.
            let ll = later.to_ascii_lowercase();
            if matches!(
                ll.as_str(),
                "rm" | "mv" | "cp" | "&&" | "||" | "|" | ";" | "then" | "do" | "fi"
            ) {
                break;
            }
            push_path_token(&later, cwd, &mut out, &mut seen);
        }
    }

    out
}

fn push_path_token(
    tok: &str,
    cwd: &Path,
    out: &mut Vec<PathBuf>,
    seen: &mut std::collections::HashSet<String>,
) {
    let tok = tok.trim().trim_matches(|c| c == '\'' || c == '"');
    if tok.is_empty() || tok == "-" || tok == "/dev/null" {
        return;
    }
    // Skip pure options and shell operators.
    if tok.starts_with('-')
        || matches!(tok, "&&" | "||" | "|" | ";" | "&" | "2" | "1" | "0")
        || tok.contains('*')
        || tok.contains('?')
        || tok.contains('[')
    {
        return;
    }
    // Require path-ish: has / or .ext or bare relative filename with a letter.
    let pathish = tok.contains('/')
        || tok.contains('\\')
        || tok.starts_with('.')
        || tok.contains('.')
        || tok
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.');
    if !pathish {
        return;
    }
    let p = if Path::new(tok).is_absolute() {
        PathBuf::from(tok)
    } else {
        cwd.join(tok)
    };
    // Prefer the same absolute/lex form mint uses so expire_path matches after rm.
    let p = canonicalize_or_absolute(&p);
    let key = normalize_path_key(&p);
    if seen.insert(key) {
        out.push(p);
    }
}

fn shell_tokens(command: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = command;
    while let Some((tok, consumed)) = take_shell_token(rest) {
        out.push(tok.to_string());
        rest = &rest[consumed..];
        rest = rest.trim_start();
        if rest.is_empty() {
            break;
        }
    }
    out
}

/// Returns (token, bytes_consumed_from_s including leading whitespace skip inside?).
fn take_shell_token(s: &str) -> Option<(&str, usize)> {
    let s_trim_start = s.trim_start();
    let lead = s.len() - s_trim_start.len();
    if s_trim_start.is_empty() {
        return None;
    }
    let bytes = s_trim_start.as_bytes();
    if bytes[0] == b'\'' || bytes[0] == b'"' {
        let quote = bytes[0];
        let mut i = 1;
        while i < bytes.len() {
            if bytes[i] == quote {
                let tok = &s_trim_start[1..i];
                return Some((tok, lead + i + 1));
            }
            if bytes[i] == b'\\' && i + 1 < bytes.len() {
                i += 2;
                continue;
            }
            i += 1;
        }
        // Unclosed quote — take rest.
        return Some((&s_trim_start[1..], lead + s_trim_start.len()));
    }
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        if c.is_ascii_whitespace() || matches!(c, b'|' | b';' | b'&' | b'>' | b'<') {
            break;
        }
        i += 1;
    }
    if i == 0 {
        // operator token
        let mut j = 1;
        while j < bytes.len() && matches!(bytes[j], b'|' | b'&' | b'>' | b'<') {
            j += 1;
        }
        return Some((&s_trim_start[..j], lead + j));
    }
    Some((&s_trim_start[..i], lead + i))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn issue_mints_snp_ulid_crockford_and_stores() {
        let mut store = SessionSnippetStore::new();
        let snip = store.issue(Path::new("/tmp/a.txt"), 1, 2, "ab".repeat(32), "hello\n", 2);
        assert!(
            is_valid_snippet_id(&snip.snippet_id),
            "ADR 0010 requires snp_ + Crockford ULID; got {}",
            snip.snippet_id
        );
        assert_eq!(snip.snippet_id.len(), 4 + 26);
        assert_eq!(snip.scope, "whole_file");
        assert_eq!(store.len(), 1);
        assert!(store.contains(&snip.snippet_id));
    }

    #[test]
    fn encode_ulid_known_vector() {
        // timestamp 0, entropy all zero → all Crockford zeros.
        let s = encode_ulid_crockford(0, &[0u8; 10]);
        assert_eq!(s, "00000000000000000000000000");
        assert_eq!(s.len(), 26);
    }

    #[test]
    fn encode_ulid_timestamp_only_is_sortable_prefix() {
        let a = encode_ulid_crockford(1, &[0u8; 10]);
        let b = encode_ulid_crockford(2, &[0u8; 10]);
        assert!(a < b, "ULID time component must sort lexicographically");
    }

    #[test]
    fn new_snippet_id_matches_adr_shape() {
        let id = new_snippet_id();
        assert!(id.starts_with("snp_"), "got {id}");
        let suffix = id.strip_prefix("snp_").expect("snp_ prefix");
        // ADR 0010 §2: suffix is exactly 26 Crockford-base32 chars.
        assert_eq!(
            suffix.len(),
            26,
            "ULID suffix must be 26 chars, not UUID-hex-32; got {suffix:?} (len={})",
            suffix.len()
        );
        for (i, b) in suffix.bytes().enumerate() {
            assert!(
                CROCKFORD_ALPHABET.as_bytes().contains(&b),
                "suffix[{i}]={:?} not in Crockford alphabet {CROCKFORD_ALPHABET}; id={id}",
                b as char
            );
        }
        // Forbidden Crockford exclusions must never appear.
        assert!(
            !suffix
                .bytes()
                .any(|b| matches!(b, b'I' | b'L' | b'O' | b'U' | b'i' | b'l' | b'o' | b'u')),
            "forbidden crockford chars in {id}"
        );
        assert!(is_valid_snippet_id(&id), "got {id}");
        // Explicitly not UUID-v7-simple under snp_ (32 lowercase hex).
        assert_ne!(suffix.len(), 32);
        assert!(
            !suffix
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && suffix.len() == 32),
            "must not be UUID hex"
        );
    }

    #[test]
    fn uuid_v7_simple_is_not_valid_snippet_id_shape() {
        // Historical mistaken shape: snp_ + Uuid::now_v7().simple() (32 hex).
        let fake = format!("snp_{}", uuid::Uuid::now_v7().simple());
        assert_eq!(fake.strip_prefix("snp_").unwrap().len(), 32);
        assert!(
            !is_valid_snippet_id(&fake),
            "UUID v7 simple must fail ADR 0010 §2 validation; got {fake}"
        );
    }

    #[test]
    fn repeated_issue_distinct_ids() {
        let mut store = SessionSnippetStore::new();
        let a = store.issue(Path::new("/tmp/a.txt"), 1, 1, "v1", "x", 1);
        let b = store.issue(Path::new("/tmp/a.txt"), 1, 1, "v1", "x", 1);
        assert_ne!(a.snippet_id, b.snippet_id);
        assert_eq!(store.len(), 2);
        assert!(is_valid_snippet_id(&a.snippet_id) && is_valid_snippet_id(&b.snippet_id));
    }

    #[test]
    fn stores_are_independent_not_process_global() {
        // Two Default stores simulate two session Resources bags — not a static.
        let mut a = SessionSnippetStore::new();
        let b = SessionSnippetStore::new();
        let snip = a.issue(Path::new("/tmp/a.txt"), 1, 1, "v", "x", 1);
        assert!(a.contains(&snip.snippet_id));
        assert!(!b.contains(&snip.snippet_id));
        assert!(b.is_empty());
    }

    #[test]
    fn line_range_empty_file() {
        assert_eq!(snippet_line_range(0, None, None), (1, 1));
    }

    #[test]
    fn line_range_window() {
        assert_eq!(snippet_line_range(10, Some(3), Some(2)), (3, 4));
        assert_eq!(snippet_line_range(10, None, None), (1, 10));
    }

    #[test]
    fn vc005_expire_path_removes_only_matching_path() {
        let mut store = SessionSnippetStore::new();
        let a = store.issue(Path::new("/tmp/a.txt"), 1, 1, "va", "a", 1);
        let b = store.issue(Path::new("/tmp/b.txt"), 1, 1, "vb", "b", 1);
        let a2 = store.issue(Path::new("/tmp/a.txt"), 1, 1, "va", "a", 1);
        store.expire_path(Path::new("/tmp/a.txt"));
        assert!(!store.contains(&a.snippet_id));
        assert!(!store.contains(&a2.snippet_id));
        assert!(store.contains(&b.snippet_id));
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn vc005_expire_all_clears_table() {
        let mut store = SessionSnippetStore::new();
        store.issue(Path::new("/tmp/a.txt"), 1, 1, "v", "x", 1);
        store.issue(Path::new("/tmp/b.txt"), 1, 1, "v", "y", 1);
        store.expire_all();
        assert!(store.is_empty());
    }

    #[test]
    fn vc005_bash_read_only_plan_is_none() {
        let cwd = Path::new("/ws");
        assert_eq!(
            bash_snippet_expire_plan("ls -la", cwd),
            BashSnippetExpirePlan::None
        );
        assert_eq!(
            bash_snippet_expire_plan("echo hello", cwd),
            BashSnippetExpirePlan::None
        );
        assert_eq!(
            bash_snippet_expire_plan("git status", cwd),
            BashSnippetExpirePlan::None
        );
    }

    #[test]
    fn vc005_bash_redirect_plan_expires_known_path() {
        let cwd = Path::new("/ws");
        match bash_snippet_expire_plan("echo x > foo.txt", cwd) {
            BashSnippetExpirePlan::Paths(paths) => {
                assert!(
                    paths.iter().any(|p| p.ends_with("foo.txt")),
                    "expected foo.txt in {paths:?}"
                );
            }
            other => panic!("expected Paths, got {other:?}"),
        }
    }

    #[test]
    fn vc005_bash_unknown_mutator_plan_is_all() {
        let cwd = Path::new("/ws");
        assert_eq!(
            bash_snippet_expire_plan("python do_stuff.py", cwd),
            BashSnippetExpirePlan::All
        );
        assert_eq!(
            bash_snippet_expire_plan("sudo true", cwd),
            BashSnippetExpirePlan::All
        );
    }

    #[test]
    fn vc005_bash_rm_known_path_plan() {
        let cwd = Path::new("/ws");
        match bash_snippet_expire_plan("rm -f bar.txt", cwd) {
            BashSnippetExpirePlan::Paths(paths) => {
                assert!(paths.iter().any(|p| p.ends_with("bar.txt")));
            }
            other => panic!("expected Paths, got {other:?}"),
        }
    }

    #[test]
    fn vc005_expire_path_matches_macos_private_tmp_asymmetry() {
        // Mint as if canonicalize produced /private/tmp/…; expire via /tmp/….
        let mut store = SessionSnippetStore::new();
        let snip = store.issue(Path::new("/private/tmp/a.txt"), 1, 1, "v", "x", 1);
        store.expire_path(Path::new("/tmp/a.txt"));
        assert!(
            !store.contains(&snip.snippet_id),
            "post-rm path form must still expire mint-time private/tmp ids"
        );
    }

    #[test]
    fn vc005_bash_fd_redirect_is_not_file_mutation() {
        let cwd = Path::new("/ws");
        assert_eq!(
            bash_snippet_expire_plan("ls 2>&1", cwd),
            BashSnippetExpirePlan::None
        );
        assert_eq!(
            bash_snippet_expire_plan("ls 2>/dev/null", cwd),
            BashSnippetExpirePlan::None
        );
        assert_eq!(
            bash_snippet_expire_plan("cat file >/dev/null", cwd),
            BashSnippetExpirePlan::None
        );
    }
}
