//! Usage / cache evidence parsing (ADR 0005).

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Raw usage object as returned by the API (field names may vary).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct UsageRaw {
    #[serde(default)]
    pub prompt_tokens: Option<u64>,
    #[serde(default)]
    pub completion_tokens: Option<u64>,
    #[serde(default)]
    pub total_tokens: Option<u64>,
    /// Common cache field names observed / anticipated; all optional.
    #[serde(default)]
    pub prompt_cache_hit_tokens: Option<u64>,
    #[serde(default)]
    pub prompt_cache_miss_tokens: Option<u64>,
    #[serde(default)]
    pub cached_tokens: Option<u64>,
    #[serde(default)]
    pub cache_hit_tokens: Option<u64>,
    #[serde(default)]
    pub cache_miss_tokens: Option<u64>,
    /// Full raw object for debugging.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Usage {
    pub prompt_tokens: Option<u64>,
    pub completion_tokens: Option<u64>,
    pub total_tokens: Option<u64>,
    pub cache_hit_tokens: Option<u64>,
    pub cache_miss_tokens: Option<u64>,
    pub raw: Value,
}

impl Usage {
    pub fn from_raw(raw: UsageRaw) -> Self {
        let cache_hit = raw
            .prompt_cache_hit_tokens
            .or(raw.cache_hit_tokens)
            .or(raw.cached_tokens);
        let cache_miss = raw.prompt_cache_miss_tokens.or(raw.cache_miss_tokens);
        let as_value = serde_json::to_value(&raw).unwrap_or(Value::Null);
        Self {
            prompt_tokens: raw.prompt_tokens,
            completion_tokens: raw.completion_tokens,
            total_tokens: raw.total_tokens,
            cache_hit_tokens: cache_hit,
            cache_miss_tokens: cache_miss,
            raw: as_value,
        }
    }

    pub fn has_cache_fields(&self) -> bool {
        self.cache_hit_tokens.is_some() || self.cache_miss_tokens.is_some()
    }
}

/// Cache evidence protocol (ADR 0005).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CacheEvidence {
    /// Real API usage fields present.
    UsageFields {
        cache_hit_tokens: Option<u64>,
        cache_miss_tokens: Option<u64>,
    },
    /// Dual identical calls substitute when usage lacks cache fields.
    SubstituteDualCall {
        first_latency_ms: u64,
        second_latency_ms: u64,
        first_prompt_tokens: Option<u64>,
        second_prompt_tokens: Option<u64>,
    },
}

impl CacheEvidence {
    pub fn from_usage(usage: &Usage) -> Option<Self> {
        if usage.has_cache_fields() {
            Some(Self::UsageFields {
                cache_hit_tokens: usage.cache_hit_tokens,
                cache_miss_tokens: usage.cache_miss_tokens,
            })
        } else {
            None
        }
    }

    pub fn log_label(&self) -> &'static str {
        match self {
            Self::UsageFields { .. } => "cache_evidence=usage_fields",
            Self::SubstituteDualCall { .. } => "cache_evidence=substitute_dual_call",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_prompt_cache_hit_fields() {
        let raw: UsageRaw = serde_json::from_value(serde_json::json!({
            "prompt_tokens": 100,
            "completion_tokens": 10,
            "prompt_cache_hit_tokens": 80,
            "prompt_cache_miss_tokens": 20
        }))
        .unwrap();
        let u = Usage::from_raw(raw);
        assert_eq!(u.cache_hit_tokens, Some(80));
        assert_eq!(u.cache_miss_tokens, Some(20));
        assert!(u.has_cache_fields());
        let ev = CacheEvidence::from_usage(&u).unwrap();
        assert!(matches!(ev, CacheEvidence::UsageFields { .. }));
    }

    #[test]
    fn no_cache_fields_returns_none_evidence() {
        let raw: UsageRaw = serde_json::from_value(serde_json::json!({
            "prompt_tokens": 50,
            "completion_tokens": 5
        }))
        .unwrap();
        let u = Usage::from_raw(raw);
        assert!(!u.has_cache_fields());
        assert!(CacheEvidence::from_usage(&u).is_none());
    }
}
