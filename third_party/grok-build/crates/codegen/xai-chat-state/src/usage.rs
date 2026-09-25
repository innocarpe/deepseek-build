//! Per-prompt and per-session billing ledgers (not serialized).
//!
//! `total_tokens()` is input + output: Responses wire `total` is live context
//! length. Compaction and other side calls never call `record_main_loop_call`.
//!
//! # Completeness ownership
//!
//! Wire incomplete is the OR of these stores (each has a distinct role):
//!
//! - **`UsageLedger.incomplete`** — durable on the bill snapshot. Set by nested
//!   subagent incomplete fold, drain timeout, true apply-miss, and
//!   `mark_usage_incomplete`. Monotonic for a ledger instance.
//! - **Sticky (`subagent_usage_not_applied` on the coordinator)** — pin-scoped
//!   **report** signal (session-only attribution or apply-miss report). Not a
//!   second token sink; does not stain ledgers by itself.
//! - **Foreground live IDs** — fold may still land; freeze drains ≤120s or fails
//!   closed. Cancel skips multi-second drain (actor-loop safety).
//! - **Background live** — never waits; prompt report incomplete immediately;
//!   spend still folds into the session ledger at completion (no session-ledger
//!   incomplete).
//!
//! Freeze and cancel share one outcome policy: ledger marks only on fail-closed;
//! sticky and background_live are report-level only.
//!
//! Projection (`PromptUsage`) never invents tokens; it only ORs completeness
//! and scrubs costs when partial or incomplete.

use indexmap::IndexMap;
use xai_grok_sampling_types::TokenUsage;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UsageTotals {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cached_read_tokens: u64,
    pub cache_creation_tokens: u64,
    pub reasoning_tokens: u64,
    pub model_calls: u64,
    pub api_duration_ms: u64,
    /// USD ticks (1e10 per USD). Absent when no call reported cost.
    pub cost_usd_ticks: Option<i64>,
    pub cost_missing_calls: u64,
}

impl UsageTotals {
    fn from_call(
        usage: &TokenUsage,
        api_duration_ms: Option<u64>,
        cost_usd_ticks: Option<i64>,
    ) -> Self {
        let cost_usd_ticks = xai_grok_sampling_types::reported_cost_ticks(cost_usd_ticks);
        Self {
            input_tokens: u64::from(usage.prompt_tokens),
            output_tokens: u64::from(usage.completion_tokens),
            cached_read_tokens: u64::from(usage.cached_prompt_tokens),
            cache_creation_tokens: u64::from(usage.cache_creation_prompt_tokens),
            reasoning_tokens: u64::from(usage.reasoning_tokens),
            model_calls: 1,
            api_duration_ms: api_duration_ms.unwrap_or(0),
            cost_usd_ticks,
            cost_missing_calls: u64::from(cost_usd_ticks.is_none()),
        }
    }

    pub fn total_tokens(&self) -> u64 {
        self.input_tokens.saturating_add(self.output_tokens)
    }

    pub fn cost_is_partial(&self) -> bool {
        self.cost_usd_ticks.is_some() && self.cost_missing_calls > 0
    }

    fn fold_totals(&mut self, other: &UsageTotals) {
        let Self {
            input_tokens,
            output_tokens,
            cached_read_tokens,
            cache_creation_tokens,
            reasoning_tokens,
            model_calls,
            api_duration_ms,
            cost_usd_ticks,
            cost_missing_calls,
        } = other;
        self.input_tokens = self.input_tokens.saturating_add(*input_tokens);
        self.output_tokens = self.output_tokens.saturating_add(*output_tokens);
        self.cached_read_tokens = self.cached_read_tokens.saturating_add(*cached_read_tokens);
        self.cache_creation_tokens = self
            .cache_creation_tokens
            .saturating_add(*cache_creation_tokens);
        self.reasoning_tokens = self.reasoning_tokens.saturating_add(*reasoning_tokens);
        self.model_calls = self.model_calls.saturating_add(*model_calls);
        self.api_duration_ms = self.api_duration_ms.saturating_add(*api_duration_ms);
        self.cost_missing_calls = self.cost_missing_calls.saturating_add(*cost_missing_calls);
        self.cost_usd_ticks = merge_cost_ticks(self.cost_usd_ticks, *cost_usd_ticks);
    }
}

fn merge_cost_ticks(a: Option<i64>, b: Option<i64>) -> Option<i64> {
    match (a, b) {
        (None, None) => None,
        (a, b) => Some(a.unwrap_or(0).saturating_add(b.unwrap_or(0))),
    }
}

/// Session-cumulative hit/miss for Path A (spec 10 §1.5.2).
///
/// Same arithmetic as the overlay counter in `dsb-agent`: a response with no
/// cache fields increments `unreported` and adds nothing to the token sums.
/// A missing half of the pair stays missing rather than becoming `0`.
/// This copy lives on the in-memory session ledger because `xai-grok-shell`
/// does not depend on `dsb-agent`. It is not serialized; the persisted copy
/// is the overlay's, on the REPL / `run` path.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CacheSessionTotals {
    hit_tokens: u64,
    miss_tokens: u64,
    reported: u64,
    unreported: u64,
}

impl CacheSessionTotals {
    pub fn record(&mut self, usage: &TokenUsage) {
        if usage.cache_hit_tokens.is_none() && usage.cache_miss_tokens.is_none() {
            self.unreported = self.unreported.saturating_add(1);
            return;
        }
        self.reported = self.reported.saturating_add(1);
        if let Some(hit) = usage.cache_hit_tokens {
            self.hit_tokens = self.hit_tokens.saturating_add(u64::from(hit));
        }
        if let Some(miss) = usage.cache_miss_tokens {
            self.miss_tokens = self.miss_tokens.saturating_add(u64::from(miss));
        }
    }

    pub fn has_evidence(&self) -> bool {
        self.reported > 0
    }

    pub fn rate_label(&self) -> String {
        let total = u128::from(self.hit_tokens) + u128::from(self.miss_tokens);
        if total == 0 {
            return "na".to_string();
        }
        let rate = u128::from(self.hit_tokens) * 100 / total;
        rate.to_string()
    }

    /// `cache_session=hit=<n>,miss=<n>,rate=<pct>,reported=<n>,unreported=<n>`
    pub fn log_label(&self) -> String {
        format!(
            "cache_session=hit={},miss={},rate={},reported={},unreported={}",
            self.hit_tokens,
            self.miss_tokens,
            self.rate_label(),
            self.reported,
            self.unreported
        )
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UsageLedger {
    pub totals: UsageTotals,
    pub by_model: IndexMap<String, UsageTotals>,
    /// Main-agent loop rounds for `num_turns` (subagents excluded).
    pub main_loop_model_calls: u64,
    /// Bill may under-count (drain timeout, nested subagent incomplete, apply failure).
    pub incomplete: bool,
    /// §1.5.2 counter for main-loop responses. Not part of the billed chip.
    pub cache_session: CacheSessionTotals,
}

impl UsageLedger {
    /// Fold one main-agent-loop model call. This is the only writer of
    /// `main_loop_model_calls` (the wire `numTurns`); side calls such as
    /// compaction must not use it.
    pub fn record_main_loop_call(
        &mut self,
        model_id: &str,
        usage: &TokenUsage,
        api_duration_ms: Option<u64>,
        cost_usd_ticks: Option<i64>,
    ) {
        let call = UsageTotals::from_call(usage, api_duration_ms, cost_usd_ticks);
        self.main_loop_model_calls = self.main_loop_model_calls.saturating_add(1);
        self.cache_session.record(usage);
        self.fold_entry(model_id, &call);
    }

    /// Fold subagent usage without incrementing `main_loop_model_calls`.
    pub fn record_subagent(&mut self, by_model: &[(String, UsageTotals)], incomplete: bool) {
        for (model_id, totals) in by_model {
            self.fold_entry(model_id, totals);
        }
        if incomplete {
            self.incomplete = true;
        }
    }

    pub fn mark_incomplete(&mut self) {
        self.incomplete = true;
    }

    fn fold_entry(&mut self, model_id: &str, totals: &UsageTotals) {
        self.totals.fold_totals(totals);
        self.by_model
            .entry(model_id.to_owned())
            .or_default()
            .fold_totals(totals);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tu(prompt: u32, completion: u32) -> TokenUsage {
        TokenUsage {
            prompt_tokens: prompt,
            completion_tokens: completion,
            total_tokens: 999_999,
            reasoning_tokens: 0,
            cached_prompt_tokens: 0,
            cache_creation_prompt_tokens: 0,
            cache_hit_tokens: None,
            cache_miss_tokens: None,
        }
    }

    #[test]
    fn ledger_sums_partial_subagent_and_zero_cost() {
        let mut ledger = UsageLedger::default();
        ledger.record_main_loop_call("m", &tu(1, 1), None, Some(0));
        assert_eq!(ledger.totals.cost_usd_ticks, None);
        assert_eq!(ledger.totals.cost_missing_calls, 1);

        ledger.record_main_loop_call("a", &tu(100, 10), Some(100), None);
        ledger.record_main_loop_call("a", &tu(50, 5), Some(50), Some(70));
        assert_eq!(ledger.totals.cost_usd_ticks, Some(70));
        assert!(ledger.totals.cost_is_partial());
        assert_eq!(ledger.main_loop_model_calls, 3);

        ledger.record_subagent(
            &[(
                "b".into(),
                UsageTotals {
                    input_tokens: 5,
                    model_calls: 1,
                    ..Default::default()
                },
            )],
            false,
        );
        assert_eq!(ledger.by_model.get("b").map(|m| m.input_tokens), Some(5));
        assert_eq!(ledger.main_loop_model_calls, 3);
        assert_eq!(ledger.totals.model_calls, 4);
        assert!(!ledger.incomplete);

        ledger.record_subagent(&[], true);
        assert!(ledger.incomplete);
    }

    fn with_cache(hit: Option<u32>, miss: Option<u32>) -> TokenUsage {
        TokenUsage {
            prompt_tokens: 100,
            completion_tokens: 1,
            cache_hit_tokens: hit,
            cache_miss_tokens: miss,
            cached_prompt_tokens: hit.unwrap_or(0),
            ..TokenUsage::default()
        }
    }

    #[test]
    fn cache_session_sums_reported_halves_and_skips_unreported() {
        let mut ledger = UsageLedger::default();
        ledger.record_main_loop_call("m", &with_cache(Some(80), Some(20)), None, None);
        ledger.record_main_loop_call("m", &with_cache(None, None), None, None);
        ledger.record_main_loop_call("m", &with_cache(Some(40), None), None, None);

        assert!(ledger.cache_session.has_evidence());
        assert_eq!(
            ledger.cache_session.log_label(),
            "cache_session=hit=120,miss=20,rate=85,reported=2,unreported=1"
        );
    }

    #[test]
    fn cache_session_with_no_fields_logs_nothing() {
        let mut ledger = UsageLedger::default();
        ledger.record_main_loop_call("m", &tu(10, 1), None, None);
        assert!(!ledger.cache_session.has_evidence());
        assert_eq!(
            ledger.cache_session.log_label(),
            "cache_session=hit=0,miss=0,rate=na,reported=0,unreported=1"
        );
    }
}
