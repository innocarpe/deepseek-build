use super::*;
use xai_chat_state::UsageLedger;
use xai_grok_sampling_types::TokenUsage;

fn tu(prompt: u32, completion: u32) -> TokenUsage {
    TokenUsage {
        prompt_tokens: prompt,
        completion_tokens: completion,
        total_tokens: prompt + completion,
        reasoning_tokens: 0,
        cached_prompt_tokens: 0,
        cache_creation_prompt_tokens: 0,
        cache_hit_tokens: None,
        cache_miss_tokens: None,
    }
}

fn live(calls: &[(&str, u32, u32, Option<i64>)]) -> UsageSummary {
    let mut ledger = UsageLedger::default();
    for (model, prompt, completion, cost) in calls {
        ledger.record_main_loop_call(model, &tu(*prompt, *completion), Some(10), *cost);
    }
    UsageSummary::from_ledger(&ledger)
}

#[test]
fn first_turn_writes_session_and_one_turn() {
    let mut file = SessionUsageFile::new("sess-1");
    let first = live(&[("grok-4", 100, 20, Some(50))]);
    file.apply_turn(1, "2026-08-26T00:00:00Z", &first, None);

    let [t0] = file.turns.as_slice() else {
        panic!("expected one turn: {:?}", file.turns);
    };
    assert_eq!(t0.turn_number, 1);
    assert_eq!(t0.usage.input_tokens, 100);
    assert_eq!(t0.usage.output_tokens, 20);
    assert_eq!(t0.usage.cost_usd_ticks, Some(50));
    assert_eq!(file.session.input_tokens, 100);
    assert_eq!(file.session.output_tokens, 20);
    assert_eq!(file.session.turn_count, 1);
    assert_eq!(file.session.cost_usd_ticks, Some(50));
    assert_eq!(file.session.primary_model_id.as_deref(), Some("grok-4"));
    assert_eq!(file.updated_at, "2026-08-26T00:00:00Z");
}

#[test]
fn session_primary_model_is_the_most_used_not_the_last_turn() {
    let mut file = SessionUsageFile::new("sess-1");
    let first = live(&[("grok-4", 100, 20, Some(50)), ("grok-4", 80, 10, Some(40))]);
    file.apply_turn(1, "t1", &first, None);
    file.apply_turn(
        2,
        "t2",
        &live(&[
            ("grok-4", 100, 20, Some(50)),
            ("grok-4", 80, 10, Some(40)),
            ("grok-fast", 10, 2, Some(1)),
        ]),
        Some(&first),
    );

    assert_eq!(
        file.turns
            .get(1)
            .and_then(|t| t.usage.primary_model_id.as_deref()),
        Some("grok-fast")
    );
    assert_eq!(file.session.primary_model_id.as_deref(), Some("grok-4"));
}

#[test]
fn second_turn_appends_and_session_becomes_latest_ledger() {
    let mut file = SessionUsageFile::new("sess-1");
    let first = live(&[("grok-4", 100, 20, Some(50))]);
    file.apply_turn(1, "t1", &first, None);
    file.apply_turn(
        2,
        "t2",
        &live(&[("grok-4", 100, 20, Some(50)), ("grok-4", 40, 10, Some(20))]),
        Some(&first),
    );

    let [_, t1] = file.turns.as_slice() else {
        panic!("expected two turns: {:?}", file.turns);
    };
    assert_eq!(t1.turn_number, 2);
    assert_eq!(t1.usage.input_tokens, 40);
    assert_eq!(t1.usage.output_tokens, 10);
    assert_eq!(t1.usage.cost_usd_ticks, Some(20));
    assert_eq!(file.session.input_tokens, 140);
    assert_eq!(file.session.output_tokens, 30);
    assert_eq!(file.session.turn_count, 2);
    assert_eq!(file.session.cost_usd_ticks, Some(70));
}

#[test]
fn inherited_turn_number_without_fold_appends() {
    let mut file = SessionUsageFile::new("sess-1");
    let first = live(&[("grok-4", 100, 20, Some(50))]);
    file.apply_turn(1, "t1", &first, None);
    file.restore_apply_cursor(None, None);
    let resumed = live(&[("grok-4", 10, 2, Some(5))]);
    file.apply_turn(1, "t-resume", &resumed, None);

    let [t0, t1] = file.turns.as_slice() else {
        panic!("expected two turns: {:?}", file.turns);
    };
    assert_eq!(t0.turn_number, 1);
    assert_eq!(t0.usage.input_tokens, 100);
    assert_eq!(t1.turn_number, 2);
    assert_eq!(t1.usage.input_tokens, 10);
    assert_eq!(file.session.input_tokens, 110);
    assert_eq!(file.session.turn_count, 2);
}

#[test]
fn duplicate_turn_number_zero_delta_does_not_mutate_turns() {
    let mut file = SessionUsageFile::new("sess-1");
    let first = live(&[("grok-4", 100, 20, Some(50))]);
    file.apply_turn(1, "t1", &first, None);
    file.apply_turn(1, "t1-again", &first, Some(&first));

    let [t0] = file.turns.as_slice() else {
        panic!("expected one turn: {:?}", file.turns);
    };
    assert_eq!(t0.ended_at, "t1");
    assert_eq!(file.session.turn_count, 1);
    assert_eq!(file.updated_at, "t1-again");
}

#[test]
fn duplicate_turn_number_folds_extra_live_usage() {
    let mut file = SessionUsageFile::new("sess-1");
    let first = live(&[("grok-4", 100, 20, Some(50))]);
    file.apply_turn(1, "t1", &first, None);
    let continued = live(&[("grok-4", 100, 20, Some(50)), ("grok-4", 40, 10, Some(20))]);
    file.apply_turn(1, "t1-late", &continued, Some(&first));

    let [t0] = file.turns.as_slice() else {
        panic!("expected one turn: {:?}", file.turns);
    };
    assert_eq!(t0.ended_at, "t1-late");
    assert_eq!(t0.usage.input_tokens, 140);
    assert_eq!(t0.usage.output_tokens, 30);
    assert_eq!(t0.usage.cost_usd_ticks, Some(70));
    assert_eq!(file.session.input_tokens, 140);
    assert_eq!(file.session.output_tokens, 30);
    assert_eq!(file.session.turn_count, 1);
    assert_eq!(file.session.cost_usd_ticks, Some(70));
}

#[test]
fn resume_folds_new_process_ledger_onto_persisted_session() {
    let mut file = SessionUsageFile::new("sess-1");
    let first = live(&[("grok-4", 100, 20, Some(50))]);
    file.apply_turn(1, "t1", &first, None);
    file.apply_turn(
        2,
        "t2",
        &live(&[("grok-4", 100, 20, Some(50)), ("grok-4", 40, 10, Some(20))]),
        Some(&first),
    );

    let post_resume_1 = live(&[("grok-4", 25, 5, Some(8))]);
    file.apply_turn(3, "t3", &post_resume_1, None);

    let [_, _, t2] = file.turns.as_slice() else {
        panic!("expected three turns: {:?}", file.turns);
    };
    assert_eq!(t2.turn_number, 3);
    assert_eq!(t2.usage.input_tokens, 25);
    assert_eq!(t2.usage.output_tokens, 5);
    assert_eq!(file.session.input_tokens, 165);
    assert_eq!(file.session.output_tokens, 35);
    assert_eq!(file.session.turn_count, 3);
    assert_eq!(file.session.cost_usd_ticks, Some(78));
}

#[test]
fn resume_later_turns_use_process_local_delta() {
    let mut file = SessionUsageFile::new("sess-1");
    let first = live(&[("grok-4", 100, 20, Some(50))]);
    file.apply_turn(1, "t1", &first, None);
    file.apply_turn(
        2,
        "t2",
        &live(&[("grok-4", 100, 20, Some(50)), ("grok-4", 40, 10, Some(20))]),
        Some(&first),
    );

    let post_resume_1 = live(&[("grok-4", 25, 5, Some(8))]);
    file.apply_turn(3, "t3", &post_resume_1, None);
    file.apply_turn(
        4,
        "t4",
        &live(&[("grok-4", 25, 5, Some(8)), ("grok-4", 30, 6, Some(9))]),
        Some(&post_resume_1),
    );

    let [_, _, _, t3] = file.turns.as_slice() else {
        panic!("expected four turns: {:?}", file.turns);
    };
    assert_eq!(t3.usage.input_tokens, 30);
    assert_eq!(t3.usage.output_tokens, 6);
    assert_eq!(file.session.input_tokens, 195);
    assert_eq!(file.session.output_tokens, 41);
    assert_eq!(file.session.turn_count, 4);
    assert_eq!(file.session.cost_usd_ticks, Some(87));
}

#[test]
fn retain_turns_through_drops_later_turns_and_rebuilds_session() {
    let mut file = SessionUsageFile::new("sess-1");
    let first = live(&[("grok-4", 100, 20, Some(50))]);
    file.apply_turn(1, "t1", &first, None);
    file.apply_turn(
        2,
        "t2",
        &live(&[("grok-4", 100, 20, Some(50)), ("grok-4", 40, 10, Some(20))]),
        Some(&first),
    );
    file.retain_turns_through(1);

    let [t0] = file.turns.as_slice() else {
        panic!("expected one turn: {:?}", file.turns);
    };
    assert_eq!(t0.turn_number, 1);
    assert_eq!(file.session.input_tokens, 100);
    assert_eq!(file.session.turn_count, 1);
    assert_eq!(file.session.cost_usd_ticks, Some(50));
}

#[test]
fn turn_lookup_returns_matching_row() {
    let mut file = SessionUsageFile::new("sess-1");
    let first = live(&[("grok-4", 100, 20, Some(50))]);
    file.apply_turn(1, "t1", &first, None);
    file.apply_turn(
        2,
        "t2",
        &live(&[("grok-4", 100, 20, Some(50)), ("grok-4", 40, 10, Some(20))]),
        Some(&first),
    );

    assert_eq!(file.turn(2).unwrap().usage.input_tokens, 40);
    assert!(file.turn(3).is_none());
}

#[test]
fn path_a_usage_summary_roundtrips_the_live_ledger() {
    let mut ledger = UsageLedger::default();
    ledger.record_main_loop_call(
        "grok-4",
        &TokenUsage {
            prompt_tokens: 100,
            completion_tokens: 10,
            cache_hit_tokens: Some(80),
            cache_miss_tokens: Some(20),
            cached_prompt_tokens: 80,
            ..TokenUsage::default()
        },
        Some(1_000),
        Some(100),
    );
    ledger.record_main_loop_call("grok-4", &tu(50, 5), Some(2_000), None);
    ledger.record_subagent(
        &[(
            "subagent-model".into(),
            xai_chat_state::UsageTotals {
                input_tokens: 7,
                output_tokens: 2,
                model_calls: 1,
                api_duration_ms: 3_000,
                cost_usd_ticks: Some(25),
                ..Default::default()
            },
        )],
        true,
    );

    let mut file = SessionUsageFile::new("sess-path-a");
    file.session = UsageSummary::from_ledger(&ledger);
    let bytes = serde_json::to_vec(&file).unwrap();
    let restored: SessionUsageFile = serde_json::from_slice(&bytes).unwrap();
    let ledger = restored.session.to_ledger();

    assert_eq!(ledger.totals.input_tokens, 157);
    assert_eq!(ledger.totals.output_tokens, 17);
    assert_eq!(ledger.totals.model_calls, 3);
    assert_eq!(ledger.totals.api_duration_ms, 6_000);
    assert_eq!(ledger.totals.cost_usd_ticks, Some(125));
    assert_eq!(ledger.totals.cost_missing_calls, 1);
    assert_eq!(ledger.main_loop_model_calls, 2);
    assert!(ledger.main_loop_model_calls_known);
    assert_eq!(ledger.cache_session.hit_tokens(), 80);
    assert_eq!(ledger.cache_session.miss_tokens(), 20);
    assert_eq!(ledger.cache_session.reported(), 1);
    assert_eq!(ledger.cache_session.unreported(), 1);
    assert!(ledger.cache_session.history_complete());
    assert_eq!(ledger.by_model["subagent-model"].model_calls, 1);
    assert_eq!(
        ledger.main_loop_model_calls, 2,
        "subagent calls stay excluded"
    );
    assert!(ledger.incomplete);
}

#[test]
fn absent_legacy_fields_are_unknown_even_when_old_model_calls_are_zero() {
    let legacy: SessionUsageFile = serde_json::from_value(serde_json::json!({
        "sessionId": "legacy-empty-count",
        "session": {
            "inputTokens": 0,
            "outputTokens": 0,
            "modelCalls": 0
        }
    }))
    .unwrap();
    let restored = legacy.session.to_ledger();
    assert!(!restored.main_loop_model_calls_known);
    assert!(!restored.cache_session.history_complete());

    let explicit_zero: SessionUsageFile = serde_json::from_value(serde_json::json!({
        "sessionId": "new-empty-count",
        "session": {
            "inputTokens": 0,
            "outputTokens": 0,
            "modelCalls": 0,
            "mainLoopModelCalls": 0,
            "cacheSession": {
                "hitTokens": 0,
                "missTokens": 0,
                "reported": 0,
                "unreported": 0,
                "historyComplete": true
            }
        }
    }))
    .unwrap();
    let restored = explicit_zero.session.to_ledger();
    assert!(restored.main_loop_model_calls_known);
    assert_eq!(restored.main_loop_model_calls, 0);
    assert!(restored.cache_session.history_complete());
    assert_eq!(restored.cache_session.hit_tokens(), 0);
}

#[test]
fn legacy_usage_without_cache_summary_keeps_billing_and_marks_cache_unknown() {
    let legacy: SessionUsageFile = serde_json::from_value(serde_json::json!({
        "sessionId": "legacy-bill",
        "session": {
            "inputTokens": 123,
            "outputTokens": 45,
            "totalTokens": 168,
            "modelCalls": 4,
            "costUsdTicks": 900
        }
    }))
    .unwrap();
    let restored = legacy.session.to_ledger();
    assert_eq!(restored.totals.input_tokens, 123);
    assert_eq!(restored.totals.output_tokens, 45);
    assert_eq!(restored.totals.cost_usd_ticks, Some(900));
    assert!(!restored.cache_session.history_complete());
    assert!(!restored.main_loop_model_calls_known);
}

#[test]
fn malformed_partial_cache_summary_is_not_defaulted_to_zero() {
    let malformed = serde_json::json!({
        "session": {
            "modelCalls": 1,
            "cacheSession": {
                "hitTokens": 10,
                "missTokens": 0,
                "historyComplete": true
            }
        }
    });
    assert!(serde_json::from_value::<SessionUsageFile>(malformed).is_err());
}

#[test]
fn serialized_usage_does_not_restore_the_process_local_apply_cursor() {
    let mut file = SessionUsageFile::new("sess-cursor");
    let first = live(&[("grok-4", 100, 20, Some(50))]);
    file.apply_turn(1, "t1", &first, None);

    let bytes = serde_json::to_vec(&file).unwrap();
    let mut resumed: SessionUsageFile = serde_json::from_slice(&bytes).unwrap();
    let next = live(&[("grok-4", 10, 2, Some(5))]);
    let written = resumed.apply_turn(1, "resume-turn", &next, None);

    assert_eq!(written, 2);
    assert_eq!(resumed.turns.len(), 2);
    assert_eq!(resumed.turns[0].usage.input_tokens, 100);
    assert_eq!(resumed.turns[1].usage.input_tokens, 10);
    assert_eq!(resumed.session.input_tokens, 110);
}

#[test]
fn covers_detects_same_process_vs_reset_ledger() {
    let bigger = live(&[("m", 10, 1, None), ("m", 5, 1, None)]);
    let smaller = live(&[("m", 5, 1, None)]);
    assert!(bigger.covers(&smaller));
    assert!(!smaller.covers(&bigger));
    assert!(smaller.covers(&UsageSummary::default()));
}
