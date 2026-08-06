//! HTTP client for DeepSeek Chat Completions.

use std::time::Duration;

use futures_util::StreamExt;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::Deserialize;

use crate::error::ProviderError;
use crate::models::AssistantMessage;
use crate::request::ChatRequest;
use crate::sse::{feed_sse_buffer, parse_sse_data, ParsedSse, ToolCallAccumulator};
use crate::usage::{CacheEvidence, Usage, UsageRaw};

/// Default DeepSeek OpenAI-compatible base URL (ADR 0005).
pub const DEFAULT_BASE_URL: &str = "https://api.deepseek.com";

#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub base_url: String,
    pub api_key: String,
    pub timeout: Duration,
    pub max_retries: u32,
}

impl ClientConfig {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            base_url: DEFAULT_BASE_URL.to_string(),
            api_key: api_key.into(),
            timeout: Duration::from_secs(120),
            max_retries: 3,
        }
    }

    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into().trim_end_matches('/').to_string();
        self
    }
}

/// Public stream events for consumers (agent / CLI).
#[derive(Debug, Clone, PartialEq)]
pub enum StreamEvent {
    ReasoningDelta(String),
    ContentDelta(String),
    ToolCallDelta {
        index: usize,
        id: Option<String>,
        name: Option<String>,
        arguments: Option<String>,
    },
    Model(String),
    FinishReason(String),
    Usage(Usage),
    Done,
}

#[derive(Debug, Clone)]
pub struct CompletedChat {
    pub message: AssistantMessage,
    pub usage: Option<Usage>,
    pub cache_evidence: Option<CacheEvidence>,
    pub model: Option<String>,
}

#[derive(Clone)]
pub struct Client {
    http: reqwest::Client,
    config: ClientConfig,
}

impl Client {
    pub fn new(config: ClientConfig) -> Result<Self, ProviderError> {
        if config.api_key.trim().is_empty() {
            return Err(ProviderError::MissingApiKey);
        }
        let http = reqwest::Client::builder()
            .timeout(config.timeout)
            .build()?;
        Ok(Self { http, config })
    }

    pub fn config(&self) -> &ClientConfig {
        &self.config
    }

    fn chat_url(&self) -> String {
        format!("{}/chat/completions", self.config.base_url)
    }

    /// Non-streaming chat completion.
    pub async fn chat(&self, request: ChatRequest) -> Result<CompletedChat, ProviderError> {
        let mut req = request;
        req.stream = false;

        let mut attempt = 0u32;
        loop {
            attempt += 1;
            match self.chat_once(&req).await {
                Ok(c) => return Ok(c),
                Err(e) if Self::is_retryable(&e) && attempt <= self.config.max_retries => {
                    let backoff = Duration::from_millis(200 * 2u64.pow(attempt.saturating_sub(1)));
                    tokio::time::sleep(backoff).await;
                }
                Err(e) => return Err(e),
            }
        }
    }

    async fn chat_once(&self, request: &ChatRequest) -> Result<CompletedChat, ProviderError> {
        let resp = self
            .http
            .post(self.chat_url())
            .header(AUTHORIZATION, format!("Bearer {}", self.config.api_key))
            .header(CONTENT_TYPE, "application/json")
            .json(request)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(ProviderError::ApiStatus {
                status: status.as_u16(),
                body,
            });
        }

        let body: NonStreamResponse = resp.json().await?;
        let choice = body
            .choices
            .into_iter()
            .next()
            .ok_or_else(|| ProviderError::Message("empty choices".into()))?;

        let msg = choice.message.unwrap_or_default();
        let usage = body.usage.map(Usage::from_raw);
        let cache_evidence = usage.as_ref().and_then(CacheEvidence::from_usage);

        Ok(CompletedChat {
            message: AssistantMessage {
                content: msg.content.unwrap_or_default(),
                reasoning_content: msg.reasoning_content.unwrap_or_default(),
                tool_calls: msg.tool_calls.unwrap_or_default(),
                finish_reason: choice.finish_reason,
                model: body.model.clone(),
            },
            usage,
            cache_evidence,
            model: body.model,
        })
    }

    /// Streaming chat; returns accumulated assistant message + usage.
    pub async fn chat_stream<F>(
        &self,
        request: ChatRequest,
        mut on_event: F,
    ) -> Result<CompletedChat, ProviderError>
    where
        F: FnMut(StreamEvent),
    {
        let mut req = request;
        req.stream = true;

        let mut attempt = 0u32;
        loop {
            attempt += 1;
            match self.chat_stream_once(&req, &mut on_event).await {
                Ok(c) => return Ok(c),
                Err(e) if Self::is_retryable(&e) && attempt <= self.config.max_retries => {
                    let backoff = Duration::from_millis(200 * 2u64.pow(attempt.saturating_sub(1)));
                    tokio::time::sleep(backoff).await;
                }
                Err(e) => return Err(e),
            }
        }
    }

    async fn chat_stream_once<F>(
        &self,
        request: &ChatRequest,
        on_event: &mut F,
    ) -> Result<CompletedChat, ProviderError>
    where
        F: FnMut(StreamEvent),
    {
        let resp = self
            .http
            .post(self.chat_url())
            .header(AUTHORIZATION, format!("Bearer {}", self.config.api_key))
            .header(CONTENT_TYPE, "application/json")
            .json(request)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(ProviderError::ApiStatus {
                status: status.as_u16(),
                body,
            });
        }

        let mut stream = resp.bytes_stream();
        let mut buffer = String::new();
        let mut content = String::new();
        let mut reasoning = String::new();
        let mut tool_acc = ToolCallAccumulator::default();
        let mut finish_reason = None;
        let mut model = None;
        let mut usage = None;
        let mut done = false;

        while let Some(item) = stream.next().await {
            let bytes = item.map_err(|e| ProviderError::StreamInterrupted(e.to_string()))?;
            let chunk = String::from_utf8_lossy(&bytes);
            let events = feed_sse_buffer(&mut buffer, &chunk)?;
            for ev in events {
                apply_parsed(
                    ev,
                    &mut content,
                    &mut reasoning,
                    &mut tool_acc,
                    &mut finish_reason,
                    &mut model,
                    &mut usage,
                    &mut done,
                    on_event,
                );
            }
        }

        // Flush remainder
        if !buffer.trim().is_empty() {
            if let Some(data) = buffer.trim().strip_prefix("data:") {
                for ev in parse_sse_data(data.trim_start())? {
                    apply_parsed(
                        ev,
                        &mut content,
                        &mut reasoning,
                        &mut tool_acc,
                        &mut finish_reason,
                        &mut model,
                        &mut usage,
                        &mut done,
                        on_event,
                    );
                }
            }
        }

        let cache_evidence = usage.as_ref().and_then(CacheEvidence::from_usage);
        Ok(CompletedChat {
            message: AssistantMessage {
                content,
                reasoning_content: reasoning,
                tool_calls: tool_acc.finish(),
                finish_reason,
                model: model.clone(),
            },
            usage,
            cache_evidence,
            model,
        })
    }

    fn is_retryable(err: &ProviderError) -> bool {
        match err {
            ProviderError::Http(e) => e.is_timeout() || e.is_connect() || e.is_request(),
            ProviderError::ApiStatus { status, .. } => {
                *status == 429 || (500..600).contains(status)
            }
            ProviderError::StreamInterrupted(_) => true,
            _ => false,
        }
    }

    /// Dual-call substitute cache evidence when usage lacks cache fields (ADR 0005).
    ///
    /// Issues two identical non-stream requests and records latency + prompt tokens.
    pub async fn cache_evidence_dual_call(
        &self,
        request: ChatRequest,
    ) -> Result<CacheEvidence, ProviderError> {
        let mut req = request;
        req.stream = false;

        let t0 = std::time::Instant::now();
        let first = self.chat_once(&req).await?;
        let first_latency_ms = t0.elapsed().as_millis() as u64;

        let t1 = std::time::Instant::now();
        let second = self.chat_once(&req).await?;
        let second_latency_ms = t1.elapsed().as_millis() as u64;

        Ok(CacheEvidence::SubstituteDualCall {
            first_latency_ms,
            second_latency_ms,
            first_prompt_tokens: first.usage.as_ref().and_then(|u| u.prompt_tokens),
            second_prompt_tokens: second.usage.as_ref().and_then(|u| u.prompt_tokens),
        })
    }
}

fn apply_parsed<F>(
    ev: ParsedSse,
    content: &mut String,
    reasoning: &mut String,
    tool_acc: &mut ToolCallAccumulator,
    finish_reason: &mut Option<String>,
    model: &mut Option<String>,
    usage: &mut Option<Usage>,
    done: &mut bool,
    on_event: &mut F,
) where
    F: FnMut(StreamEvent),
{
    match ev {
        ParsedSse::ReasoningDelta(s) => {
            reasoning.push_str(&s);
            on_event(StreamEvent::ReasoningDelta(s));
        }
        ParsedSse::ContentDelta(s) => {
            content.push_str(&s);
            on_event(StreamEvent::ContentDelta(s));
        }
        ParsedSse::ToolCallDelta {
            index,
            id,
            name,
            arguments,
        } => {
            tool_acc.apply(index, id.clone(), name.clone(), arguments.clone());
            on_event(StreamEvent::ToolCallDelta {
                index,
                id,
                name,
                arguments,
            });
        }
        ParsedSse::Model(m) => {
            *model = Some(m.clone());
            on_event(StreamEvent::Model(m));
        }
        ParsedSse::FinishReason(r) => {
            *finish_reason = Some(r.clone());
            on_event(StreamEvent::FinishReason(r));
        }
        ParsedSse::Usage(u) => {
            *usage = Some(u.clone());
            on_event(StreamEvent::Usage(u));
        }
        ParsedSse::Done => {
            *done = true;
            on_event(StreamEvent::Done);
        }
        ParsedSse::Ignore => {}
    }
}

#[derive(Debug, Deserialize)]
struct NonStreamResponse {
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    choices: Vec<NonStreamChoice>,
    #[serde(default)]
    usage: Option<UsageRaw>,
}

#[derive(Debug, Deserialize)]
struct NonStreamChoice {
    #[serde(default)]
    message: Option<NonStreamMessage>,
    #[serde(default)]
    finish_reason: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct NonStreamMessage {
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    reasoning_content: Option<String>,
    #[serde(default)]
    tool_calls: Option<Vec<crate::models::ToolCall>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ChatMessage, ModelId, ReasoningEffort, ThinkingMode};
    use crate::request::ChatRequestBuilder;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn stream_accumulates_reasoning_and_content() {
        let server = MockServer::start().await;
        let body = concat!(
            "data: {\"model\":\"deepseek-v4-flash\",\"choices\":[{\"delta\":{\"reasoning_content\":\"r1\"}}]}\n\n",
            "data: {\"choices\":[{\"delta\":{\"content\":\"hello\"}}]}\n\n",
            "data: {\"choices\":[{\"delta\":{\"content\":\" world\"},\"finish_reason\":\"stop\"}]}\n\n",
            "data: {\"usage\":{\"prompt_tokens\":5,\"completion_tokens\":2,\"prompt_cache_hit_tokens\":3}}\n\n",
            "data: [DONE]\n\n",
        );
        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .and(header("authorization", "Bearer test-key"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("content-type", "text/event-stream")
                    .set_body_string(body),
            )
            .mount(&server)
            .await;

        let client = Client::new(ClientConfig::new("test-key").with_base_url(server.uri())).unwrap();
        let req = ChatRequestBuilder::new(ModelId::Flash)
            .messages(vec![ChatMessage::user("hi")])
            .build();

        let mut reasoning_parts = Vec::new();
        let mut content_parts = Vec::new();
        let completed = client
            .chat_stream(req, |ev| match ev {
                StreamEvent::ReasoningDelta(s) => reasoning_parts.push(s),
                StreamEvent::ContentDelta(s) => content_parts.push(s),
                _ => {}
            })
            .await
            .unwrap();

        assert_eq!(completed.message.reasoning_content, "r1");
        assert_eq!(completed.message.content, "hello world");
        assert_eq!(completed.model.as_deref(), Some("deepseek-v4-flash"));
        assert!(completed.cache_evidence.is_some());
        assert_eq!(reasoning_parts, vec!["r1".to_string()]);
        assert_eq!(content_parts, vec!["hello".to_string(), " world".to_string()]);
    }

    #[tokio::test]
    async fn non_stream_request_shape_includes_thinking() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "model": "deepseek-v4-pro",
                "choices": [{
                    "message": {
                        "role": "assistant",
                        "content": "ok",
                        "reasoning_content": "thought"
                    },
                    "finish_reason": "stop"
                }],
                "usage": {"prompt_tokens": 1, "completion_tokens": 1}
            })))
            .mount(&server)
            .await;

        let client = Client::new(ClientConfig::new("k").with_base_url(server.uri())).unwrap();
        let req = ChatRequestBuilder::new(ModelId::Pro)
            .stream(false)
            .reasoning_effort(Some(ReasoningEffort::Max))
            .thinking(Some(ThinkingMode::enabled()))
            .messages(vec![ChatMessage::user("x")])
            .build();

        // Capture request body via wiremock received requests after call
        let completed = client.chat(req).await.unwrap();
        assert_eq!(completed.message.content, "ok");
        assert_eq!(completed.message.reasoning_content, "thought");
        assert_eq!(completed.model.as_deref(), Some("deepseek-v4-pro"));

        let requests = server.received_requests().await.unwrap();
        assert_eq!(requests.len(), 1);
        let body: serde_json::Value = serde_json::from_slice(&requests[0].body).unwrap();
        assert_eq!(body["model"], "deepseek-v4-pro");
        assert_eq!(body["thinking"]["type"], "enabled");
        assert_eq!(body["reasoning_effort"], "max");
        assert!(body.get("temperature").is_none());
        assert_eq!(body["stream"], false);
    }

    #[tokio::test]
    async fn api_error_surfaces_status() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(400).set_body_string(
                r#"{"error":{"message":"Missing reasoning_content"}}"#,
            ))
            .mount(&server)
            .await;

        let client = Client::new(ClientConfig::new("k").with_base_url(server.uri())).unwrap();
        let req = ChatRequestBuilder::new(ModelId::Flash)
            .stream(false)
            .messages(vec![ChatMessage::user("x")])
            .build();
        let err = client.chat(req).await.unwrap_err();
        match err {
            ProviderError::ApiStatus { status, body } => {
                assert_eq!(status, 400);
                assert!(body.contains("reasoning_content"));
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[tokio::test]
    async fn dual_call_substitute_protocol() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "model": "deepseek-v4-flash",
                "choices": [{"message": {"role":"assistant","content":"a"}, "finish_reason":"stop"}],
                "usage": {"prompt_tokens": 42, "completion_tokens": 1}
            })))
            .expect(2)
            .mount(&server)
            .await;

        let client = Client::new(ClientConfig::new("k").with_base_url(server.uri())).unwrap();
        let req = ChatRequestBuilder::new(ModelId::Flash)
            .messages(vec![ChatMessage::system("stable"), ChatMessage::user("t")])
            .build();
        let ev = client.cache_evidence_dual_call(req).await.unwrap();
        match ev {
            CacheEvidence::SubstituteDualCall {
                first_prompt_tokens,
                second_prompt_tokens,
                ..
            } => {
                assert_eq!(first_prompt_tokens, Some(42));
                assert_eq!(second_prompt_tokens, Some(42));
            }
            other => panic!("expected substitute, got {other:?}"),
        }
        assert_eq!(ev.log_label(), "cache_evidence=substitute_dual_call");
    }
}
