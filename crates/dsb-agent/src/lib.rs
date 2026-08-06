//! Agent loop: routing, tool-call repair, multi-turn transcript (M1).
//!
//! Specs: 15 (repair), 20 (routing), 30 (thinking wire via provider).
//! Sessions: JSONL persist/resume under `~/.deepseek-build/sessions/`.

mod loop_;
mod pairing;
mod parallel;
mod repair;
mod routing;
mod session;
mod subagent;

pub use loop_::{Agent, AgentConfig, TurnEvent, TurnOutcome};
pub use pairing::{InterruptedTool, PAIRING_INTERRUPTED_CONTENT, pair_tool_results};
pub use parallel::{MAX_PARALLEL_READONLY, is_mutating_tool, partition_indices};
pub use repair::{RepairError, RepairOutcome, repair_tool_arguments};
pub use routing::{ModelRouter, Preset, RouteDecision, RouteSource, TurnModelOverride};
pub use session::{SessionError, SessionRecord, SessionStore, SessionSummary};
pub use subagent::{
    SubagentError, WorkerKind, WorkerOutcome, parent_after_worker, run_worker, worker_stable_prefix,
};
