//! Request builder with thinking/effort wire shape (spec 30, ADR 0005).

use serde::Serialize;

use crate::models::{
    ChatMessage, ModelId, ReasoningEffort, ThinkingMode, ToolChoice, ToolDefinition,
};

/// Default max_tokens for M1 (ADR 0005).
pub const DEFAULT_MAX_TOKENS: u32 = 8192;

#[derive(Debug, Clone, Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<ReasoningEffort>,
    /// DeepSeek thinking control (body field; OpenAI SDKs call this extra_body).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<ThinkingMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolDefinition>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,
    /// Omitted when thinking is enabled (no effect + false knobs).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct ChatRequestBuilder {
    model: ModelId,
    messages: Vec<ChatMessage>,
    stream: bool,
    max_tokens: Option<u32>,
    reasoning_effort: Option<ReasoningEffort>,
    thinking: Option<ThinkingMode>,
    tools: Option<Vec<ToolDefinition>>,
    tool_choice: Option<ToolChoice>,
    temperature: Option<f32>,
    top_p: Option<f32>,
}

impl ChatRequestBuilder {
    pub fn new(model: ModelId) -> Self {
        Self {
            model,
            messages: Vec::new(),
            stream: true,
            max_tokens: Some(DEFAULT_MAX_TOKENS),
            reasoning_effort: Some(ReasoningEffort::High),
            thinking: Some(ThinkingMode::enabled()),
            tools: None,
            tool_choice: None,
            temperature: None,
            top_p: None,
        }
    }

    pub fn messages(mut self, messages: Vec<ChatMessage>) -> Self {
        self.messages = messages;
        self
    }

    pub fn stream(mut self, stream: bool) -> Self {
        self.stream = stream;
        self
    }

    pub fn max_tokens(mut self, max_tokens: Option<u32>) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    pub fn reasoning_effort(mut self, effort: Option<ReasoningEffort>) -> Self {
        self.reasoning_effort = effort;
        self
    }

    pub fn thinking(mut self, thinking: Option<ThinkingMode>) -> Self {
        self.thinking = thinking;
        self
    }

    pub fn tools(mut self, tools: Option<Vec<ToolDefinition>>) -> Self {
        self.tools = tools;
        self
    }

    pub fn tool_choice(mut self, tool_choice: Option<ToolChoice>) -> Self {
        self.tool_choice = tool_choice;
        self
    }

    pub fn temperature(mut self, temperature: Option<f32>) -> Self {
        self.temperature = temperature;
        self
    }

    pub fn top_p(mut self, top_p: Option<f32>) -> Self {
        self.top_p = top_p;
        self
    }

    pub fn build(self) -> ChatRequest {
        let thinking_enabled = matches!(
            self.thinking.as_ref().map(|t| t.type_),
            Some(crate::models::ThinkingType::Enabled)
        );
        // Spec 30 / ADR 0005: omit sampling knobs when thinking is enabled.
        let (temperature, top_p) = if thinking_enabled {
            (None, None)
        } else {
            (self.temperature, self.top_p)
        };
        let reasoning_effort = if thinking_enabled {
            self.reasoning_effort
        } else {
            // When thinking disabled, effort may be omitted.
            self.reasoning_effort
        };

        ChatRequest {
            model: self.model.as_wire().to_string(),
            messages: self.messages,
            stream: self.stream,
            max_tokens: self.max_tokens,
            reasoning_effort,
            thinking: self.thinking,
            tools: self.tools,
            tool_choice: self.tool_choice,
            temperature,
            top_p,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ChatMessage, ThinkingType};

    #[test]
    fn request_includes_thinking_enabled() {
        let req = ChatRequestBuilder::new(ModelId::Flash)
            .messages(vec![ChatMessage::user("hi")])
            .build();
        let v = serde_json::to_value(&req).unwrap();
        assert_eq!(v["thinking"]["type"], "enabled");
        assert_eq!(v["model"], "deepseek-v4-flash");
    }

    #[test]
    fn request_includes_reasoning_effort() {
        let req = ChatRequestBuilder::new(ModelId::Pro)
            .reasoning_effort(Some(ReasoningEffort::Max))
            .messages(vec![ChatMessage::user("plan")])
            .build();
        let v = serde_json::to_value(&req).unwrap();
        assert_eq!(v["reasoning_effort"], "max");
        assert_eq!(v["model"], "deepseek-v4-pro");
    }

    #[test]
    fn omits_temperature_when_thinking() {
        let req = ChatRequestBuilder::new(ModelId::Flash)
            .thinking(Some(ThinkingMode::enabled()))
            .temperature(Some(0.7))
            .top_p(Some(0.9))
            .messages(vec![ChatMessage::user("hi")])
            .build();
        let v = serde_json::to_value(&req).unwrap();
        assert!(v.get("temperature").is_none());
        assert!(v.get("top_p").is_none());
        assert_eq!(v["thinking"]["type"], "enabled");
    }

    #[test]
    fn allows_temperature_when_thinking_disabled() {
        let req = ChatRequestBuilder::new(ModelId::Flash)
            .thinking(Some(ThinkingMode {
                type_: ThinkingType::Disabled,
            }))
            .temperature(Some(0.2))
            .messages(vec![ChatMessage::user("hi")])
            .build();
        let v = serde_json::to_value(&req).unwrap();
        assert!((v["temperature"].as_f64().unwrap() - 0.2).abs() < 1e-6);
        assert_eq!(v["thinking"]["type"], "disabled");
    }

    #[test]
    fn serializes_reasoning_content_on_assistant() {
        let msg = ChatMessage::assistant_with_reasoning(
            Some("answer".into()),
            Some("think step".into()),
            None,
        );
        let v = serde_json::to_value(&msg).unwrap();
        assert_eq!(v["reasoning_content"], "think step");
        assert_eq!(v["content"], "answer");
        assert_eq!(v["role"], "assistant");
    }

    #[test]
    fn stream_defaults_true() {
        let req = ChatRequestBuilder::new(ModelId::Flash)
            .messages(vec![ChatMessage::user("x")])
            .build();
        assert!(req.stream);
        let v = serde_json::to_value(&req).unwrap();
        assert_eq!(v["stream"], true);
    }
}
