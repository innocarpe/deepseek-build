//! Session-cumulative cache hit/miss (spec 10 §1.5.2).
//!
//! Per-call cache evidence answers *how warm was this call*. The cost question
//! belongs to the session, so this counter accumulates over the whole session
//! and is never reset by a turn.
//!
//! The rule that gives the counter its value is the unreported one: a response
//! that carried no cache fields increments `unreported` and contributes
//! **nothing** to `hit`, `miss`, or `rate`. Counting it as a miss would invent
//! a number the provider never sent.

use dsb_provider_deepseek::CacheEvidence;
use serde::{Deserialize, Serialize};

/// Accumulated cache evidence for one session (spec 10 §1.5.2).
///
/// The unit of accumulation is **one model response**, because that is the unit
/// a provider reports prompt tokens in: a user turn that runs tool rounds makes
/// several requests, and each one carries its own usage. The log line is
/// printed once per user turn, after its rounds complete.
///
/// The counter is stored **with the session** (spec 100 `meta`). "Reset on a
/// new session" is the contract, and a resumed conversation is not a new
/// session — spec 100 §1.1 item 4 has the CLI resume by id, and §1.5.1 rule 5
/// already treats the resuming process as the continuation of one
/// conversation. Keeping the counter in process memory only would make a
/// resumed session silently restart its totals, which is exactly the cost
/// question §1.5.2 exists to answer.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheSessionTotals {
    #[serde(default)]
    hit_tokens: u64,
    #[serde(default)]
    miss_tokens: u64,
    #[serde(default)]
    reported: u64,
    #[serde(default)]
    unreported: u64,
}

impl CacheSessionTotals {
    /// Fold one model call's cache evidence into the session counter.
    ///
    /// `None` — and a substitute-protocol measurement, which is not a response
    /// that carried cache fields — increments `unreported` only. Neither
    /// touches `hit`, `miss`, or `rate`.
    pub fn record(&mut self, evidence: Option<&CacheEvidence>) {
        match evidence {
            Some(CacheEvidence::UsageFields {
                cache_hit_tokens,
                cache_miss_tokens,
            }) => {
                self.reported += 1;
                // Only a number the provider actually sent is added. A missing
                // half stays missing rather than defaulting to a zero, which
                // would read as a measured value in the rate.
                if let Some(hit) = cache_hit_tokens {
                    self.hit_tokens += hit;
                }
                if let Some(miss) = cache_miss_tokens {
                    self.miss_tokens += miss;
                }
            }
            _ => self.unreported += 1,
        }
    }

    /// True once any call in this session carried cache fields.
    ///
    /// This is the condition §1.5.2 puts on printing the line: a session whose
    /// calls all arrived without cache fields has nothing to report and logs
    /// nothing. Once one call has reported, every later turn logs — including
    /// its unreported ones, which is the only way that count is ever observed.
    pub fn has_evidence(&self) -> bool {
        self.reported > 0
    }

    pub fn hit_tokens(&self) -> u64 {
        self.hit_tokens
    }

    pub fn miss_tokens(&self) -> u64 {
        self.miss_tokens
    }

    pub fn reported(&self) -> u64 {
        self.reported
    }

    pub fn unreported(&self) -> u64 {
        self.unreported
    }

    /// `hit * 100 / (hit + miss)`, integer, floor. `None` is §1.5.2's `na`,
    /// which is the only correct answer when nothing was reported: a rate over
    /// zero tokens is not 0%, it is undefined.
    ///
    /// Computed in `u128` so a long session cannot wrap the multiplication.
    /// Integer division on non-negative values is the floor the contract asks
    /// for (1/3 → 33, 2/3 → 66).
    pub fn rate_pct(&self) -> Option<u64> {
        let total = u128::from(self.hit_tokens) + u128::from(self.miss_tokens);
        if total == 0 {
            return None;
        }
        let rate = u128::from(self.hit_tokens) * 100 / total;
        Some(rate as u64)
    }

    /// `rate` value as it appears in the log field: the integer, or `na`.
    pub fn rate_label(&self) -> String {
        match self.rate_pct() {
            Some(pct) => pct.to_string(),
            None => "na".to_string(),
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn usage_fields(hit: Option<u64>, miss: Option<u64>) -> CacheEvidence {
        CacheEvidence::UsageFields {
            cache_hit_tokens: hit,
            cache_miss_tokens: miss,
        }
    }

    /// Spec 10 §4.4: accumulation is over token sums, and an unreported turn
    /// moves only the unreported count.
    #[test]
    fn cache_totals_accumulate_and_track_unreported() {
        let mut totals = CacheSessionTotals::default();
        totals.record(Some(&usage_fields(Some(80), Some(20))));
        totals.record(None); // carried no cache fields
        totals.record(Some(&usage_fields(Some(40), Some(10))));

        assert_eq!(totals.hit_tokens(), 120);
        assert_eq!(totals.miss_tokens(), 30);
        assert_eq!(totals.reported(), 2);
        assert_eq!(totals.unreported(), 1);

        // Token sums, not turn counts: three turns counted as one hit each
        // would read 67%, and two of three as reported would read 66%.
        assert_eq!(totals.rate_pct(), Some(80));
        assert_eq!(
            totals.log_label(),
            "cache_session=hit=120,miss=30,rate=80,reported=2,unreported=1"
        );
    }

    /// The unreported turn contributes nothing — it is not folded in as a
    /// zero-token miss, and it is not counted as a reported turn.
    #[test]
    fn an_unreported_turn_does_not_contaminate_the_totals() {
        let mut with_gap = CacheSessionTotals::default();
        with_gap.record(Some(&usage_fields(Some(80), Some(20))));
        with_gap.record(None);
        with_gap.record(Some(&usage_fields(Some(40), Some(10))));

        let mut without_gap = CacheSessionTotals::default();
        without_gap.record(Some(&usage_fields(Some(80), Some(20))));
        without_gap.record(Some(&usage_fields(Some(40), Some(10))));

        assert_eq!(with_gap.hit_tokens(), without_gap.hit_tokens());
        assert_eq!(with_gap.miss_tokens(), without_gap.miss_tokens());
        assert_eq!(with_gap.rate_pct(), without_gap.rate_pct());
        // The gap is visible only where it belongs.
        assert_eq!(with_gap.unreported(), without_gap.unreported() + 1);
    }

    /// `rate=na` is the `hit + miss == 0` path, and it is not `0`.
    #[test]
    fn rate_is_na_when_no_tokens_were_reported() {
        let mut totals = CacheSessionTotals::default();
        totals.record(None);
        totals.record(None);

        assert_eq!(totals.rate_pct(), None);
        assert_eq!(totals.rate_label(), "na");
        assert!(!totals.has_evidence());
        assert_eq!(
            totals.log_label(),
            "cache_session=hit=0,miss=0,rate=na,reported=0,unreported=2"
        );
    }

    /// A session with no evidence at all prints nothing; one that has reported
    /// once keeps printing through its unreported turns.
    #[test]
    fn the_line_is_gated_on_evidence_anywhere_in_the_session() {
        let mut all_unreported = CacheSessionTotals::default();
        all_unreported.record(None);
        all_unreported.record(None);
        assert!(!all_unreported.has_evidence());

        let mut one_reported = CacheSessionTotals::default();
        one_reported.record(Some(&usage_fields(Some(1), Some(1))));
        one_reported.record(None);
        assert!(one_reported.has_evidence());
    }

    /// Integer floor: 1/3 → 33, 2/3 → 66. Rounding would give 33 and 67.
    #[test]
    fn rate_floors_integer_division() {
        let mut one_of_three = CacheSessionTotals::default();
        one_of_three.record(Some(&usage_fields(Some(1), Some(2))));
        assert_eq!(one_of_three.rate_pct(), Some(33));

        let mut two_of_three = CacheSessionTotals::default();
        two_of_three.record(Some(&usage_fields(Some(2), Some(1))));
        assert_eq!(two_of_three.rate_pct(), Some(66));

        // 100/101 floors to 99, not 100.
        let mut almost_all = CacheSessionTotals::default();
        almost_all.record(Some(&usage_fields(Some(100), Some(1))));
        assert_eq!(almost_all.rate_pct(), Some(99));
    }

    /// A provider that sends only the hit half is not defaulted to a miss.
    #[test]
    fn a_missing_half_is_not_defaulted_to_a_zero_miss() {
        let mut totals = CacheSessionTotals::default();
        totals.record(Some(&usage_fields(Some(70), None)));

        assert_eq!(totals.hit_tokens(), 70);
        assert_eq!(totals.miss_tokens(), 0);
        assert_eq!(totals.reported(), 1);
        assert_eq!(totals.unreported(), 0);
        // 70/70 — the honest answer for a response that reported no miss.
        assert_eq!(totals.rate_pct(), Some(100));
    }

    /// The dual-call substitute measures latency; it is not a response that
    /// carried cache fields, so it is unreported.
    #[test]
    fn substitute_protocol_counts_as_unreported() {
        let mut totals = CacheSessionTotals::default();
        totals.record(Some(&CacheEvidence::SubstituteDualCall {
            first_latency_ms: 10,
            second_latency_ms: 12,
            first_prompt_tokens: Some(5),
            second_prompt_tokens: Some(5),
        }));

        assert_eq!(totals.reported(), 0);
        assert_eq!(totals.unreported(), 1);
        assert_eq!(totals.hit_tokens(), 0);
        assert_eq!(totals.miss_tokens(), 0);
        assert!(!totals.has_evidence());
    }

    /// A fresh counter is a fresh session: the totals are per session and are
    /// never carried across one.
    #[test]
    fn a_new_session_starts_from_zero() {
        let mut previous = CacheSessionTotals::default();
        previous.record(Some(&usage_fields(Some(80), Some(20))));
        assert!(previous.has_evidence());

        let fresh = CacheSessionTotals::default();
        assert_eq!(fresh.hit_tokens(), 0);
        assert_eq!(fresh.miss_tokens(), 0);
        assert_eq!(fresh.reported(), 0);
        assert_eq!(fresh.unreported(), 0);
        assert_eq!(fresh.rate_pct(), None);
        assert!(!fresh.has_evidence());
    }

    /// A large session does not wrap the rate multiplication.
    #[test]
    fn rate_survives_token_sums_past_u64_multiplication() {
        let mut totals = CacheSessionTotals::default();
        totals.record(Some(&usage_fields(Some(u64::MAX / 4), Some(0))));
        assert_eq!(totals.rate_pct(), Some(100));
    }
}
