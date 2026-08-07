# Heart 3.x — Spec binding under the Grok agent path

**Wave unit:** `3x-H0-1` · **Story:** G003 SpecMap · **Plan:** `heart-3x`  
**DoD owner:** [PRD-v3.md](../product/PRD-v3.md) §3 P0  
**PR units:** [WAVE_3x_PR_DAG.md](../product/WAVE_3x_PR_DAG.md)  
**Test plan (H0-2):** [HEART_3X_P0_TEST_PLAN.md](../product/HEART_3X_P0_TEST_PLAN.md)

**Purpose:** Tell implementers **which file/crate owns** Spec 45 / 90 / 10 / 15 / 20 on the
**default product agent path** (`dsb` → `deepseek-build-agent`), what already exists on the
**thin 1.x path**, and whether 3.0.0 **adapts** Grok surfaces or **ports** dsb crates into them.

This is **not** a greenfield rewrite of Grok. Conflict rule: L3 never ships by violating L1/L2
([HARNESS_PHILOSOPHY.md](./HARNESS_PHILOSOPHY.md)).

---

## 1. Two runtime paths (fail-close honesty)

| Path | Entry | Composition | Heart status as of 2.x shell |
|------|-------|-------------|------------------------------|
| **A. Product agent (default)** | `dsb` / `deepseek-build` bare | `crates/dsb-cli` → `agent_launch` → **`deepseek-build-agent`** (vendored Grok pager) | DeepSeek **paint + routing seed**; L1/L2 hearts **not** controlling tools/context |
| **B. Thin / legacy** | `dsb run`, `dsb chat`, `repl-legacy`, `dsb-tools` MVP | `dsb-agent` + `dsb-tools` + `dsb-context` + `dsb-provider-deepseek` | Spec 45/90/10/15/20 **implemented** with automated tests — **not** the default TUI path |

**3.0.0 P0 is true only when Path A enforces the hearts.** Green tests on Path B alone do **not** cut `v3.0.0`.

```text
User types: dsb
  → dsb-cli (product entry, config seed, GROK_HOME bridge)
  → deepseek-build-agent  [third_party/grok-build xai-grok-pager-bin composition]
       → xai-grok-shell agent loop
       → xai-grok-tools registry / implementations
       → xai-grok-workspace capability modes
       → sampler (must use api.deepseek.com + chat_completions)
```

Thin path (non-default):

```text
dsb run | chat | tools
  → dsb-agent loop_ / repair / routing
  → dsb-tools (snippets + permissions)
  → dsb-context (stable prefix + epoch)
  → dsb-provider-deepseek
```

---

## 2. Ownership matrix (adapt vs reimplement)

Legend:

| Strategy | Meaning |
|----------|---------|
| **Adapt** | Keep Grok crate as primary; enforce Spec *spirit* at the Grok boundary (wrapper, gate, schema, session state). Prefer this. |
| **Port** | Call or embed `dsb-*` logic from the agent path without re-owning a second edit engine. |
| **Thin-only today** | Contract exists only under Path B; 3.x must **bind** it to Path A. |
| **Do not** | Greenfield second agent; free-form whole-file primary; YOLO product default. |

| Spec | Normative doc | Thin owner (Path B) | Grok surface (Path A) | 3.0.0 strategy |
|------|---------------|---------------------|------------------------|----------------|
| **45** Snippet edit | [45-snippet-edit.md](../specs/45-snippet-edit.md) | `crates/dsb-tools` `snippets.rs`, `tools.rs` (`read`→`snippet_id`, `edit` requires id) | Default edit: `xai-grok-tools` **SearchReplace** (`implementations/grok_build/search_replace`, also opencode/hashline variants); wire name often `search_replace` / hashline aliases | **Adapt** SearchReplace (or product-default toolset) so primary path is **snippet-safe** (session snippet or equivalent version+scope); fail closed without valid scope; **do not** make free-form whole-file the default. Optional **port** of `SnippetStore` into tool pre/post hooks. |
| **90** Permissions | [90-permissions.md](../specs/90-permissions.md) | `crates/dsb-tools` `permissions.rs`, `grants.rs`; headless Ask→Deny | `xai-grok-workspace::capability::CapabilityMode` (ReadOnly / ReadWrite / Execute / All); reverse-request permission hooks (`xai-computer-hub-sdk` / shell `pending_interaction`); session phase `PermissionPrompt` | **Adapt** Grok capability + reverse-requests to Spec 90 matrix (allow/deny/ask, bash classify spirit, **headless fail-closed**). Product default must **not** be YOLO-only. Port policy tables from `dsb-tools` where cheaper than re-encoding. |
| **10** Cache / prefix | [10-cache-contract.md](../specs/10-cache-contract.md) | `crates/dsb-context` `prefix.rs`, `epoch.rs`, `canonicalize.rs` | Grok context assembly: shell session + `xai-grok-compaction` + chat-state / prompts under `xai-grok-shell` / pager | **Adapt** agent message assembly so a **stable prefix** + **volatile tail** exist under the real loop; epoch/hash tests against **agent** assembly (not only thin `dsb-context`). Port canonicalize/epoch helpers if useful. |
| **15** Tool-call repair | [15-tool-call-repair.md](../specs/15-tool-call-repair.md) | `crates/dsb-agent` `repair.rs` (1 auto-repair then error) | Tool dispatch in shell/tools registry; arg parse before execute | **Port or re-bind** repair before Grok tool execute on default DeepSeek turns; max 1 repair; never invent required args / rename tool. |
| **20** Flash / Pro | [20-model-routing.md](../specs/20-model-routing.md) | `crates/dsb-agent` `routing.rs`; models in `dsb-provider-deepseek` | Agent models from `~/.deepseek-build` config (`[model.deepseek-v4-flash]`, `[model.deepseek-v4-pro]`); sampler `api_backend = chat_completions`; **per-model `base_url = https://api.deepseek.com`** (load-bearing — see `dsb-cli` `agent_launch`) | **Adapt** config + UI/commands so Flash is default, Pro escalate is dogfoodable, turn visibility of wire model; keep seed/repair of `base_url` (G001). |

---

## 3. Critical Path A files (implementer index)

Paths relative to repo root. Vendor tree: `third_party/grok-build/`.

### 3.1 Product entry / config (already 2.x)

| Concern | Location |
|---------|----------|
| Dual CLI entry | `crates/dsb-cli/src/main.rs`, `agent_launch.rs` |
| Agent bin name | `deepseek-build-agent` (`AGENT_BIN_NAME`) |
| Model seed + `base_url` repair | `crates/dsb-cli/src/agent_launch.rs` (`DEEPSEEK_API_BASE_URL`) |
| Install / build agent | `scripts/build-grok-pager.sh`, `scripts/install.sh` |
| Hermetic GROK_HOME for tests | `scripts/lib/common.sh` |

### 3.2 L1 edit (Spec 45) — bind here in G004

| Concern | Location |
|---------|----------|
| Default SearchReplace impl | `third_party/grok-build/crates/codegen/xai-grok-tools/src/implementations/grok_build/search_replace/` |
| OpenCode-style edit (shares helpers) | `…/implementations/opencode/edit/` |
| Hashline toolset | `…/implementations/grok_build_hashline/` |
| Tool registry / taxonomy | `…/xai-grok-tools/src/registry/`, `tool_taxonomy.rs` |
| Thin reference impl + tests | `crates/dsb-tools/src/snippets.rs`, `tools.rs` |

**Adapt rule:** Grok hashline/anchor is allowed **only if** it satisfies Spec 45 version/scope/uniqueness semantics ([HARNESS_PHILOSOPHY.md](./HARNESS_PHILOSOPHY.md) §4.1 Grok note).

### 3.3 L1 permissions (Spec 90) — bind here in G005

| Concern | Location |
|---------|----------|
| Capability filter | `third_party/grok-build/crates/codegen/xai-grok-workspace/src/capability.rs` |
| Permission reverse-request hooks | `xai-computer-hub-sdk` harness permission_request; shell `pending_interaction` |
| Session permission phase | `xai-tool-protocol` `SessionPhase::PermissionPrompt` |
| Thin reference policy | `crates/dsb-tools/src/permissions.rs`, `grants.rs` |

### 3.4 L2 prefix (Spec 10) — bind here in G006

| Concern | Location |
|---------|----------|
| Grok compaction / context | `third_party/grok-build/crates/common/xai-grok-compaction/` |
| Shell session / compaction pipeline | `xai-grok-shell/src/session/` (incl. `compaction.rs`) |
| Thin reference prefix/epoch | `crates/dsb-context/src/{prefix,epoch,canonicalize}.rs` |

### 3.5 L2 repair + routing (Spec 15 / 20) — bind here in G007

| Concern | Location |
|---------|----------|
| Thin repair | `crates/dsb-agent/src/repair.rs` |
| Thin routing | `crates/dsb-agent/src/routing.rs` |
| Provider wire ids | `crates/dsb-provider-deepseek/` (`deepseek-v4-flash`, `deepseek-v4-pro`) |
| Agent model tables / sampler | Grok config models + shell agent sampling (`base_url`, `chat_completions`) |

---

## 4. What is already green (do not re-prove as 3.0.0)

| Layer | Evidence | Limit |
|-------|----------|-------|
| Spec docs G2/G3 | [GATES.md](../GATES.md) specs 10/15/20/45/90 ready-for-impl | Spec ≠ agent enforcement |
| Thin L1/L2 | `cargo test -p dsb-tools` / `dsb-context` / `dsb-agent`; [W3_L1_L2_MATRIX.md](../product/evidence/W3_L1_L2_MATRIX.md) | Path B only |
| 2.x shell | PRD-v2; agent entry + DeepSeek UI/npm | Hearts residual on Path A |
| base_url + pre-3x harness | G001 / PRE_3X_TEST_MATRIX; T4.0 green when key present | Precondition, not heart fusion |

---

## 5. Explicit non-bindings (out of 3.0.0)

| Topic | Where it goes |
|-------|----------------|
| L3 worktree/subagent fleet as product identity | PRD-v4 / later plan |
| Multi-vendor identity | never product core |
| Gajae multi-stage planning as core loop | non-goal |
| Everyday vendor-full cargo test | optional; disk bomb |
| Claiming 3.0.0 from npm chrome alone | forbidden |

---

## 6. Story → code map

| Story | WAVE | Primary bind targets |
|-------|------|----------------------|
| G004 L1-Snippet | 3x-H1-1 | Grok SearchReplace (+ session snippet state); tests fail closed on free-form primary |
| G005 L1-Permissions | 3x-H1-2,3 | CapabilityMode + reverse-request + headless matrix; dogfood evidence |
| G006 L2-Prefix | 3x-H2-1 | Agent context assembly epoch/hash (Grok stack) |
| G007 L2-RepairRoute | 3x-H2-2,3 | Repair before tool exec; Flash default / Pro escalate under agent |
| G008 Cut-3.0.0 | 3x-H3-* | Honesty docs + evidence + tag **`v3.0.0`** only |

---

## 7. H0 exit criteria

Implementers can answer without spelunking:

1. **Which path is product default?** Path A (`deepseek-build-agent`).  
2. **Where does edit run today on that path?** Grok `search_replace` / hashline — **not** `dsb-tools` edit.  
3. **Where is the reference Spec 45/90 implementation?** `crates/dsb-tools` (thin).  
4. **Adapt or rewrite Grok?** **Adapt** (or port thin helpers into Grok boundaries).  
5. **What red→green cases prove 3.0.0?** [HEART_3X_P0_TEST_PLAN.md](../product/HEART_3X_P0_TEST_PLAN.md).

When this file and the test plan are on `main`, G003 SpecMap is complete.
