//! Agent loop: routing, tool-call repair, multi-turn transcript (M1).
//!
//! Specs: 15 (repair), 20 (routing), 30 (thinking wire via provider).
//! Sessions: JSONL persist/resume under `~/.deepseek-build/sessions/`.

mod loop_;
mod pairing;
mod parallel;
mod path_a_turn;
mod repair;
mod routing;
mod session;
mod subagent;

pub use loop_::{Agent, AgentConfig, TurnEvent, TurnOutcome};
pub use pairing::{InterruptedTool, PAIRING_INTERRUPTED_CONTENT, pair_tool_results};
pub use parallel::{MAX_PARALLEL_READONLY, is_mutating_tool, partition_indices};
pub use path_a_turn::{
    PathAToolCall, PathAToolPrep, path_a_default_router, path_a_flash_wire_id, path_a_pro_wire_id,
    prepare_path_a_tool_call, route_path_a_turn,
};
pub use repair::{RepairError, RepairOutcome, repair_tool_arguments};
pub use routing::{
    ModelRouter, Preset, RouteDecision, RouteSource, TurnModelOverride, apply_routing_command,
};
pub use session::{SessionError, SessionRecord, SessionStore, SessionSummary};
pub use subagent::{
    SubagentError, WorkerKind, WorkerOutcome, parent_after_worker, run_worker, worker_stable_prefix,
};
