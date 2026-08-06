//! DeepSeek Chat Completions provider (ADR 0005).
//!
//! - Base URL: `https://api.deepseek.com`
//! - Models: `deepseek-v4-flash` (default), `deepseek-v4-pro`
//! - Streaming SSE; separate `reasoning_content` vs `content`
//! - Thinking via body field `thinking: { type: enabled|disabled }`
//! - Effort via `reasoning_effort: low|high|max`
//! - With tools: always pass `reasoning_content` back on later calls

mod client;
mod error;
mod models;
mod request;
mod sse;
mod usage;

pub use client::{Client, ClientConfig, CompletedChat, StreamEvent};
pub use error::ProviderError;
pub use models::{
    AssistantMessage, ChatMessage, ContentPart, FunctionCall, ModelId, ReasoningEffort, Role,
    ThinkingMode, ThinkingType, ToolCall, ToolChoice, ToolDefinition, ToolFunction,
    MODEL_FLASH, MODEL_PRO,
};
pub use request::{ChatRequest, ChatRequestBuilder};
pub use usage::{CacheEvidence, Usage, UsageRaw};
