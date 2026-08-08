//! Tests for the DeepSeek status dispatch: refresh gating, result storage,
//! and stale-session dropping.

use super::*;
use xai_grok_shell::extensions::deepseek::{DeepSeekBalance, DeepSeekStatusResponse};
use xai_grok_shell::extensions::notification::PromptUsage;

/// Minimal DeepSeek status fixture.
fn ds_status(is_deepseek: bool, balance: Option<DeepSeekBalance>) -> DeepSeekStatusResponse {
    DeepSeekStatusResponse {
        is_deepseek,
        balance,
        usage: PromptUsage::default(),
    }
}

fn usd_balance() -> DeepSeekBalance {
    DeepSeekBalance {
        currency: "USD".into(),
        total_balance: "9.82".into(),
        is_available: true,
    }
}

fn complete_deepseek(app: &mut AppView, session_id: &str, status: DeepSeekStatusResponse) {
    let _ = dispatch(
        Action::TaskComplete(TaskResult::DeepSeekStatusComplete {
            agent_id: AgentId(0),
            session_id: session_id.to_string().into(),
            status: Box::new(status),
        }),
        app,
    );
}

// ── maybe_refresh gating ───────────────────────────────────────────

#[test]
fn refresh_fires_when_status_unknown() {
    let mut app = test_app_with_agent();
    let effects = maybe_refresh_deepseek_status(&mut app, AgentId(0));
    assert!(
        matches!(
            effects.as_slice(),
            [Effect::FetchDeepSeekStatus { agent_id, .. }] if *agent_id == AgentId(0)
        ),
        "unknown status must still probe: {effects:?}"
    );
}

#[test]
fn refresh_fires_when_deepseek_confirmed() {
    let mut app = test_app_with_agent();
    complete_deepseek(&mut app, "test-session", ds_status(true, None));
    let effects = maybe_refresh_deepseek_status(&mut app, AgentId(0));
    assert!(
        matches!(
            effects.as_slice(),
            [Effect::FetchDeepSeekStatus { agent_id, .. }] if *agent_id == AgentId(0)
        ),
        "confirmed DeepSeek session keeps refreshing: {effects:?}"
    );
}

#[test]
fn refresh_skips_when_not_deepseek() {
    let mut app = test_app_with_agent();
    complete_deepseek(&mut app, "test-session", ds_status(false, None));
    let effects = maybe_refresh_deepseek_status(&mut app, AgentId(0));
    assert!(
        effects.is_empty(),
        "non-DeepSeek session must not keep polling: {effects:?}"
    );
}

#[test]
fn refresh_skips_without_session_id() {
    let mut app = test_app_with_agent();
    app.agents.get_mut(&AgentId(0)).unwrap().session.session_id = None;
    let effects = maybe_refresh_deepseek_status(&mut app, AgentId(0));
    assert!(effects.is_empty(), "no session id → no fetch: {effects:?}");
}

// ── result handling ────────────────────────────────────────────────

#[test]
fn complete_stores_status_on_agent() {
    let mut app = test_app_with_agent();
    let status = ds_status(true, Some(usd_balance()));
    complete_deepseek(&mut app, "test-session", status);
    let stored = app
        .agents
        .get(&AgentId(0))
        .and_then(|a| a.deepseek_status.as_ref())
        .expect("status stored");
    assert!(stored.is_deepseek);
    assert_eq!(
        stored.balance.as_ref().map(|b| b.currency.as_str()),
        Some("USD")
    );
}

#[test]
fn complete_drops_stale_session_result() {
    let mut app = test_app_with_agent();
    complete_deepseek(
        &mut app,
        "stale-session",
        ds_status(true, Some(usd_balance())),
    );
    assert!(
        app.agents
            .get(&AgentId(0))
            .unwrap()
            .deepseek_status
            .is_none(),
        "result for a different session must be dropped"
    );
}

#[test]
fn failed_is_silent_and_keeps_state() {
    let mut app = test_app_with_agent();
    complete_deepseek(
        &mut app,
        "test-session",
        ds_status(true, Some(usd_balance())),
    );
    let effects = dispatch(
        Action::TaskComplete(TaskResult::DeepSeekStatusFailed {
            agent_id: AgentId(0),
            session_id: "test-session".to_string().into(),
            error: "boom".into(),
        }),
        &mut app,
    );
    assert!(effects.is_empty(), "failure is silent: {effects:?}");
    let stored = app
        .agents
        .get(&AgentId(0))
        .unwrap()
        .deepseek_status
        .as_ref();
    assert!(
        stored.is_some_and(|s| s.is_deepseek),
        "previous status must survive a failed refresh"
    );
}

#[test]
fn transient_failure_keeps_polling_for_current_session() {
    let mut app = test_app_with_agent();
    let effects = dispatch(
        Action::TaskComplete(TaskResult::DeepSeekStatusFailed {
            agent_id: AgentId(0),
            session_id: "test-session".to_string().into(),
            error: "connection reset".into(),
        }),
        &mut app,
    );
    assert!(effects.is_empty(), "failure is silent: {effects:?}");
    assert!(app.deepseek_poll_wanted(), "transient failure must retry");
    assert_eq!(maybe_refresh_deepseek_status(&mut app, AgentId(0)).len(), 1);
}

#[test]
fn unsupported_status_stops_only_matching_session() {
    let mut app = test_app_with_agent();
    let _ = dispatch(
        Action::TaskComplete(TaskResult::DeepSeekStatusFailed {
            agent_id: AgentId(0),
            session_id: "test-session".to_string().into(),
            error: "not supported by this agent version".into(),
        }),
        &mut app,
    );
    assert!(
        !app.deepseek_poll_wanted(),
        "unsupported capability is terminal"
    );

    let _ = dispatch(
        Action::TaskComplete(TaskResult::DeepSeekStatusFailed {
            agent_id: AgentId(0),
            session_id: "old-session".to_string().into(),
            error: "not supported by this agent version".into(),
        }),
        &mut app,
    );
    assert!(
        !app.deepseek_poll_wanted(),
        "stale failure cannot reopen polling"
    );
}

#[test]
fn stale_unsupported_failure_does_not_disable_new_session() {
    let mut app = test_app_with_agent();
    app.agents.get_mut(&AgentId(0)).unwrap().session.session_id = Some("new-session".into());
    let _ = dispatch(
        Action::TaskComplete(TaskResult::DeepSeekStatusFailed {
            agent_id: AgentId(0),
            session_id: "old-session".to_string().into(),
            error: "not supported by this agent version".into(),
        }),
        &mut app,
    );
    assert!(
        app.deepseek_poll_wanted(),
        "new session must retain first probe"
    );
    assert_eq!(maybe_refresh_deepseek_status(&mut app, AgentId(0)).len(), 1);
}

#[test]
fn not_deepseek_flips_poll_wanted_off() {
    let mut app = test_app_with_agent();
    complete_deepseek(&mut app, "test-session", ds_status(false, None));
    assert!(
        !app.deepseek_poll_wanted(),
        "confirmed non-DeepSeek session stops the poll timer"
    );
    // Unknown (no status) still wants polling.
    let fresh = test_app_with_agent();
    assert!(fresh.deepseek_poll_wanted());
}
