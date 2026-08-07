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
//! VC003: issue on successful **UTF-8 text** `read_file` only. Edit require
//! (VC004), write/bash invalidation (VC005), and resume/fork restore (VC006)
//! are out of scope.

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
            !suffix.bytes().any(|b| matches!(b, b'I' | b'L' | b'O' | b'U'
                | b'i' | b'l' | b'o' | b'u')),
            "forbidden crockford chars in {id}"
        );
        assert!(is_valid_snippet_id(&id), "got {id}");
        // Explicitly not UUID-v7-simple under snp_ (32 lowercase hex).
        assert_ne!(suffix.len(), 32);
        assert!(
            !suffix.bytes().all(|b| b.is_ascii_hexdigit() && suffix.len() == 32),
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
}
