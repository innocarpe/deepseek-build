//! Agent loop: routing, tool-call repair, multi-turn transcript (M1).
//!
//! Specs: 15 (repair), 20 (routing), 30 (thinking wire via provider).
//! Sessions: JSONL persist/resume under `~/.deepseek-build/sessions/`.

mod loop_;
mod pairing;
mod repair;
mod routing;
mod session;

pub use loop_::{Agent, AgentConfig, TurnEvent, TurnOutcome};
pub use pairing::{pair_tool_results, InterruptedTool, PAIRING_INTERRUPTED_CONTENT};
pub use repair::{repair_tool_arguments, RepairError, RepairOutcome};
pub use routing::{ModelRouter, Preset, RouteDecision, RouteSource, TurnModelOverride};
pub use session::{SessionError, SessionRecord, SessionStore, SessionSummary};
