# Cold start — `deepseek-native-depth-6x` (dsh reflection)

> [!IMPORTANT]
> **Status: proposed, awaiting the board's PR merge.** The board it executes is
> `docs/product/DEEPSEEK_NATIVE_DEPTH_6X_GOALS.md`, carried by
> [PR #204](https://github.com/innocarpe/deepseek-build/pull/204). If that PR is
> not merged, start from the branch and re-check the units, not from this text.
>
> This prompt is written for a session opened **in a fresh worktree of this
> repo**, by path (`skills/worktree-dispatch`). It does not assume a `dsb` or
> `grok` binary — it assumes the tree, `git`, `gh`, and cargo.

---

## 0. Read this first (the two-paragraph version)

The product is **DeepSeek Build** (`deepseek-build` / `dsb`), a DeepSeek-native
coding agent built on a vendored open-source Grok Build tree plus an overlay.
The **`6.0.0`** base port just landed: the vendored tree moved from Grok Build
`1.0.0` to `1.0.41` (PR #200, merged at `593a1e4`). Its release cut is owned by
**another lane** — do not touch versions, CHANGELOG, tags or npm.

Your work is the **next** thing: **`6.1.0`**, the DeepSeek-native depth
continuation (`docs/product/PRD-v6.md` §7). It reflects DeepSeek's **official**
open-source harness, `dsh` (`deepseek-ai/deepseek-harness`), into this product.
The research is already done — `docs/research/dsh-deepseek-harness.md` — and
the board is re-measured against the `1.0.41` tree.

**The single most important thing to internalize:** half of the dsh items are
**already implemented** in this base. The board §1 lists them with file paths.
Do not rebuild them. Your job is the **remaining** half, and the first unit is
an investigation that decides whether the rest is even possible.

---

## 1. Immutable facts (verified 2026-09-25; re-verify before trusting)

| Fact | Value | How to re-check |
|---|---|---|
| Base merged | Grok Build `1.0.41` (PR #200, `593a1e4`) | `git fetch origin main && git log --oneline -3 origin/main` |
| Vendor pin | `036a5d83…` | `git show origin/main:third_party/grok-build/SOURCE_REV` |
| Product version | **`5.7.0`** on disk; `6.0.0` release cut is another lane's | `rg '^version' Cargo.toml` |
| **Path A = the vendored tree** | `xai-grok-shell` does **not** depend on `dsb-context` | `rg dsb third_party/grok-build/crates/codegen/xai-grok-shell/Cargo.toml` (empty) |
| Live prefix assembly | `third_party/grok-build/crates/codegen/xai-grok-shell/src/session/helpers/spec10_path_a_assembly.rs`, called from `session/acp_session_impl/turn.rs:2187` | `rg apply_spec10_to_conversation_request third_party/grok-build/crates/` |
| Path A transport | `api_backend = "chat_completions"` | `rg 'api_backend' crates/dsb-cli/src/agent_launch.rs` |
| Cache key reachability | `prompt_cache_key` reaches the wire **only** on the Responses mapping | `third_party/grok-build/crates/codegen/xai-grok-sampling-types/src/types.rs:1068-1071` |

**SemVer rule (fail-close):** always write full `MAJOR.MINOR.PATCH`. Never write
`6.1` or `v6`. Never invent a version already on `main`, tagged, or published.

---

## 2. Non-negotiables

1. **Do not touch the `6.0.0` release lane.** No version bump, no CHANGELOG
   edit, no tag, no `npm publish`. Another session owns it.
2. **Do not touch `main` or the primary checkout.** You work in your own
   worktree; the primary checkout is a control tower that stays clean.
3. **Do not touch another session's worktree.** One exists on
   `feat/cache-attribution` and is working the cache-attribution trio. See §4.
4. **Any product-behavior change lands in the vendored tree.** A change confined
   to `crates/dsb-*` cannot control Path A — the overlay's Path A function only
   stamps an epoch file at launch. Only Path A counts for product P0
   ([OWNER_BAR_ACCEPTANCE.md](./OWNER_BAR_ACCEPTANCE.md) §2.1).
5. **`third_party/grok-build` builds are serial across all worktrees.** A cold
   build is 30–60+ minutes. Check before you start one:
   `pgrep -fl 'grok-build/target'`. If a build is running, wait or ask.
6. **Vendored edits follow the carried-patch discipline.**
   [GROK_VENDOR.md](../architecture/GROK_VENDOR.md) lists in-tree deviations;
   a future `grok-sync` re-applies them. Add your deviation to that list.
7. **`gh` per command, never `gh auth switch`:**
   `GH_TOKEN="$(gh auth token --user innocarpe)" gh … --repo innocarpe/deepseek-build`.
   `gh pr create` also needs `--head <your-branch>`.
8. **This repo is public.** No private paths, no personal context, no company
   names. Home paths as `~`.

---

## 3. Your units, in order

Full detail is in the board. This is the execution order and what each unit is
for.

### U0.2 — the wire inventory (do this first)

**Why first:** it decides the fate of five other units. Path A runs Chat
Completions, and `prompt_cache_key` only reaches the wire on the Responses
mapping. If the product's cache keys are inert on its own backend, then
"cache-miss attribution" has nothing to attribute, and the highest-value dsh
idea — appending a changed prompt or tool set *after* cached history — may not
be expressible at all.

**Deliverable: evidence, not a decision.** Answer three questions, each with a
path or a measurement:

1. Does a cache key reach the wire on `api_backend = "chat_completions"`?
   Trace it in code, then confirm on a real request if a route is available.
2. What does the DeepSeek usage payload actually report for cache reads on this
   backend? Name the exact field(s).
3. Can an in-history prompt or tool update be expressed on this transport?
   This is the `systemPromptUpdate: 'in-history'` / `toolUpdate` capability that
   dsh gets from the Messages path
   (`docs/research/dsh-deepseek-harness.md` §6).

**Output:** a research note under `docs/research/`, plus a short comment on the
board's PR stating which of the five gated units survive. Do **not** change
product code in this unit.

### U1.1 — runtime invariant: the request equals the log projection

The dsh source checks, on every model request, that the payload matches what the
session log says it should be, and fails loudly on divergence
(`docs/research/dsh-deepseek-harness.md` §3). This product has no such check.
Add one **at the sampling boundary in the vendored tree** — not in the overlay.
Also make an unrestorable log entry fail closed instead of silently resuming.

### U1.2 — context beside a tool result

A result-preserving path for model-visible notices (repeat-call nudges, policy
notes) so they do not have to be smuggled inside the tool's own output. The
hook system has `PostToolUse`/`PostToolUseFailure` but no inject path.

### U1.3 — a deny-only guard

A policy hook that may deny or abstain but can **never** allow, so listener
ordering cannot resurrect an action a stricter layer refused.

### Wave 2 and 3 — gated on U0.2

Cache-key reachability, attribution, cumulative surface (Wave 2); the in-history
prompt/tool update (Wave 3). **Do not start these before U0.2 reports.**

---

## 4. The coordination hazard you must respect

A separate session is working `feat/cache-attribution`: cache-miss attribution,
a cumulative hit/miss surface, and a scored regression bench. Its brief states
that the cache layer is the product workspace and that it only touches `crates/`.

**For Path A that premise is wrong.** The live assembly is
`spec10_path_a_assembly.rs` in the vendored tree;
`crates/dsb-context::assemble_path_a_context` is invoked at launch only to stamp
an epoch file, with placeholder inputs.

So: **do not duplicate that session's work, and do not fix its premise
yourself.** Report the discrepancy to the tower with the two file paths above
and let it decide whether that session redirects into the vendored tree or hands
its spec and tests over for you to wire. If you are asked to take it over, the
board's U2.2 is where it lands.

---

## 5. Floor check (re-run at the start of every session)

```sh
git fetch origin main
git log --oneline -3 origin/main
git show origin/main:third_party/grok-build/SOURCE_REV
rg '^version' Cargo.toml
npm view @innocarpe/deepseek-build version
gh pr list --repo innocarpe/deepseek-build --state open
```

If `main` moved past `593a1e4`, re-read the board's §1 before trusting its file
paths — a newer base may have filled more of the gaps.

---

## 6. Per-PR requirements

Every PR follows [`skills/pr-authoring`](../../skills/pr-authoring/SKILL.md) and
[pr-body-standard.md](../contributing/pr-body-standard.md), and answers:

- **Cache impact** — none / low / medium / high, and why.
- **Which layer** — L1, L2, or L3.
- **Path A reachability** — does this change code the installed binary actually
  runs? Name the call site, or say plainly that it does not.
- **Measured or asserted** — for any cost or reuse claim, name the measurement.
- **What was not taken** — which dsh rows this PR declined, and why.

---

## 7. Definition of done

1. U0.2's evidence exists, and the board says which gated units survive.
2. U1.1–U1.3 are merged with tests that fail when the property they protect is
   broken — a guard is only a guard when a regression makes it fail.
3. Wave 2/3 either merged or explicitly dropped, with the reason recorded.
4. The *not taken* list exists, with a file path per row (board §1).
5. No release-lane artifacts were touched.

---

## 8. What would make you stop and ask

- The board's PR is not merged — then start from its branch and re-verify units.
- U0.2 shows the transport cannot do an in-history update — that removes Wave 3
  from this line; report rather than improvise.
- U1.1 turns out to have no seam inside the live path — that would make it Path B
  theater. Stop and report; do not ship a check nothing calls.
- Two units in a row cannot be measured — the measurement story is wrong.
- A vendored build is already running somewhere else.
