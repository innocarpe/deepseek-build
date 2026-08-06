//! Stable prefix / cache epoch builder (spec 10).
//!
//! ```text
//! messages_to_api =
//!   stable_prefix_messages   // byte-stable across turns when inputs unchanged
//!   + volatile_tail_messages // user, tool results, dynamic reminders
//! ```

mod canonicalize;
mod epoch;
mod prefix;
mod skills;

pub use canonicalize::{canonicalize_json, stable_prefix_bytes};
pub use epoch::PrefixEpoch;
pub use prefix::{
    discover_project_instructions, EnvironmentSummary, PrefixBuilder, PrefixBuildInputs,
    PrefixError, SkillIndexEntry, StablePrefix, VolatileTail, DEFAULT_SYSTEM_PROMPT,
};
pub use skills::{discover_skills_index, load_skill_body, SkillError};

use dsb_provider_deepseek::ChatMessage;

/// Full request message list = stable prefix + volatile tail.
pub fn assemble_messages(stable: &StablePrefix, tail: &VolatileTail) -> Vec<ChatMessage> {
    let mut out = stable.messages.clone();
    out.extend(tail.messages.clone());
    out
}
