//! Multi-turn agent loop: routing, repair, tools (spec 45/90).

use std::path::PathBuf;
use std::sync::Arc;

use dsb_context::{
    assemble_messages, discover_project_instructions, discover_skills_index, EnvironmentSummary,
    PrefixBuildInputs, PrefixBuilder, SkillIndexEntry, StablePrefix, VolatileTail,
    DEFAULT_SYSTEM_PROMPT,
};
use dsb_provider_deepseek::ReasoningEffort;
use dsb_provider_deepseek::{
    ChatMessage, ChatRequestBuilder, Client, ModelId, ProviderError, StreamEvent, ToolCall,
    ToolDefinition, ThinkingMode, MODEL_PRO,
};
use dsb_tools::{
    catalog_from_config, catalog_tool_definitions, default_coding_policy, dogfood_coding_policy,
    load_mcp_config, tool_definitions, AskCallback, PermissionPolicy, Scope, ToolExecutor,
    ToolName, ToolRequest,
};
use thiserror::Error;

use crate::pairing::{pair_tool_results, tools_in_play, InterruptedTool};
use crate::repair::{repair_tool_arguments, RepairError};
use crate::routing::{apply_routing_command, ModelRouter, Preset, RouteDecision};
use crate::session::{SessionError, SessionStore};

#[derive(Debug, Error)]
pub enum AgentError {
    #[error(transparent)]
    Provider(#[from] ProviderError),
    #[error(transparent)]
    Context(#[from] dsb_context::PrefixError),
    #[error("repair: {0}")]
    Repair(#[from] RepairError),
    #[error(transparent)]
    Session(#[from] SessionError),
    #[error("{0}")]
    Message(String),
}

#[derive(Clone)]
pub struct AgentConfig {
    pub workspace_root: PathBuf,
    pub system_prompt: String,
    pub tools: Vec<ToolDefinition>,
    pub skills_index: Vec<SkillIndexEntry>,
    pub preset: Preset,
    pub max_tool_rounds: u32,
    /// When true, print model visibility lines via TurnEvent.
    pub show_model: bool,
    /// Headless permission policy (ask → deny when no TTY / no ask callback).
    pub headless: bool,
    /// Allow workspace writes without interactive ask (still denies out-of-cwd).
    pub allow_workspace_write: bool,
    /// Actually execute bash (default false: classify + permission only).
    pub bash_execute: bool,
    /// Trusted local dogfood profile: workspace write + bash execute under policy.
    /// Still denies write/delete outside the workspace.
    pub dogfood: bool,
    /// Optional user skills directory (`~/.deepseek-build/skills`).
    pub user_skills_root: Option<PathBuf>,
    /// User config home for permission grants (`permission-grants.json`).
    pub grants_home: Option<PathBuf>,
    /// Interactive ask callback (allow once / always / deny). Requires `headless: false`.
    pub ask_callback: Option<AskCallback>,
    /// When true, auto-discover skills into the stable index.
    pub discover_skills: bool,
    /// Override reasoning effort for all turns (CLI `--effort`).
    pub effort_override: Option<ReasoningEffort>,
    /// When Some(false), disable thinking for the session.
    pub thinking_enabled: Option<bool>,
}

impl std::fmt::Debug for AgentConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgentConfig")
            .field("workspace_root", &self.workspace_root)
            .field("preset", &self.preset)
            .field("headless", &self.headless)
            .field("allow_workspace_write", &self.allow_workspace_write)
            .field("bash_execute", &self.bash_execute)
            .field("dogfood", &self.dogfood)
            .field("grants_home", &self.grants_home)
            .field("ask_callback", &self.ask_callback.as_ref().map(|_| "<fn>"))
            .field("discover_skills", &self.discover_skills)
            .field("effort_override", &self.effort_override)
            .field("thinking_enabled", &self.thinking_enabled)
            .finish_non_exhaustive()
    }
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            workspace_root: PathBuf::from("."),
            system_prompt: DEFAULT_SYSTEM_PROMPT.to_string(),
            tools: tool_definitions(),
            skills_index: Vec::new(),
            preset: Preset::Flash,
            max_tool_rounds: 8,
            show_model: true,
            headless: true,
            allow_workspace_write: false,
            bash_execute: false,
            dogfood: false,
            user_skills_root: None,
            grants_home: None,
            ask_callback: None,
            discover_skills: true,
            effort_override: None,
            thinking_enabled: None,
        }
    }
}

fn build_policy(cfg: &AgentConfig) -> PermissionPolicy {
    if cfg.dogfood || cfg.allow_workspace_write {
        let mut p = if cfg.dogfood {
            dogfood_coding_policy(cfg.headless)
        } else {
            default_coding_policy(cfg.headless)
        };
        if cfg.allow_workspace_write || cfg.dogfood {
            p.allow.insert(Scope::WriteInCwd);
            p.allow.insert(Scope::DeleteInCwd);
            p.ask.remove(&Scope::WriteInCwd);
            p.ask.remove(&Scope::DeleteInCwd);
        }
        p
    } else {
        default_coding_policy(cfg.headless)
    }
}

/// Events emitted during a turn (for CLI rendering).
#[derive(Debug, Clone)]
pub enum TurnEvent {
    ModelVisibility(String),
    ReasoningDelta(String),
    ContentDelta(String),
    ToolCallProposed { name: String, arguments: String },
    ToolRepairApplied { name: String },
    ToolError { name: String, error: String },
    Warning(String),
    PrefixEpoch(String),
    CacheEvidence(String),
}

#[derive(Debug, Clone)]
pub struct TurnOutcome {
    pub assistant_text: String,
    pub reasoning_text: String,
    pub route: RouteDecision,
    pub model_used: String,
    pub tool_rounds: u32,
}

/// Session agent holding stable prefix + volatile transcript.
pub struct Agent {
    client: Arc<Client>,
    config: AgentConfig,
    router: ModelRouter,
    stable: StablePrefix,
    /// Volatile transcript (user/assistant/tool after stable prefix).
    tail: VolatileTail,
    tools: ToolExecutor,
}

impl Agent {
    pub fn new(client: Arc<Client>, config: AgentConfig) -> Result<Self, AgentError> {
        let project_instructions = discover_project_instructions(&config.workspace_root)?;
        let environment = EnvironmentSummary::detect(&config.workspace_root);
        let skills_index = if config.discover_skills && config.skills_index.is_empty() {
            discover_skills_index(
                &config.workspace_root,
                config.user_skills_root.as_deref(),
            )
            .unwrap_or_default()
        } else {
            config.skills_index.clone()
        };
        // MCP catalog (spec 80): load static catalogs; fingerprint joins tool schemas for epoch.
        let mcp_cfg = load_mcp_config(
            &config.workspace_root,
            config.grants_home.as_deref(),
        )
        .unwrap_or_default();
        let mcp_catalog = catalog_from_config(&mcp_cfg).unwrap_or_default();
        let mut tools_defs = if config.tools.is_empty() {
            tool_definitions()
        } else {
            config.tools.clone()
        };
        if !mcp_catalog.is_empty() {
            tools_defs.extend(catalog_tool_definitions(&mcp_catalog));
            // Embed fingerprint note in system via tools doc already; also stamp description.
            let _fp = mcp_catalog.fingerprint_hex.clone();
        }
        let inputs = PrefixBuildInputs {
            system_prompt: config.system_prompt.clone(),
            tools: tools_defs,
            skills_index,
            environment,
            project_instructions,
        };
        let stable = PrefixBuilder::new().build(&inputs)?;
        let router = ModelRouter::new(config.preset);
        let policy = build_policy(&config);
        let mut tools = ToolExecutor::new(config.workspace_root.clone(), policy);
        tools.bash_execute = config.bash_execute || config.dogfood;
        tools.user_skills_root = config.user_skills_root.clone();
        tools.mcp_catalog = mcp_catalog;
        if let Some(home) = &config.grants_home {
            tools = tools.with_grants_home(home);
        }
        let mut config = config;
        if let Some(cb) = config.ask_callback.take() {
            tools.set_ask_callback_arc(cb);
        }
        Ok(Self {
            client,
            config,
            router,
            stable,
            tail: VolatileTail::new(),
            tools,
        })
    }

    pub fn router_mut(&mut self) -> &mut ModelRouter {
        &mut self.router
    }

    pub fn prefix_epoch_short(&self) -> &str {
        self.stable.epoch.short()
    }

    pub fn transcript_tail(&self) -> &[ChatMessage] {
        &self.tail.messages
    }

    /// Replace volatile transcript (e.g. session load). Applies tool-pair repair (spec 15).
    pub fn load_transcript(
        &mut self,
        messages: Vec<ChatMessage>,
    ) -> (usize, Vec<InterruptedTool>) {
        let (paired, holes) = pair_tool_results(&messages);
        let n = holes.len();
        self.tail.messages = paired;
        (n, holes)
    }

    /// Load session from store; returns number of repaired interrupted tool holes.
    pub fn resume_session(
        &mut self,
        store: &SessionStore,
        id: &str,
    ) -> Result<usize, AgentError> {
        let (messages, holes, _) = store.load(id)?;
        self.tail.messages = messages;
        Ok(holes.len())
    }

    /// Persist current volatile transcript to the session store.
    pub fn persist_session(
        &self,
        store: &SessionStore,
        id: &str,
    ) -> Result<(), AgentError> {
        let ws = self.config.workspace_root.to_string_lossy();
        store.save(id, &self.tail.messages, Some(ws.as_ref()))?;
        Ok(())
    }

    /// Run one user turn (may include internal tool rounds; M1 tools return stub errors).
    pub async fn run_turn<F>(
        &mut self,
        user_input: &str,
        mut on_event: F,
    ) -> Result<TurnOutcome, AgentError>
    where
        F: FnMut(TurnEvent),
    {
        let (user_text, _cmd) = apply_routing_command(&mut self.router, user_input);
        if user_text.trim().is_empty() && _cmd.is_some() {
            // command-only: still need a message for the model; use a short ack prompt
            // Actually for /preset max with no text, just return empty outcome.
            let route = self.router.route_turn_for_preset("");
            return Ok(TurnOutcome {
                assistant_text: String::new(),
                reasoning_text: String::new(),
                model_used: route.wire_model.clone(),
                route,
                tool_rounds: 0,
            });
        }

        let mut route = self.router.route_turn_for_preset(&user_text);
        if let Some(e) = self.config.effort_override {
            route.effort = e;
        }
        if let Some(enabled) = self.config.thinking_enabled {
            route.thinking = if enabled {
                ThinkingMode::enabled()
            } else {
                ThinkingMode::disabled()
            };
        }
        if self.config.show_model {
            on_event(TurnEvent::ModelVisibility(route.visibility_line()));
        }
        on_event(TurnEvent::PrefixEpoch(self.stable.epoch.log_label()));

        self.tail.push_user(user_text);

        let mut tool_rounds = 0u32;
        let mut last_route = route.clone();
        // Initialized before loop; always overwritten by the first successful stream.
        #[allow(unused_assignments)]
        let mut last_content = String::new();
        #[allow(unused_assignments)]
        let mut last_reasoning = String::new();
        let mut model_used = route.wire_model.clone();

        loop {
            // Repair pairing before send
            let (paired, holes) = pair_tool_results(&self.tail.messages);
            if !holes.is_empty() {
                self.tail.messages = paired;
                on_event(TurnEvent::Warning(format!(
                    "repaired {} interrupted tool result(s)",
                    holes.len()
                )));
            }

            let messages = assemble_messages(&self.stable, &self.tail);
            // When tools are in play, reasoning_content must be present on assistant msgs (provider types keep it).
            let _ = tools_in_play(&messages);

            let tools = if self.config.tools.is_empty() {
                None
            } else {
                Some(self.config.tools.clone())
            };

            let request = ChatRequestBuilder::new(last_route.model.clone())
                .messages(messages)
                .stream(true)
                .thinking(Some(last_route.thinking.clone()))
                .reasoning_effort(Some(last_route.effort))
                .tools(tools)
                .build();

            let completed = match self
                .client
                .chat_stream(request, |ev| match ev {
                    StreamEvent::ReasoningDelta(s) => on_event(TurnEvent::ReasoningDelta(s)),
                    StreamEvent::ContentDelta(s) => on_event(TurnEvent::ContentDelta(s)),
                    StreamEvent::Model(m) => {
                        model_used = m;
                    }
                    _ => {}
                })
                .await
            {
                Ok(c) => c,
                Err(ProviderError::ApiStatus { status, body: _ })
                    if status == 404 && last_route.model.as_wire() == MODEL_PRO =>
                {
                    let fb = ModelRouter::fallback_flash("pro unavailable (404)");
                    on_event(TurnEvent::Warning(fb.warning.clone().unwrap_or_default()));
                    on_event(TurnEvent::ModelVisibility(fb.visibility_line()));
                    last_route = fb;
                    continue;
                }
                Err(e) => return Err(e.into()),
            };

            if let Some(ev) = &completed.cache_evidence {
                on_event(TurnEvent::CacheEvidence(ev.log_label().to_string()));
            }

            last_content = completed.message.content.clone();
            last_reasoning = completed.message.reasoning_content.clone();
            if let Some(m) = &completed.model {
                model_used = m.clone();
            }

            // Push assistant with reasoning for tool replay
            let tool_calls = completed.message.tool_calls.clone();
            self.tail.push(ChatMessage::assistant_with_reasoning(
                if last_content.is_empty() {
                    None
                } else {
                    Some(last_content.clone())
                },
                if last_reasoning.is_empty() {
                    None
                } else {
                    Some(last_reasoning.clone())
                },
                if tool_calls.is_empty() {
                    None
                } else {
                    Some(tool_calls.clone())
                },
            ));

            if tool_calls.is_empty() {
                break;
            }

            tool_rounds += 1;
            if tool_rounds > self.config.max_tool_rounds {
                on_event(TurnEvent::Warning(
                    "max tool rounds reached; stopping".into(),
                ));
                break;
            }

            // Spec 50 / G4: concurrent read-only tools; mutating serial after.
            self.handle_tool_calls_batch(&tool_calls, &mut on_event)?;
            // continue loop for model to consume tool results
        }

        Ok(TurnOutcome {
            assistant_text: last_content,
            reasoning_text: last_reasoning,
            route: last_route,
            model_used,
            tool_rounds,
        })
    }

    /// Spec 50: run read-only tools concurrently (capped), then mutating tools serially.
    fn handle_tool_calls_batch<F>(
        &mut self,
        tool_calls: &[ToolCall],
        on_event: &mut F,
    ) -> Result<(), AgentError>
    where
        F: FnMut(TurnEvent),
    {
        if tool_calls.is_empty() {
            return Ok(());
        }
        if tool_calls.len() == 1 {
            return self.handle_tool_call(&tool_calls[0], on_event);
        }

        // Repair arguments first (serial; cheap) so classification uses fixed JSON.
        let mut prepared: Vec<(ToolCall, Result<serde_json::Value, String>)> = Vec::new();
        for call in tool_calls {
            on_event(TurnEvent::ToolCallProposed {
                name: call.function.name.clone(),
                arguments: call.function.arguments.clone(),
            });
            let schema = self
                .config
                .tools
                .iter()
                .find(|t| t.function.name == call.function.name)
                .and_then(|t| t.function.parameters.clone());
            match repair_tool_arguments(&call.function.arguments, schema.as_ref()) {
                Ok(outcome) => {
                    if outcome.repair_applied {
                        on_event(TurnEvent::ToolRepairApplied {
                            name: call.function.name.clone(),
                        });
                    }
                    prepared.push((call.clone(), Ok(outcome.arguments)));
                }
                Err(e) => prepared.push((call.clone(), Err(e.to_string()))),
            }
        }

        let class_input: Vec<(String, serde_json::Value)> = prepared
            .iter()
            .map(|(c, r)| {
                (
                    c.function.name.clone(),
                    r.as_ref()
                        .ok()
                        .cloned()
                        .unwrap_or_else(|| serde_json::json!({})),
                )
            })
            .collect();
        let (ro_idx, mu_idx) = crate::parallel::partition_indices(&class_input);

        // Parallel read-only: isolated executors (separate snippet tables — safe for reads).
        if ro_idx.len() > 1 {
            on_event(TurnEvent::Warning(format!(
                "parallel_readonly n={} (spec 50 / G4)",
                ro_idx.len().min(crate::parallel::MAX_PARALLEL_READONLY)
            )));
            let workspace = self.tools.workspace.clone();
            let policy = self.tools.policy.clone();
            let user_skills = self.tools.user_skills_root.clone();
            let chunk: Vec<_> = ro_idx
                .iter()
                .take(crate::parallel::MAX_PARALLEL_READONLY)
                .copied()
                .collect();
            let mut handles = Vec::new();
            for i in chunk {
                let (call, repaired) = &prepared[i];
                let call = call.clone();
                let repaired = repaired.clone();
                let workspace = workspace.clone();
                let policy = policy.clone();
                let user_skills = user_skills.clone();
                handles.push(std::thread::spawn(move || {
                    let content = match repaired {
                        Err(e) => serde_json::json!({
                            "error": "invalid_tool_arguments",
                            "tool": call.function.name,
                            "message": e
                        })
                        .to_string(),
                        Ok(args) => {
                            let mut ex = ToolExecutor::new(workspace, policy);
                            ex.user_skills_root = user_skills;
                            match ToolName::parse(&call.function.name) {
                                Some(name) => {
                                    let req = ToolRequest {
                                        name,
                                        arguments: args,
                                    };
                                    match ex.execute(&req) {
                                        Ok(r) => r.content,
                                        Err(e) => serde_json::json!({
                                            "error": "tool_failed",
                                            "tool": call.function.name,
                                            "message": e.to_string()
                                        })
                                        .to_string(),
                                    }
                                }
                                None => serde_json::json!({
                                    "error": "unknown_tool",
                                    "tool": call.function.name
                                })
                                .to_string(),
                            }
                        }
                    };
                    (call.id, content)
                }));
            }
            for h in handles {
                match h.join() {
                    Ok((id, content)) => {
                        self.tail.push(ChatMessage::tool_result(id, content));
                    }
                    Err(_) => {
                        on_event(TurnEvent::Warning(
                            "parallel tool worker panicked".into(),
                        ));
                    }
                }
            }
            // Remainder read-only beyond cap: serial
            for i in ro_idx
                .iter()
                .skip(crate::parallel::MAX_PARALLEL_READONLY)
                .copied()
            {
                self.finish_prepared_call(&prepared[i], on_event)?;
            }
        } else {
            for i in ro_idx {
                self.finish_prepared_call(&prepared[i], on_event)?;
            }
        }

        for i in mu_idx {
            self.finish_prepared_call(&prepared[i], on_event)?;
        }
        Ok(())
    }

    fn finish_prepared_call<F>(
        &mut self,
        prepared: &(ToolCall, Result<serde_json::Value, String>),
        on_event: &mut F,
    ) -> Result<(), AgentError>
    where
        F: FnMut(TurnEvent),
    {
        let (call, repaired) = prepared;
        let repaired = match repaired {
            Ok(v) => v.clone(),
            Err(e) => {
                on_event(TurnEvent::ToolError {
                    name: call.function.name.clone(),
                    error: e.clone(),
                });
                let body = serde_json::json!({
                    "error": "invalid_tool_arguments",
                    "tool": call.function.name,
                    "message": e
                });
                self.tail
                    .push(ChatMessage::tool_result(call.id.clone(), body.to_string()));
                return Ok(());
            }
        };
        if let Some(ToolName::Subagent) = ToolName::parse(&call.function.name) {
            return self.run_subagent_tool(call, &repaired, on_event);
        }
        let exec_result = if let Some(name) = ToolName::parse(&call.function.name) {
            let req = ToolRequest {
                name,
                arguments: repaired.clone(),
            };
            self.tools.execute(&req)
        } else if call.function.name.starts_with("mcp__") {
            self.tools.execute_mcp(&call.function.name, &repaired)
        } else {
            let body = serde_json::json!({
                "error": "unknown_tool",
                "tool": call.function.name
            });
            self.tail
                .push(ChatMessage::tool_result(call.id.clone(), body.to_string()));
            return Ok(());
        };
        match exec_result {
            Ok(resp) => {
                self.tail
                    .push(ChatMessage::tool_result(call.id.clone(), resp.content));
            }
            Err(e) => {
                on_event(TurnEvent::ToolError {
                    name: call.function.name.clone(),
                    error: e.to_string(),
                });
                let body = serde_json::json!({
                    "error": "tool_failed",
                    "tool": call.function.name,
                    "message": e.to_string()
                });
                self.tail
                    .push(ChatMessage::tool_result(call.id.clone(), body.to_string()));
            }
        }
        Ok(())
    }

    fn run_subagent_tool<F>(
        &mut self,
        call: &ToolCall,
        args: &serde_json::Value,
        on_event: &mut F,
    ) -> Result<(), AgentError>
    where
        F: FnMut(TurnEvent),
    {
        let kind_s = args
            .get("kind")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let task = args.get("task").and_then(|v| v.as_str()).unwrap_or("");
        let Some(kind) = crate::subagent::WorkerKind::parse(kind_s) else {
            let body = serde_json::json!({
                "error": "unknown_kind",
                "kind": kind_s
            });
            self.tail
                .push(ChatMessage::tool_result(call.id.clone(), body.to_string()));
            return Ok(());
        };
        if task.is_empty() {
            let body = serde_json::json!({"error": "missing task"});
            self.tail
                .push(ChatMessage::tool_result(call.id.clone(), body.to_string()));
            return Ok(());
        }
        on_event(TurnEvent::Warning(format!(
            "subagent kind={} (spec 60 / G5)",
            kind.as_str()
        )));
        match crate::subagent::run_worker(
            kind,
            &self.config.workspace_root,
            task,
            &self.tools.policy,
            self.tools.user_skills_root.as_deref(),
        ) {
            Ok(out) => {
                crate::subagent::parent_after_worker(&mut self.tools, &out);
                let body = serde_json::json!({
                    "ok": true,
                    "kind": out.kind.as_str(),
                    "summary": out.summary,
                    "tool_rounds": out.tool_rounds,
                    "mutated": out.mutated,
                    "prefix_epoch": out.prefix_epoch_short,
                });
                self.tail
                    .push(ChatMessage::tool_result(call.id.clone(), body.to_string()));
            }
            Err(e) => {
                on_event(TurnEvent::ToolError {
                    name: call.function.name.clone(),
                    error: e.to_string(),
                });
                let body = serde_json::json!({
                    "error": "subagent_failed",
                    "message": e.to_string()
                });
                self.tail
                    .push(ChatMessage::tool_result(call.id.clone(), body.to_string()));
            }
        }
        Ok(())
    }

    fn handle_tool_call<F>(&mut self, call: &ToolCall, on_event: &mut F) -> Result<(), AgentError>
    where
        F: FnMut(TurnEvent),
    {
        on_event(TurnEvent::ToolCallProposed {
            name: call.function.name.clone(),
            arguments: call.function.arguments.clone(),
        });

        let schema = self
            .config
            .tools
            .iter()
            .find(|t| t.function.name == call.function.name)
            .and_then(|t| t.function.parameters.clone());

        let repaired = match repair_tool_arguments(&call.function.arguments, schema.as_ref()) {
            Ok(outcome) => {
                if outcome.repair_applied {
                    on_event(TurnEvent::ToolRepairApplied {
                        name: call.function.name.clone(),
                    });
                }
                outcome.arguments
            }
            Err(e) => {
                on_event(TurnEvent::ToolError {
                    name: call.function.name.clone(),
                    error: e.to_string(),
                });
                let body = serde_json::json!({
                    "error": "invalid_tool_arguments",
                    "tool": call.function.name,
                    "message": e.to_string()
                });
                self.tail
                    .push(ChatMessage::tool_result(call.id.clone(), body.to_string()));
                return Ok(());
            }
        };

        self.finish_prepared_call(&(call.clone(), Ok(repaired)), on_event)
    }

    /// Dual-call cache evidence helper (ADR 0005 substitute).
    pub async fn cache_evidence_dual_call(&self) -> Result<String, AgentError> {
        let messages = assemble_messages(&self.stable, &VolatileTail::new());
        let req = ChatRequestBuilder::new(ModelId::Flash)
            .messages(messages)
            .stream(false)
            .build();
        let ev = self.client.cache_evidence_dual_call(req).await?;
        Ok(ev.log_label().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dsb_provider_deepseek::ClientConfig;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn multi_turn_keeps_stable_prefix_epoch() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("content-type", "text/event-stream")
                    .set_body_string(concat!(
                        "data: {\"model\":\"deepseek-v4-flash\",\"choices\":[{\"delta\":{\"content\":\"ok1\"}}]}\n\n",
                        "data: [DONE]\n\n",
                    )),
            )
            .mount(&server)
            .await;

        let client = Arc::new(
            Client::new(ClientConfig::new("k").with_base_url(server.uri())).unwrap(),
        );
        let mut agent = Agent::new(
            client,
            AgentConfig {
                workspace_root: std::env::temp_dir(),
                show_model: true,
                ..AgentConfig::default()
            },
        )
        .unwrap();
        let epoch1 = agent.prefix_epoch_short().to_string();
        let mut models = Vec::new();
        agent
            .run_turn("hi", |ev| {
                if let TurnEvent::ModelVisibility(m) = ev {
                    models.push(m);
                }
            })
            .await
            .unwrap();
        let epoch2 = agent.prefix_epoch_short().to_string();
        assert_eq!(epoch1, epoch2);
        assert!(models.iter().any(|m| m.contains("deepseek-v4-flash")));
    }

    #[tokio::test]
    async fn pro_once_visible() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("content-type", "text/event-stream")
                    .set_body_string(concat!(
                        "data: {\"model\":\"deepseek-v4-pro\",\"choices\":[{\"delta\":{\"content\":\"pro-ans\"}}]}\n\n",
                        "data: [DONE]\n\n",
                    )),
            )
            .mount(&server)
            .await;

        let client = Arc::new(
            Client::new(ClientConfig::new("k").with_base_url(server.uri())).unwrap(),
        );
        let mut agent = Agent::new(
            client,
            AgentConfig {
                workspace_root: std::env::temp_dir(),
                ..AgentConfig::default()
            },
        )
        .unwrap();
        agent.router_mut().set_auto_router(false);
        let mut vis = Vec::new();
        let out = agent
            .run_turn("/pro hard problem", |ev| {
                if let TurnEvent::ModelVisibility(m) = ev {
                    vis.push(m);
                }
            })
            .await
            .unwrap();
        assert!(vis.iter().any(|v| v.contains(MODEL_PRO)));
        assert_eq!(out.route.wire_model, MODEL_PRO);
        // next turn flash
        let out2 = agent.run_turn("follow up", |_| {}).await.unwrap();
        assert_eq!(out2.route.wire_model, "deepseek-v4-flash");
    }
}
