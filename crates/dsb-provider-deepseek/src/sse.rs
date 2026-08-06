//! SSE line parser for OpenAI-compatible chat completion streams.

use serde::Deserialize;

use crate::error::ProviderError;
use crate::models::{FunctionCall, ToolCall};
use crate::usage::{Usage, UsageRaw};

/// One logical event from a streaming chat completion.
#[derive(Debug, Clone, PartialEq)]
pub enum ParsedSse {
    /// Incremental reasoning / chain-of-thought.
    ReasoningDelta(String),
    /// Incremental visible content.
    ContentDelta(String),
    /// Tool call fragment (index-based accumulation).
    ToolCallDelta {
        index: usize,
        id: Option<String>,
        name: Option<String>,
        arguments: Option<String>,
    },
    /// Model id from chunk.
    Model(String),
    FinishReason(String),
    Usage(Usage),
    Done,
    /// Non-fatal skippable chunk.
    Ignore,
}

#[derive(Debug, Deserialize)]
struct StreamChunk {
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    choices: Vec<StreamChoice>,
    #[serde(default)]
    usage: Option<UsageRaw>,
}

#[derive(Debug, Deserialize)]
struct StreamChoice {
    #[serde(default)]
    delta: Option<StreamDelta>,
    #[serde(default)]
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct StreamDelta {
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    reasoning_content: Option<String>,
    #[serde(default)]
    tool_calls: Option<Vec<ToolCallDelta>>,
    #[serde(default)]
    role: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ToolCallDelta {
    #[serde(default)]
    index: Option<usize>,
    #[serde(default)]
    id: Option<String>,
    #[serde(default, rename = "type")]
    type_: Option<String>,
    #[serde(default)]
    function: Option<FunctionDelta>,
}

#[derive(Debug, Deserialize)]
struct FunctionDelta {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    arguments: Option<String>,
}

/// Parse one SSE `data:` payload (without the `data:` prefix).
pub fn parse_sse_data(data: &str) -> Result<Vec<ParsedSse>, ProviderError> {
    let data = data.trim();
    if data.is_empty() {
        return Ok(vec![ParsedSse::Ignore]);
    }
    if data == "[DONE]" {
        return Ok(vec![ParsedSse::Done]);
    }

    let chunk: StreamChunk = serde_json::from_str(data).map_err(|e| {
        ProviderError::InvalidSse(format!("json parse: {e}; data={}", truncate(data, 200)))
    })?;

    let mut out = Vec::new();

    if let Some(model) = chunk.model
        && !model.is_empty()
    {
        out.push(ParsedSse::Model(model));
    }

    if let Some(usage) = chunk.usage {
        out.push(ParsedSse::Usage(Usage::from_raw(usage)));
    }

    for choice in chunk.choices {
        if let Some(reason) = choice.finish_reason {
            out.push(ParsedSse::FinishReason(reason));
        }
        let Some(delta) = choice.delta else {
            continue;
        };
        // role-only deltas are ignored
        let _ = delta.role;

        if let Some(rc) = delta.reasoning_content
            && !rc.is_empty()
        {
            out.push(ParsedSse::ReasoningDelta(rc));
        }
        if let Some(c) = delta.content
            && !c.is_empty()
        {
            out.push(ParsedSse::ContentDelta(c));
        }
        if let Some(tcs) = delta.tool_calls {
            for tc in tcs {
                let index = tc.index.unwrap_or(0);
                let name = tc.function.as_ref().and_then(|f| f.name.clone());
                let arguments = tc.function.as_ref().and_then(|f| f.arguments.clone());
                out.push(ParsedSse::ToolCallDelta {
                    index,
                    id: tc.id,
                    name,
                    arguments,
                });
                let _ = tc.type_;
            }
        }
    }

    if out.is_empty() {
        out.push(ParsedSse::Ignore);
    }
    Ok(out)
}

/// Feed a multi-line SSE buffer chunk; return parsed events and remainder.
pub fn feed_sse_buffer(buffer: &mut String, chunk: &str) -> Result<Vec<ParsedSse>, ProviderError> {
    buffer.push_str(chunk);
    let mut events = Vec::new();

    loop {
        let Some(pos) = buffer.find('\n') else {
            break;
        };
        let mut line = buffer[..pos].to_string();
        buffer.drain(..=pos);
        if line.ends_with('\r') {
            line.pop();
        }
        if line.is_empty() {
            continue;
        }
        // SSE comments
        if line.starts_with(':') {
            continue;
        }
        if let Some(data) = line.strip_prefix("data:") {
            let data = data.trim_start();
            events.extend(parse_sse_data(data)?);
        }
        // event: / id: ignored for chat completions
    }

    Ok(events)
}

/// Accumulate tool call deltas into complete ToolCall values.
#[derive(Debug, Default)]
pub struct ToolCallAccumulator {
    slots: Vec<PartialToolCall>,
}

#[derive(Debug, Default, Clone)]
struct PartialToolCall {
    id: String,
    name: String,
    arguments: String,
}

impl ToolCallAccumulator {
    pub fn apply(
        &mut self,
        index: usize,
        id: Option<String>,
        name: Option<String>,
        arguments: Option<String>,
    ) {
        while self.slots.len() <= index {
            self.slots.push(PartialToolCall::default());
        }
        let slot = &mut self.slots[index];
        if let Some(id) = id
            && !id.is_empty()
        {
            slot.id = id;
        }
        if let Some(name) = name
            && !name.is_empty()
        {
            slot.name = name;
        }
        if let Some(args) = arguments {
            slot.arguments.push_str(&args);
        }
    }

    pub fn finish(self) -> Vec<ToolCall> {
        self.slots
            .into_iter()
            .filter(|s| !s.id.is_empty() || !s.name.is_empty() || !s.arguments.is_empty())
            .map(|s| ToolCall {
                id: if s.id.is_empty() {
                    format!("call_{}", s.name)
                } else {
                    s.id
                },
                type_: "function".into(),
                function: FunctionCall {
                    name: s.name,
                    arguments: s.arguments,
                },
            })
            .collect()
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_reasoning_and_content_deltas() {
        let data = r#"{"id":"x","choices":[{"delta":{"reasoning_content":"think"},"index":0}]}"#;
        let ev = parse_sse_data(data).unwrap();
        assert_eq!(ev, vec![ParsedSse::ReasoningDelta("think".into())]);

        let data = r#"{"choices":[{"delta":{"content":"hi"},"index":0}]}"#;
        let ev = parse_sse_data(data).unwrap();
        assert_eq!(ev, vec![ParsedSse::ContentDelta("hi".into())]);
    }

    #[test]
    fn parses_done() {
        assert_eq!(parse_sse_data("[DONE]").unwrap(), vec![ParsedSse::Done]);
    }

    #[test]
    fn feed_buffer_split_across_chunks() {
        let mut buf = String::new();
        let e1 = feed_sse_buffer(
            &mut buf,
            "data: {\"choices\":[{\"delta\":{\"content\":\"hel",
        )
        .unwrap();
        assert!(e1.is_empty());
        let e2 = feed_sse_buffer(&mut buf, "lo\"},\"index\":0}]}\n\ndata: [DONE]\n\n").unwrap();
        assert!(
            e2.iter()
                .any(|e| matches!(e, ParsedSse::ContentDelta(s) if s == "hello"))
        );
        assert!(e2.iter().any(|e| matches!(e, ParsedSse::Done)));
    }

    #[test]
    fn accumulates_tool_calls() {
        let mut acc = ToolCallAccumulator::default();
        acc.apply(
            0,
            Some("call_1".into()),
            Some("read".into()),
            Some("{\"p".into()),
        );
        acc.apply(0, None, None, Some("ath\":\"a\"}".into()));
        let tools = acc.finish();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].id, "call_1");
        assert_eq!(tools[0].function.name, "read");
        assert_eq!(tools[0].function.arguments, r#"{"path":"a"}"#);
    }

    #[test]
    fn parses_usage_in_chunk() {
        let data = r#"{"choices":[],"usage":{"prompt_tokens":10,"completion_tokens":2,"prompt_cache_hit_tokens":8}}"#;
        let ev = parse_sse_data(data).unwrap();
        match &ev[0] {
            ParsedSse::Usage(u) => {
                assert_eq!(u.prompt_tokens, Some(10));
                assert_eq!(u.cache_hit_tokens, Some(8));
            }
            other => panic!("expected usage, got {other:?}"),
        }
    }
}
