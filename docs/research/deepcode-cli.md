# Research: Deep Code CLI

**Upstream:** https://github.com/lessweb/deepcode-cli  
**Package:** `@vegamo/deepcode-cli`  
**DeepSeek docs:** Integrate with Deep Code (official agent integrations list)

## Why it matters

Officially listed DeepSeek-oriented terminal agent. Tuned for V4: thinking, effort, Skills, MCP, permissions, context caching. Philosophy: harness should match DeepSeek tool habits (“better models, worse tools” framing in their docs).

## Surface to mirror (behavior, not necessarily code)

- `thinkingEnabled` / `reasoningEffort`
- Skills paths: `.deepcode/skills`, `.agents/skills`, user-level counterparts
- `/model`, `/plan`, `/new`, `/resume`, `/fork`, `/skills`, `/mcp`, `/undo`
- MCP servers + fine-grained permissions
- `notify` script hook
- Shared settings idea (CLI ↔ editor) — editor later

## Takeaways

- Do not invent a random settings schema if Deep Code’s knobs already map to the API
- Skills interoperability via `.agents/skills` is valuable
- Light `/plan` is in; heavy multi-day planning harness is not

## Local clone

Not yet required in sibling tree; clone into `OpenSources/deepcode-cli` when implementing specs 30/70/80/90.
