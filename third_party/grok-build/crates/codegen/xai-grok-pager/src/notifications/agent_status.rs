//! OSC 9999 agent-status reporting for hosts that speak it (Orca).
//!
//! Orca reads an explicit status stream out of the pane's PTY instead of
//! guessing a turn boundary from the tab title. Its wire format is a JSON
//! object in an OSC 9999 string — `ESC ] 9999 ; {json} BEL` — carrying one of
//! `working` / `blocked` / `waiting` / `done` plus an optional agent type,
//! model, and prompt preview.
//!
//! Why dsb sends it: without an explicit stream Orca has only the title to go
//! on, and `DeepSeek Build` names no agent it knows. A pane then reports no
//! status at all — the sidebar dot, the mobile row, and the turn-complete
//! notification all stay empty. The title cannot fix this on its own: Orca's
//! agent vocabulary is a closed set with no dsb member, so a dsb title is
//! either ignored or misread (the braille spinner reads as a working Claude).
//!
//! Why gated on the host: the sequence is meaningless outside a host that
//! parses it, so this module stays silent unless Orca is the host rather than
//! appending bytes to every terminal dsb runs in.
//!
//! Fail-soft: a payload that cannot be serialized, or a host that does not
//! speak the protocol, produces `None`. Status reporting is never a hard
//! dependency of the pager.

use std::sync::OnceLock;

use serde::Serialize;

use crate::notifications::tmux;
use crate::terminal::TerminalContext;

/// OSC code for explicit agent status. See Orca `src/shared/agent-status-osc.ts`.
pub const OSC_AGENT_STATUS_PREFIX: &str = "\x1b]9999;";

/// The agent id dsb reports itself as.
///
/// Why the canonical product name and not the `dsb` alias: a host keys launch
/// commands, icons, and telemetry on this id, so it has to name the binary the
/// host would spawn. `deepseek-build` is the primary command; `dsb` is its
/// alias.
pub const AGENT_TYPE_ID: &str = "deepseek-build";

/// Maximum characters kept from the session title used as the prompt preview.
/// Mirrors the host's own field cap so a long title is trimmed here rather
/// than silently truncated at the far end of the wire.
const MAX_PROMPT_CHARS: usize = 200;

/// Maximum characters kept from the model name.
const MAX_MODEL_CHARS: usize = 60;

/// The turn state dsb reports. Mirrors the host's `AgentStatusState`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentStatusState {
    /// A turn or command is running.
    Working,
    /// Blocked on the user — a pending permission or approval prompt.
    Waiting,
    /// Resting at the prompt; no turn in flight.
    Done,
}

impl AgentStatusState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Working => "working",
            Self::Waiting => "waiting",
            Self::Done => "done",
        }
    }
}

/// Whether the current host parses OSC 9999 agent status.
///
/// A parameter rather than an ambient read so every decision below is a pure
/// function of its inputs — the env lookup happens once, in [`host`], and the
/// tests never mutate process-global state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentStatusHost {
    Supported,
    Unsupported,
}

impl AgentStatusHost {
    pub fn is_supported(self) -> bool {
        matches!(self, Self::Supported)
    }
}

/// The wire payload. Field names are camelCase to match the host's parser.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AgentStatusPayload<'a> {
    state: &'a str,
    agent_type: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    prompt: Option<&'a str>,
}

/// Detect whether dsb is running inside a host that reads OSC 9999.
///
/// Orca stamps both variables into every pane it spawns, and either is
/// sufficient: `ORCA_AGENT_HOOK_ENDPOINT` survives a `TERM_PROGRAM` override,
/// and `TERM_PROGRAM` survives a stripped environment.
///
/// Cached: the answer cannot change while the process runs, and this is read
/// from a tick that fires at frame rate.
pub fn host() -> AgentStatusHost {
    static HOST: OnceLock<AgentStatusHost> = OnceLock::new();
    *HOST.get_or_init(|| {
        if std::env::var_os("ORCA_AGENT_HOOK_ENDPOINT").is_some_and(|v| !v.is_empty())
            || std::env::var("TERM_PROGRAM").is_ok_and(|v| v == "Orca")
        {
            AgentStatusHost::Supported
        } else {
            AgentStatusHost::Unsupported
        }
    })
}

/// Strip control characters so a model- or remote-sourced string cannot
/// terminate the OSC early and inject escapes into the terminal.
///
/// Same filter the tab title and notification bodies use; kept local so a
/// future change to one cannot silently weaken the other.
fn sanitize(value: &str, max_chars: usize) -> Option<String> {
    let cleaned: String = value.chars().filter(|c| !c.is_control()).collect();
    let trimmed = cleaned.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(trimmed.chars().take(max_chars).collect())
}

/// Build the OSC 9999 escape for one status frame.
///
/// Returns `None` when the host does not speak the protocol or the payload
/// cannot be serialized.
pub fn build_agent_status_escape(
    host: AgentStatusHost,
    state: AgentStatusState,
    model: Option<&str>,
    prompt: Option<&str>,
    ctx: &TerminalContext,
) -> Option<String> {
    if !host.is_supported() {
        return None;
    }
    // Bound to locals so the payload can borrow them for the whole call.
    let model = model.and_then(|m| sanitize(m, MAX_MODEL_CHARS));
    let prompt = prompt.and_then(|p| sanitize(p, MAX_PROMPT_CHARS));
    let payload = AgentStatusPayload {
        state: state.as_str(),
        agent_type: AGENT_TYPE_ID,
        model: model.as_deref(),
        prompt: prompt.as_deref(),
    };
    let json = serde_json::to_string(&payload).ok()?;
    let sequence = format!("{OSC_AGENT_STATUS_PREFIX}{json}\x07");
    if tmux::passthrough_available(ctx) {
        Some(tmux::tmux_passthrough(&sequence))
    } else {
        Some(sequence)
    }
}

/// Emits status frames, suppressing repeats of the last one.
///
/// Why dedup here rather than at the call site: the reporter is driven from
/// the same ticks that repaint the title, so without a cache a resting pane
/// would rewrite an identical frame several times a second.
#[derive(Debug, Default)]
pub struct AgentStatusReporter {
    last_frame: Option<String>,
}

impl AgentStatusReporter {
    pub fn new() -> Self {
        Self { last_frame: None }
    }

    /// The escape to emit for `state`, or `None` when it repeats the last one
    /// (or the host does not speak the protocol).
    pub fn frame_for(
        &mut self,
        host: AgentStatusHost,
        state: AgentStatusState,
        model: Option<&str>,
        prompt: Option<&str>,
        ctx: &TerminalContext,
    ) -> Option<String> {
        let escape = build_agent_status_escape(host, state, model, prompt, ctx)?;
        if self.last_frame.as_deref() == Some(escape.as_str()) {
            return None;
        }
        self.last_frame = Some(escape.clone());
        Some(escape)
    }

    /// Forget the last frame so the next call re-emits even when unchanged.
    ///
    /// Why: a host that restarted, or a pane whose stream was reset, has no
    /// memory of the frame dsb already sent — the dedup cache would starve it.
    pub fn reset(&mut self) {
        self.last_frame = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terminal::{MultiplexerKind, TerminalContext};

    const ON: AgentStatusHost = AgentStatusHost::Supported;
    const OFF: AgentStatusHost = AgentStatusHost::Unsupported;

    fn plain_ctx() -> TerminalContext {
        TerminalContext::default()
    }

    #[test]
    fn silent_outside_the_host() {
        assert_eq!(
            build_agent_status_escape(OFF, AgentStatusState::Done, None, None, &plain_ctx()),
            None
        );
    }

    #[test]
    fn frame_carries_state_agent_type_and_terminator() {
        let esc = build_agent_status_escape(
            ON,
            AgentStatusState::Working,
            Some("deepseek-v4-pro"),
            Some("my session"),
            &plain_ctx(),
        )
        .expect("host speaks the protocol");
        assert!(esc.starts_with(OSC_AGENT_STATUS_PREFIX));
        assert!(esc.ends_with('\x07'));
        assert!(esc.contains("\"state\":\"working\""));
        assert!(esc.contains("\"agentType\":\"deepseek-build\""));
        assert!(esc.contains("\"model\":\"deepseek-v4-pro\""));
        assert!(esc.contains("\"prompt\":\"my session\""));
    }

    #[test]
    fn every_state_has_a_wire_name() {
        for (state, name) in [
            (AgentStatusState::Working, "working"),
            (AgentStatusState::Waiting, "waiting"),
            (AgentStatusState::Done, "done"),
        ] {
            let esc =
                build_agent_status_escape(ON, state, None, None, &plain_ctx()).expect("emitted");
            assert!(esc.contains(&format!("\"state\":\"{name}\"")), "{esc}");
        }
    }

    #[test]
    fn optional_fields_are_omitted_not_nulled() {
        let esc = build_agent_status_escape(ON, AgentStatusState::Done, None, None, &plain_ctx())
            .expect("emitted");
        assert!(!esc.contains("model"), "{esc}");
        assert!(!esc.contains("prompt"), "{esc}");
    }

    #[test]
    fn control_characters_cannot_escape_the_osc() {
        let esc = build_agent_status_escape(
            ON,
            AgentStatusState::Working,
            Some("evil\x07\x1b]52;c;payload"),
            Some("session\x1b]0;spoofed\x07"),
            &plain_ctx(),
        )
        .expect("emitted");
        // Exactly one BEL — the frame's own terminator.
        assert_eq!(esc.matches('\x07').count(), 1, "{esc:?}");
        // The injected OSC introducers left with their ESC.
        assert!(!esc.contains("\x1b]52"), "{esc:?}");
        assert!(!esc.contains("\x1b]0;spoofed"), "{esc:?}");
    }

    #[test]
    fn overlong_fields_are_truncated() {
        let long_prompt = "p".repeat(MAX_PROMPT_CHARS * 3);
        let esc = build_agent_status_escape(
            ON,
            AgentStatusState::Done,
            None,
            Some(&long_prompt),
            &plain_ctx(),
        )
        .expect("emitted");
        assert!(esc.contains(&"p".repeat(MAX_PROMPT_CHARS)));
        assert!(!esc.contains(&"p".repeat(MAX_PROMPT_CHARS + 1)));
    }

    #[test]
    fn blank_fields_are_treated_as_absent() {
        let esc = build_agent_status_escape(
            ON,
            AgentStatusState::Done,
            Some("   "),
            Some(""),
            &plain_ctx(),
        )
        .expect("emitted");
        assert!(!esc.contains("model"), "{esc}");
        assert!(!esc.contains("prompt"), "{esc}");
    }

    #[test]
    fn tmux_wraps_the_frame_in_passthrough() {
        let ctx = TerminalContext {
            multiplexer: MultiplexerKind::Tmux,
            tmux_version: Some("tmux 3.3".into()),
            ..Default::default()
        };
        let esc = build_agent_status_escape(ON, AgentStatusState::Done, None, None, &ctx)
            .expect("emitted");
        assert!(esc.starts_with("\x1bPtmux;\x1b"), "{esc:?}");
        assert!(esc.ends_with("\x1b\\"), "{esc:?}");
    }

    #[test]
    fn reporter_suppresses_a_repeated_frame() {
        let mut reporter = AgentStatusReporter::new();
        let ctx = plain_ctx();
        assert!(
            reporter
                .frame_for(ON, AgentStatusState::Done, None, None, &ctx)
                .is_some()
        );
        assert!(
            reporter
                .frame_for(ON, AgentStatusState::Done, None, None, &ctx)
                .is_none()
        );
    }

    #[test]
    fn reporter_emits_on_every_transition() {
        let mut reporter = AgentStatusReporter::new();
        let ctx = plain_ctx();
        assert!(
            reporter
                .frame_for(ON, AgentStatusState::Working, None, None, &ctx)
                .is_some()
        );
        assert!(
            reporter
                .frame_for(ON, AgentStatusState::Done, None, None, &ctx)
                .is_some()
        );
        assert!(
            reporter
                .frame_for(ON, AgentStatusState::Working, None, None, &ctx)
                .is_some()
        );
    }

    #[test]
    fn reporter_reemits_after_reset() {
        let mut reporter = AgentStatusReporter::new();
        let ctx = plain_ctx();
        assert!(
            reporter
                .frame_for(ON, AgentStatusState::Done, None, None, &ctx)
                .is_some()
        );
        reporter.reset();
        assert!(
            reporter
                .frame_for(ON, AgentStatusState::Done, None, None, &ctx)
                .is_some()
        );
    }

    #[test]
    fn reporter_stays_silent_and_uncached_off_host() {
        let mut reporter = AgentStatusReporter::new();
        let ctx = plain_ctx();
        assert!(
            reporter
                .frame_for(OFF, AgentStatusState::Working, None, None, &ctx)
                .is_none()
        );
        // Nothing was cached, so the first on-host frame still emits.
        assert!(
            reporter
                .frame_for(ON, AgentStatusState::Working, None, None, &ctx)
                .is_some()
        );
    }

    #[test]
    fn reporter_retries_a_frame_that_never_emitted() {
        // A pane that goes quiet before the host is reachable must still be
        // able to report: the cache only holds frames actually produced.
        let mut reporter = AgentStatusReporter::new();
        let ctx = plain_ctx();
        assert!(
            reporter
                .frame_for(OFF, AgentStatusState::Done, None, None, &ctx)
                .is_none()
        );
        assert!(
            reporter
                .frame_for(OFF, AgentStatusState::Done, None, None, &ctx)
                .is_none()
        );
        assert!(
            reporter
                .frame_for(ON, AgentStatusState::Done, None, None, &ctx)
                .is_some()
        );
    }
}
