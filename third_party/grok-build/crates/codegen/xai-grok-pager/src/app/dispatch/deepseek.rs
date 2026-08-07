//! DeepSeek account status: balance + session usage for the bottom
//! status row. Mirrors the `x.ai/billing` + `x.ai/session/usage` wiring
//! but is fully auth-agnostic — the shell talks to the DeepSeek REST
//! API directly when the session targets a DeepSeek endpoint.
//!
//! The shell's `x.ai/deepseek/status` response carries an `is_deepseek`
//! flag because the pager cannot see per-session base URLs. The bottom
//! row renders only when that flag is true; a successful
//! `is_deepseek=false` also stops the polling timer so x.ai sessions
//! never burn a fetch every 60 seconds.

use agent_client_protocol as acp;

use crate::app::actions::Effect;
use crate::app::agent::AgentId;
use crate::app::agent_view::AgentView;
use crate::app::app_view::AppView;
use xai_grok_shell::extensions::deepseek::DeepSeekStatusResponse;

/// Whether the agent currently wants DeepSeek status refreshes.
///
/// Unknown (`None`) counts as "wanted" so the first fetch (session init)
/// discovers whether the session is DeepSeek-backed. A successful
/// `is_deepseek=false` response flips this to `false` and stops polling;
/// transient failures keep `None`/previous state so the next poll
/// self-heals.
fn deepseek_poll_wanted(agent: &AgentView) -> bool {
    agent
        .deepseek_status
        .as_ref()
        .map(|s| s.is_deepseek)
        .unwrap_or(true)
}

/// Fetch DeepSeek status after a turn (and on session init via the
/// callers in lifecycle/load). No-op when the agent has confirmed it is
/// not DeepSeek-backed or has no session id yet.
pub(crate) fn maybe_refresh_deepseek_status(
    app: &mut AppView,
    agent_id: AgentId,
) -> Vec<Effect> {
    let Some(agent) = app.agents.get(&agent_id) else {
        return vec![];
    };
    if !deepseek_poll_wanted(agent) {
        return vec![];
    }
    let Some(session_id) = agent.session.session_id.clone() else {
        return vec![];
    };
    vec![Effect::FetchDeepSeekStatus { agent_id, session_id }]
}

/// Store a fresh DeepSeek status on the agent. Drops the result when the
/// session has moved on (stale in-flight fetch).
pub(super) fn handle_deepseek_status_complete(
    app: &mut AppView,
    agent_id: AgentId,
    session_id: &acp::SessionId,
    status: DeepSeekStatusResponse,
) -> Vec<Effect> {
    if app
        .agents
        .get(&agent_id)
        .and_then(|a| a.session.session_id.as_ref())
        != Some(session_id)
    {
        return vec![];
    }
    if let Some(agent) = app.agents.get_mut(&agent_id) {
        agent.deepseek_status = Some(status);
    }
    vec![]
}

/// DeepSeek status fetch failed. Fail-soft: log and keep the previous
/// state (chip stays as-is); the next poll retries.
pub(super) fn handle_deepseek_status_failed(
    app: &mut AppView,
    agent_id: AgentId,
    session_id: &acp::SessionId,
    error: &str,
) -> Vec<Effect> {
    let _ = app;
    let _ = agent_id;
    let _ = session_id;
    tracing::debug!("deepseek status fetch failed: {error}");
    vec![]
}
