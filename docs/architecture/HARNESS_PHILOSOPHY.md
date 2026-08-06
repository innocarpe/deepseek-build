# Harness philosophy

**Status:** Normative product architecture (pre-implementation)  
**Audience:** Anyone writing specs, ADRs, or runtime code for DeepSeek Build  
**Last updated:** 2026-08-06

This document is the **design spine**. Feature specs (`docs/specs/`) refine it; they must not contradict it without an ADR that supersedes a named section here.

---

## 1. One sentence

DeepSeek Build is a **DeepSeek-native coding harness**: tool shapes, context layout, recovery, and safety are tuned for how DeepSeek V4 actually behaves; Grok-class parallelism is layered on top **without** abandoning that native fit.

It is **not** “Grok Build with `base_url=api.deepseek.com`.”

---

## 2. Why a native harness (Deep Code thesis)

Coding agent quality is **model × harness**, not model alone.

Tool schemas are not neutral. Models carry **usage habits** from training and RL (see Armin Ronacher, [*Better Models: Worse Tools*](https://lucumr.pocoo.org/2026/7/4/better-models-worse-tools/)). A stronger model can be *more* brittle on unfamiliar tool shapes.

Implications we accept:

| Implication | Design consequence |
|-------------|-------------------|
| Generic “OpenAI-tools” agents underperform on DeepSeek | Prefer DeepSeek-proven contracts over cargo-cult Claude/Grok tool UX |
| Recovery paths matter more than perfect first-shot tool JSON | Snippet edits, tool-call repair, strict validation + tolerant recovery |
| Cost is structural | Cache-stable prefixes are an invariant, not a late optimization |
| Skills and permissions shape model behavior | They are quality mechanisms, not bolt-ons |

**Product goal (adapted from Deep Code):** on real coding tasks, DeepSeek Build + DeepSeek V4 should beat “generic strong CLI + DeepSeek API” on **result quality per dollar**, while matching or beating Grok Build on **wall-clock progress** for multi-step work.

Reference: [Deep Code architecture (EN)](https://github.com/lessweb/deepcode-cli/blob/main/docs/architecture_en.md).

---

## 3. Three layers (who wins when)

Sources are **not** a single ranked list for every decision. They own different layers:

```text
┌─────────────────────────────────────────────────────────┐
│ L1  DeepSeek-native contracts (Deep Code + Reasonix)  │
│     tools shape · edit · cache prefix · skills · perms  │
├─────────────────────────────────────────────────────────┤
│ L2  Cost & session economics (Reasonix primary)         │
│     Flash/Pro · effort · prefix stability · repair      │
├─────────────────────────────────────────────────────────┤
│ L3  Execution throughput (Grok Build primary)           │
│     parallel tools · bg shell · subagents · worktrees   │
└─────────────────────────────────────────────────────────┘
```

### Conflict resolution (fail-close)

| Conflict | Winner | Example |
|----------|--------|---------|
| Edit/tool schema vs “faster Grok-like edit” | **L1 Deep Code** | Snippet-scoped edit over free-form whole-file guess |
| Cache-stable prefix vs “inject everything every turn” | **L1/L2** | Dynamic tree walks stay on turn **tail** |
| Subagent fan-out vs cold uncached prefixes | **L2 constraints on L3** | Workers share stable template; Flash-default workers |
| Parallel speed vs permission honesty | **L1 permissions** | Side-effect declaration still required |
| UX knobs (thinking/effort/skills paths) | **L1 Deep Code surface** | Match official DeepSeek-oriented CLI habits |
| Orchestration topology | **L3 Grok** | Subagents, bg tasks, multi-wait |

**Hard rule:** L3 may never ship a feature that **knowingly** violates L1/L2 invariants “for speed.” Speed without cache/tool fit is a false north star for this product.

---

## 4. Deep Code four pillars (adopted)

These are first-class design pillars for DeepSeek Build. Spec IDs are where they become testable.

### 4.1 Pillar A — Repair tool use through **snippets** (edit contract)

**Problem:** Path + large `old_string`/`new_string` fails in predictable ways (stale view, wrong repeated block, indentation, over-replace, JSON escapes).

**Contract (normative intent):**

1. Text files are read into **session-local state**.  
2. `read` returns content **and** a `snippet_id` (path, range, version, scope, preview).  
3. `edit` **requires** a valid `snippet_id` from the current session.  
4. File must not have changed under the agent since the snippet was issued (version check).  
5. Replacement is searched **only within snippet scope**.  
6. Non-unique matches → return candidate snippets; **do not guess**.  
7. Bulk replace may require expected occurrence count.

**Tool surface bias (Deep Code-aligned, small core):**

`bash`, `read`, `write`, `edit`, `ask_user` (or equivalent), light plan update, `web_search` — plus dynamically mounted MCP. Prefer a **small, predictable action language** over an exploding tool zoo.

**Bypass law (normative):**

| Tool | Rule |
|------|------|
| `edit` | **Must** use valid `snippet_id` (spec 45). |
| `write` | **Create-new only** by default, or overwrite only with explicit flag **and** same version/safety policy as edit for existing paths (spec 45/40). Must not become “edit without snippet.” |
| `bash` | File mutation via shell is a **high side-effect** class (spec 90). Default policy: **ask** (or deny) for write/delete outside an allowlist; never a silent full bypass of snippet safety for routine edits. |

**Spec:** `45-snippet-edit` (and parts of `40-tools`, `90-permissions`).

**Grok note:** Hashline/anchor edits may inform implementation **only if** they satisfy the snippet/version/scope semantics above. Do not replace the contract with a different shape “because Grok has anchors.”

### 4.2 Pillar B — **Cache-aware** context layout

DeepSeek context/prefix cache rewards **stable repeated prefixes** (best-effort; still design for it).

**Normative layout (stable → volatile):**

```text
[stable prefix]
  system prompt
  tool schemas / docs (canonical, ordered)
  default skills *index* (not full bodies)
  runtime/environment summary (small, stable)
  project standing instructions (AGENTS.md / DEEPSEEK.md / …)
[volatile tail]
  user turn
  dynamic reminders
  large tool outputs (snip/prune before summary)
  ephemeral paths / timestamps
```

**Additional session rules (Deep Code + Reasonix):**

- Persist session as **replayable** message log (JSONL or equivalent).  
- Repair **tool-call / tool-result pairing** on replay (including interrupted tools).  
- Never rewrite stable tool schemas mid-session without starting a **new** cache epoch (explicit, rare).

**Spec:** `10-cache-contract` (shared Reasonix + Deep Code).

**Reasonix emphasis:** prefix **byte-stability** across turns is an **invariant**, not a nice-to-have. Mid-session mutation of the stable system/tool/memory prefix is a **bug**.

**Ownership when Deep Code session layout and Reasonix byte-stability conflict:**  
Reasonix **wins on what must stay byte-identical** across turns. Deep Code **wins on which logical sections** belong in the prefix vs tail and on tool/result pairing repair. Spec 10 must encode both; if they clash, **byte-stability of the declared stable sections wins**, and the section boundaries adjust via ADR.

**What “byte-stable” means (minimum for spec 10 — not optional):**

- Canonical serialization (field order, JSON key order, newlines, Unicode normalization)  
- Epoch / invalidation when tools schema or skills index changes  
- Golden fixtures: two consecutive builds with identical inputs → identical prefix bytes  
- Provider cache hit/miss telemetry when the API reports it; “intent to reuse” is not acceptance  

Philosophy names the invariant; **spec 10 makes it executable**.

### 4.3 Pillar C — Skills as **structured on-demand context**

Skills are **not** traditional plugins. They are **structured context** injected when useful.

**Rules:**

1. Default context stays **lean** (do not dump all skill bodies into every turn).  
2. Discovery paths interoperability: project/user `.agents/skills`, product-local skills dirs (Deep Code-compatible).  
3. **Index** (name + description) may live in the stable prefix; **bodies** load on demand.  
4. Implicit match may use the model over candidates; already-loaded skills are not reloaded.  
5. Skills may opt out of implicit invocation.  
6. Loading a skill body must not thrash the stable prefix (bodies → tail or dedicated non-prefix slot per spec 10/70).

**Spec:** `70-skills`.

### 4.4 Pillar D — Permissions via **side-effect classification**

Permissions are **agent quality**, not only compliance.

**Rules:**

1. Concrete scopes: read/write/delete in/out of workspace; git query/mutate; network; MCP; etc.  
2. File tools classify by **path**.  
3. `bash` (and similar) requires the model to **declare side effects** of the operation; policy decides allow / ask / deny.  
4. Low-risk work stays fast; high-risk stops for confirmation.  
5. Auditable: command = text + declared effects + decision.

**Spec:** `90-permissions`.

**Anti-pattern:** YOLO-only mode as the sole product mode.

---

## 5. Reasonix pillars (adopted)

Reasonix is the cost/cache culture twin of Deep Code’s session design.

| Pillar | Normative intent | Spec |
|--------|------------------|------|
| **Cache-first loop** | Stable system+tools+memory prefix; ride the tail for dynamics | `10` |
| **Flash-first economics** | Default Flash; Pro on escalate (`/pro`, presets, router) | `20` |
| **Tool-call repair** | Schema-aware repair of malformed tool JSON before dispatch | `15-tool-call-repair` |
| **Thinking/effort honesty** | Surface DeepSeek thinking + effort knobs; don’t hide max effort always | `30` |
| **Long-session affordability** | User-visible cache/cost signals when telemetry available | `10`, UX |

Reasonix desktop/plugin surface is **out of scope** for v1 CLI; the **loop economics** are not.

---

## 6. Grok Build pillars (adopted — L3 only)

| Pillar | Normative intent | Spec |
|--------|------------------|------|
| Parallel independent tool calls | One turn, many tools when independent | `50` |
| Background shell + multi-wait | Long commands don’t block the conversation | `50` |
| Subagents with own context | Explore/implement/review fan-out | `60` |
| Optional worktree isolation | Write workers don’t collide | `60` |
| Native-speed local tools | Prefer low-overhead tool runtime | `40` |

**Subagent cache law (L2 constrains L3):**

- Worker system/tool **templates** must be **shared and stable** where possible.  
- Prefer **Flash** workers; Pro for review/architecture only.  
- Parent receives **summaries**, not full cold transcripts.  
- Spawning N workers that each re-pay a unique 20KB uncached system prompt is a **design failure**.

---

## 7. Model routing philosophy

| Mode | Model | Effort | Use |
|------|-------|--------|-----|
| Default loop | V4-Flash | medium/low | explore, edit, tool churn |
| Escalation | V4-Pro | high/max | hard design, tough bugs, final review |
| Presets | flash / balanced / max | maps effort+model | session-level UX (Deep Code `/model`, Reasonix presets) |

Router heuristics live in `20-model-routing`. Product must never silently run **max effort Pro on every turn** as the only path.

---

## 8. Plan mode philosophy

- **In:** light `/plan` checklist that **unblocks execution** (Deep Code).  
- **Out:** multi-stage interview → ralplan → ultragoal style harnesses that stall wall-clock progress (Gajae-class; see NON_GOALS).

Plan is a **tool for the model**, not a project-management religion.

**Spec:** `110-plan-mode`.

---

## 9. Non-goals of this philosophy (v1)

- Multi-vendor “works equally on Claude/GPT/DeepSeek” as identity  
- Cloning Grok monorepo wholesale  
- Cloning Reasonix desktop  
- Cloning Deep Code Node stack  
- Process-police CI as a substitute for harness design  

---

## 10. Spec map (philosophy → contracts)

| Spec ID | Title | Primary philosophy owner |
|---------|-------|---------------------------|
| 10 | Cache contract | Reasonix + Deep Code B |
| 15 | Tool-call repair | Reasonix |
| 20 | Model routing Flash/Pro | Reasonix + Deep Code UX |
| 30 | Thinking & effort | Deep Code + API |
| 40 | Core tools surface | Deep Code (small set) + Grok speed |
| 45 | Snippet edit contract | Deep Code A |
| 50 | Parallelism & background | Grok L3 |
| 60 | Subagents (+ cache law) | Grok L3 under L2 |
| 70 | Skills as structured context | Deep Code C |
| 80 | MCP | Deep Code surface |
| 90 | Side-effect permissions | Deep Code D |
| 100 | Sessions new/resume/fork | Deep Code surface |
| 110 | Light plan mode | Deep Code |
| 120 | Project config surface | All |

---

## 11. Implementation gates (hard)

These gates exist so L3 work cannot ship on prose-only L1/L2.

| Gate | Required before |
|------|-----------------|
| **G0** HARNESS_PHILOSOPHY + SOURCES layered model merged | Any runtime PR |
| **G1** Toolchain/config ADR (language, binary name, state dir, secrets) | Any `crates/` / package scaffolding |
| **G2** Specs **10, 15, 20, 30** status = ready-for-impl | M1 provider loop code |
| **G3** Specs **45 + minimum 90** (or shell denied until 90) ready | M2 mutating tools / shell |
| **G4** Spec **50** ready | Parallel tool dispatch |
| **G5** Spec **60** ready (worker cache law measurable) | Subagent fan-out |
| **G6** Specs **70, 80, 100, 110** ready | Skills/MCP/sessions/plan product surface |

**Definition of ready-for-impl:** acceptance criteria + failure modes + non-goals + test plan (golden or manual) + philosophy section citations. Index row must not say `TODO`.

**Launch definition (PRD):** “M1 implementation started” is **invalid** until G1+G2 pass. Starting code with empty specs is a process bug.

---

## 12. Review checklist for future changes

Any PR that touches tools, prompts, sessions, skills, or permissions must answer:

1. Which layer (L1/L2/L3) does this change?  
2. Does it weaken snippet/edit safety, cache stability, skill leanness, or side-effect honesty?  
3. Cache-impact: none / low / medium / high — why?  
4. If it copies Grok behavior, does it still satisfy Deep Code pillars A–D?  
5. Which **gate** (G0–G6) does this PR assume is already green?

If the answer to (2) is yes without an ADR, **reject**.

---

## 13. References

- Deep Code architecture (EN): https://github.com/lessweb/deepcode-cli/blob/main/docs/architecture_en.md  
- Reasonix project (local): `OpenSources/DeepSeek-Reasonix`  
- Grok Build (local): `OpenSources/grok-build`  
- Product: [VISION](../product/VISION.md), [SOURCES](../product/SOURCES.md), [PRD-v1](../product/PRD-v1.md)  
