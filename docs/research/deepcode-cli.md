# Research: Deep Code CLI

**Upstream:** https://github.com/lessweb/deepcode-cli  
**Architecture (EN):** https://github.com/lessweb/deepcode-cli/blob/main/docs/architecture_en.md  
**Package:** `@vegamo/deepcode-cli`  
**DeepSeek docs:** agent integration “Deep Code”

**Binding product extraction:** [HARNESS_PHILOSOPHY.md](../architecture/HARNESS_PHILOSOPHY.md) §4  
This file is **research evidence**, not a substitute for the philosophy doc.

---

## Why it matters

Officially listed DeepSeek-oriented terminal agent. Design goal: better results than “Claude Code + DeepSeek” at lower cost by **adapting the harness to DeepSeek**, not by being a generic multi-model framework.

Core insight (via Ronacher): **tool schemas are not neutral** — models carry tool-use habits.

## Four core designs (architecture_en.md)

### 1. Snippet-based edit repair

- `read` maintains session-local file state and returns `snippet_id`.  
- `edit` requires `snippet_id`; scoped replace; version check; non-unique → candidates.  
- Tolerates recoverable model mistakes while staying strict on validation.  
- Small built-in tool set: bash, read, write, edit, AskUserQuestion, UpdatePlan, WebSearch + MCP.

### 2. Cache-aware context management

- Stable content before volatile user content (system, tools, skills index, project instructions).  
- Session JSONL persistence and consistent replay.  
- Tool-call/result pairing repair (including interrupted tools).

### 3. Agent Skills as structured context

- Do not stuff all knowledge into every turn.  
- On-demand load; model-assisted matching over candidates.  
- Skills are structured context, not classic plugins.

### 4. Side-effect permission classification

- Scopes for filesystem, git, network, MCP, etc.  
- Bash declares side effects; file tools classify by path.  
- Safety **and** agent quality (predictable boundaries).

## Surface to mirror (UX)

- `thinkingEnabled` / `reasoningEffort`  
- Skills paths: `.deepcode/skills`, `.agents/skills`, user-level  
- `/model`, `/plan`, `/new`, `/resume`, `/fork`, `/skills`, `/mcp`, `/undo`  
- MCP + fine-grained permissions  
- `notify` hook  
- Shared settings idea (CLI ↔ editor) — editor later  

## What we deliberately do not copy day one

- Node/TS stack  
- VS Code extension parity  
- Every slash name byte-identical  

## Promote to product

Already in HARNESS_PHILOSOPHY + specs index: 10, 30, 40, 45, 70, 80, 90, 100, 110.
