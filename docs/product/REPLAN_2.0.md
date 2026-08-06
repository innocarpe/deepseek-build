# Replan — DeepSeek Build **2.0.0** (Grok Build base)

**Status:** Normative product replan (2026-08-06)  
**Supersedes for product direction:** prior overnight interpretation of Waves A–D as “done 1.0.0 product”  
**Does not delete:** 1.x code, specs, tests, or published packages (historical scaffold)

---

## 0. Why this document exists

### Owner intent (restated, fail-close)

1. **Product category** = Claude Code / Codex CLI / **Grok Build** 급  
   → 터미널 **코딩 에이전트** (업계에서 “CLI”라고 부르지만, Grok은 **full-screen TUI + agent runtime**).
2. **Base** = 오픈소스 **Grok Build** (`grok` / `xai-grok-pager` 트리).  
   가장 빠르고 좋았고, 오픈소스라 DeepSeek 용으로 파생 가능하다고 판단.
3. **Overlay** = Deep Code (L1 계약) + Reasonix (L2 캐시/비용) 특장점.
4. **`dsb` / `deepseek-build` 를 치면** 그 에이전트가 **바로** 떠야 한다.  
   그게 **원래 1.0.0 의미**였다.

### What actually happened (1.x)

| Claimed | Reality |
|---------|---------|
| “1.0.0 shipped” | Greenfield `dsb-*` scaffold + docs/gates train |
| “Grok-class” | MVP parallel/bg/subagent heuristics, **not** Grok runtime |
| “dsb opens agent” | Subcommand clap → thin REPL; **not** Grok TUI |
| Onboarding | Late (1.1.0); not the core product |

**Judgment:** 1.x is a **useful research / contract scaffold**. It is **not** the product the owner ordered.

---

## 1. Versioning reset (already published — no history rewrite)

npm / git already have **`1.0.0`** and **`1.1.0`**. We **do not** unpublish or force-delete.

| Line | Meaning (from now) |
|------|---------------------|
| **`1.0.0` – `1.x.y`** | **Legacy scaffold line** — clap agent, contracts, tests, npm package that works as a thin agent. **Not** Grok-base product. |
| **`2.0.0`** | **First real product** matching owner intent: Grok Build–class agent UX, Grok tree as base, DeepSeek native. |
| **`2.0.0-alpha.*` / `2.0.0-beta.*`** | Previews while integrating Grok base (optional). |

### SemVer honesty

- **2.0.0 is a major** because the **user-facing product identity changes** (entry experience, runtime base), not because we “finished more checklists.”
- 1.x remains installable for experiments; README must say **scaffold / not the Grok-base product**.
- Tags: `v2.0.0` only when §3 DoD is green.

### Messaging (npm / GitHub)

| Surface | Text |
|---------|------|
| README badge / status | “**1.x = scaffold.** Target product is **2.0.0** (Grok Build base).” |
| `package.json` description (at 2.0 cut) | DeepSeek-native agent on Grok Build runtime |
| 1.x releases notes | “Legacy scaffold; prefer 2.x for product intent” |

---

## 2. Product definition of done — **`2.0.0`**

`2.0.0` is **not** “all specs green on paper.” It is:

### Must (P0) — ship blockers

1. **`dsb` / `deepseek-build` with no args** launches a **Grok Build–class full-screen coding agent** (TUI + agent loop), not a subcommand help line and not a bare `❯` line REPL as the only UX.
2. **Base runtime** is **derived from Grok Build open source** (fork, workspace vendor, or monorepo extract — see §4). Not a from-scratch reimplementation of “Grok vibes.”
3. **DeepSeek** is the **default model provider** (API key / setup onboarding works on first run).
4. **L1 minimum still true under that shell:** snippet-safe edit path, permission fail-closed, no YOLO-only default.
5. **L2 minimum still true:** stable tool/system prefix discipline (or documented equivalent in the Grok context stack with tests).
6. **Install story boring enough to dogfood:** binary or npm path that lands `dsb` on PATH and opens the agent.

### Should (P1) — same major if ready

7. Skills index vs body (Deep Code) without thrashing prefix.  
8. Flash-first / Pro escalate (or DeepSeek model routing equivalent).  
9. Parallel tools / bg shell / subagents **via Grok’s real mechanisms**, not only dsb MVP shims.  
10. Theme: readable DeepSeek blue accents in the TUI (not Grok-black-only default).

### Explicit non-goals for **2.0.0**

- Multi-vendor “works equally on Claude/GPT” as identity  
- Gajae multi-stage planning harness as core loop  
- Perfect 1:1 pixel clone of every Grok screen  
- Deleting 1.x from npm history  

---

## 3. Architecture plan

```text
┌────────────────────────────────────────────────────────────┐
│  bins: deepseek-build / dsb                                │
│  → entry = Grok pager/TUI composition root (renamed)       │
└───────────────────────────┬────────────────────────────────┘
                            │
        ┌───────────────────┼───────────────────┐
        ▼                   ▼                   ▼
   Grok agent runtime   Grok TUI/pager    Grok tools/shell
   (xai-grok-agent…)    (xai-grok-pager)  (native speed)
        │
        ▼
   DeepSeek provider adapter (from dsb-provider-deepseek + auth)
        │
        ▼
   L1/L2 policy overlays (snippet, permissions, cache epoch)
   ← port tests/contracts from 1.x dsb-tools / dsb-context
```

### Reuse from 1.x (keep — do not throw away)

| 1.x asset | Role in 2.0 |
|-----------|-------------|
| `dsb-provider-deepseek` | DeepSeek API client, SSE, models |
| `dsb-config` credentials + setup | First-run key onboarding |
| Specs 10/15/20/30/40/45/90/… | Contract tests / acceptance |
| `dsb-tools` snippet/permissions | Policy layer or port into Grok tool path |
| `dsb-context` prefix/epoch | L2 tests / adapter |
| CI product-ci gate | Keep product CI culture |
| Docs user-guide structure | Rewrite against real TUI |

### Likely deprecate as *product entry* (keep as lib/tests)

| 1.x | Note |
|-----|------|
| clap thin REPL as default UX | Secondary: `dsb repl-legacy` or drop from default |
| In-process subagent heuristics | Prefer Grok’s real subagent/worktree |
| Static-only MCP catalog | Prefer Grok MCP surface + DeepSeek auth |

---

## 4. Base integration strategies (pick one early ADR)

| Strategy | Pros | Cons | Recommendation |
|----------|------|------|----------------|
| **A. Fork grok-build → deepseek-build 2.x** | Cleanest “base” | License/sync cost; large tree | **Default preferred** if license OK |
| **B. Git subtree/submodule of grok-build** | Clear upstream pin | Awkward dual-root | OK for research spike |
| **C. Cargo path deps to sibling `../grok-build`** | Fast local | Bad for npm/release | **Spike only** |
| **D. Continue greenfield** | — | **Fails owner intent** | **Rejected** for 2.0 |

**First ADR for 2.0:** `docs/adr/00xx-grok-build-base.md` choosing A/B and license notes (Apache-2.0 compatibility, attribution, `SOURCE_REV` pin).

---

## 5. Execution waves (2.x train)

Plan id suggestion: **`grokbase-2x`** (ultragoal).

| Wave | SemVer band | Outcome |
|------|-------------|---------|
| **W0 Research** | docs only / `2.0.0-alpha.0` optional | Map grok crates; auth; provider plug points; license; minimal `cargo run` TUI |
| **W1 Shell** | `2.0.0-alpha.N` | `dsb` launches Grok TUI composition root; branding DeepSeek Build; setup/auth works |
| **W2 DeepSeek wire** | `2.0.0-beta.N` | Default models DeepSeek; chat/edit loop live dogfood |
| **W3 L1/L2 overlay** | `2.0.0-beta.N` | Snippet + permissions + prefix discipline under real shell |
| **W4 Product cut** | **`2.0.0`** | §2 P0 green; README/npm point here; 1.x marked legacy |

### PR unit principles (unchanged culture)

- Spec/ADR before large runtime moves  
- Small mergeable PRs; path-gated CI  
- No claiming gate green without evidence  
- **Do not** bump to `2.0.0` until W4 DoD  

---

## 6. Ultragoal chain (replace overnight A–D closure as product SSOT)

Previous chain closed **scaffold** work. New chain for product:

```text
replan-2.0 (this doc, ADR)
  → grokbase-w0-research
  → grokbase-w1-shell
  → grokbase-w2-deepseek
  → grokbase-w3-l1l2
  → grokbase-w4-cut-2.0.0
```

Cold-start prompt: `docs/product/ULTRAGOAL_PROMPT_COLD_START_2.0.md` (to be added with first research PR).

---

## 7. Communication plan

| Audience | Message |
|----------|---------|
| Owner | 1.x ≠ product; 2.0.0 is real goal; plan is Grok base |
| npm users of 1.x | Scaffold remains; breaking UX comes at 2.0.0 |
| Contributors | Stop extending thin REPL as if it were Grok; invest in base integration |

README top status (required after this replan merges):

> **Current npm `1.x` is a contract/scaffold line.**  
> **Product target: `2.0.0` — Grok Build–class agent (`dsb` opens full agent), DeepSeek-native, Deep Code + Reasonix overlays.**  
> See [REPLAN_2.0.md](./REPLAN_2.0.md).

---

## 8. Immediate next actions (ordered)

1. **Merge this replan** (docs + versioning honesty).  
2. **ADR: Grok Build base strategy** (A vs B) + license/attribution.  
3. **W0 spike PR:** build `xai-grok-pager-bin` from local `../grok-build`; document run path; list provider/auth injection points.  
4. **Freeze feature creep on 1.x REPL** unless it unblocks 2.0 spike.  
5. **Ultragoal `grokbase-2x` create-goals** after ADR.

---

## 9. Success feeling (2.0.0)

> I type `dsb` (or `deepseek-build`).  
> A **Grok-class coding agent** opens.  
> It uses **DeepSeek**.  
> Edits are **safe**, long sessions stay **affordable**, work stays **fast**.  
> I do not need a subcommand to “enter” the product.

Until that is true, **do not call the product done** — regardless of checklist green or prior 1.0.0 tags.
