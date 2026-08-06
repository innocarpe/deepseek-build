# Adversarial design review — harness philosophy

- **Reviewer:** Codex CLI (`model_reasoning_effort=xhigh`)
- **Date:** 2026-08-06
- **Scope:** HARNESS_PHILOSOPHY + SOURCES + specs index + PRD/MILESTONES/VISION
- **Verdict in review:** FAIL (pre-fix); product docs amended in same PR to address P0 contradictions

Claude Opus review was **not available** (Claude Code OAuth session expired).

---

## Verdict

FAIL — the four Deep Code/Reasonix pillars exist as prose, but every executable contract is still TODO and the milestone gates allow Grok-shaped runtime work to start first.

## Critical (P0)

1. **The “specs before code” gate is not real.**  
   Evidence: [`docs/specs/00-overview.md:9-27`] lists every runtime spec as `TODO`; the directory contains only `00-overview.md`. [`docs/product/PRD-v1.md:228-234`] permits “M1 implementation started” while the required specs remain unchecked.  
   Action: create and merge the concrete specs before runtime PRs. Change the launch gate from “implementation started” to “spec merged, acceptance tests defined, and implementation checked against it.”

2. **Grok still controls the actual execution order.**  
   Evidence: wall-clock progress is the north star in [`docs/product/MILESTONES.md:3-6`]; M2 explicitly prioritizes tools, parallel calls, and background shell in [`docs/product/MILESTONES.md:54-70`]; permissions do not arrive until M3 in [`docs/product/MILESTONES.md:74-89`]. The repo structure is also explicitly modeled primarily on Grok in [`docs/architecture/REPO_LAYOUT.md:37-52`].  
   Inference: an implementer can satisfy the visible M2 goal with generic Grok-style tools before snippet safety, permission policy, or cache tests exist. L1/L2 are labeled “owners” but are not release-blocking invariants.  
   Action: make snippet validity, cache-byte tests, and baseline permission enforcement prerequisites for M2. Add a milestone rule that no L3 feature can ship while its L1/L2 dependency is prose-only.

3. **The edit contract has bypasses.**  
   Evidence: [`docs/architecture/HARNESS_PHILOSOPHY.md:80-96`] constrains `edit`, but also exposes `write` and `bash`; [`docs/product/PRD-v1.md:93-102`] puts edit and shell in the M2 core tool set. Nothing states whether `write` may overwrite an existing file or whether `bash` may mutate files outside the edit contract.  
   The upstream Deep Code architecture makes the intended boundary explicit: `read` creates session-local snippets and `edit` requires them, with scoped replacement and version checks ([Deep Code architecture](https://github.com/lessweb/deepcode-cli/blob/main/docs/architecture_en.md)).  
   Action: specify create-vs-overwrite semantics for `write`, forbid or explicitly classify shell-based file mutation, define binary/generated-file behavior, and add stale-version, repeated-match, race, and bypass tests.

4. **“Byte-stable cache” is not an implementable contract.**  
   Evidence: [`docs/architecture/HARNESS_PHILOSOPHY.md:98-126`] names the layout but does not define canonical serialization, field ordering, whitespace, newline/Unicode rules, timestamp/cwd handling, or invalidation. [`docs/product/PRD-v1.md:37,145-147,188`] accepts “telemetry of cache hit (or proxy),” and [`docs/product/MILESTONES.md:47-50`] asks for “reuse intent,” not proof.  
   Action: spec 10 must define exact request-byte construction, stable-prefix epochs, skill/instruction snapshots, schema changes, replay/compaction, interrupted calls, and worker prefixes. Require golden byte fixtures plus provider cache hit/miss evidence; a proxy must not substitute for the invariant.

5. **Reasonix economics are slogans, not routing behavior.**  
   Evidence: Flash-first and Pro escalation are stated in [`docs/architecture/HARNESS_PHILOSOPHY.md:161-173,196-204`] and [`docs/product/PRD-v1.md:34-39`], but there are no escalation signals, token/cost budgets, price source, cache billing assumptions, fallback rules, or precedence between user choice and router choice.  
   Action: specs 20/30 and the provider contract must define model selection precedence, escalation thresholds, per-turn budgets, unavailable-model behavior, explicit Pro visibility, and usage/cache telemetry. Pin the actual provider model identifiers instead of treating `deepseek-v4-flash` and `deepseek-v4-pro` as settled facts.

6. **Tool-call repair is internally classified as both M1 and M6.**  
   Evidence: philosophy and the spec index make repair a Reasonix pillar in [`docs/architecture/HARNESS_PHILOSOPHY.md:165-170`] and [`docs/specs/00-overview.md:14-16,41-51`]. M1 includes it in [`docs/product/MILESTONES.md:36-50`]. But [`docs/product/PRD-v1.md:104-111`] demotes it to an M6 “should have.”  
   Action: resolve the milestone now. If the M1 loop dispatches tools, repair and tool/result pairing must be specified and tested in M1. Define allowed repairs, schema validation, retry limits, semantic-error boundaries, audit output, and “never dispatch invalid side effects” behavior.

7. **DeepSeek provider behavior is missing from the contract.**  
   Evidence: [`docs/product/PRD-v1.md:171-180`] only names an endpoint, models, and generic OpenAI compatibility; [`docs/product/MILESTONES.md:45-50`] already calls for streaming, cache handling, repair, and effort flags.  
   Action: write a provider ADR/spec covering request/response schemas, streaming events, thinking and effort fields, cancellation, retries, rate limits, cache headers/usage fields, authentication, model capability detection, and version pinning before implementing the client.

## Major (P1)

- **Skills can silently destroy the cache invariant.**  
  [`docs/architecture/HARNESS_PHILOSOPHY.md:128-141`] says the skill index may be stable and bodies go in the tail, but does not define path precedence, deterministic ordering, duplicate names, symlinks, discovery snapshots, hot reload, body size limits, or what starts a new cache epoch. Write spec 70 together with the cache contract.

- **Permission classification is far too vague to enforce.**  
  [`docs/architecture/HARNESS_PHILOSOPHY.md:143-157`] names scopes but not symlink resolution, path traversal, nested workspaces, Git worktrees, shell pipelines, redirection, command substitution, environment leakage, network detection, approval lifetime, or concurrent approval handling. [`docs/product/NON_GOALS.md:14`] only says YOLO-only mode is out of scope. Spec 90 needs a concrete policy state machine and audit schema before shell exists.

- **Parallel execution has no correctness semantics.**  
  [`docs/product/MILESTONES.md:63-70`] mentions dispatch and result ordering, but not deterministic ordering, cancellation, timeout, partial failure, stdout/stderr limits, conflicting edits, approval races, or whether tools commit effects as they finish or in planned order. Spec 50 must define these before parallel tools ship.

- **Subagent cache law is aspirational.**  
  [`docs/architecture/HARNESS_PHILOSOPHY.md:187-192`] says templates should be shared “where possible” and workers should prefer Flash. There is no required template identity, context budget, summary schema, permission inheritance, model override rule, or cache-cost measurement. Spec 60 must turn “where possible” into a measurable prohibition against unique cold prefixes.

- **MCP directly conflicts with stable schemas unless its lifecycle is specified.**  
  The philosophy permits dynamic MCP mounting in [`docs/architecture/HARNESS_PHILOSOPHY.md:90-92`], while forbidding mid-session stable-schema rewrites in [`docs/architecture/HARNESS_PHILOSOPHY.md:120-126`]. [`docs/product/MILESTONES.md:122-125`] only says MCP must avoid cache thrashing. Spec 80 must define discovery timing, schema snapshots, tool removal, permission classification, and cache-epoch rollover.

- **Sessions, replay, and compaction are deferred while cache claims already depend on them.**  
  [`docs/architecture/HARNESS_PHILOSOPHY.md:118-126`] requires replayable logs and repaired pairings, but sessions are scheduled for M5 in [`docs/product/MILESTONES.md:111-127`]. [`docs/product/PRD-v1.md:143-147`] promises compaction behavior without defining compaction. Decide whether session persistence is an M1 dependency and specify compaction before claiming long-session economics.

- **The success metrics cannot establish the product thesis.**  
  [`docs/architecture/HARNESS_PHILOSOPHY.md:34`], [`docs/product/VISION.md:7-11,33-38`], and [`docs/product/PRD-v1.md:182-192`] use “quality per dollar,” “wall-clock progress,” and “affordable” without task corpus, baseline, cost model, latency definition, or minimum acceptance threshold. Add a small adversarial benchmark and cost/latency measurement protocol before making comparative claims.

- **Toolchain, binary, config, and authentication remain open while M1 is supposed to start.**  
  [`docs/product/PRD-v1.md:171-180,218-224`] leaves language, package name, config path, and compaction unresolved. [`docs/product/MILESTONES.md:39-50`] nevertheless schedules provider implementation. The toolchain/config ADR must precede repository runtime scaffolding.

## Minor (P2)

- **The upstream reference is floating.**  
  [`docs/product/SOURCES.md:20-24`] and [`docs/research/deepcode-cli.md:3-9`] point at `main`. Pin the reviewed Deep Code commit or archive the relevant evidence so later upstream edits do not silently change the design basis.

- **Document status is inconsistent.**  
  [`docs/architecture/HARNESS_PHILOSOPHY.md:3-7`] is normative, [`docs/product/PRD-v1.md:5`] says “Draft → active,” [`docs/product/VISION.md:33`] calls the signals “v0,” and every spec is still `TODO`. Define one status vocabulary and one authority for readiness.

- **The UX is explicitly non-contractual.**  
  [`docs/product/PRD-v1.md:155-169`] says exact command names may differ, while no spec covers interrupt behavior, TUI/CLI acceptance, streamed thinking display, or error presentation. That invites incompatible implementations and brittle user expectations.

- **Layer ownership is ambiguous around cache.**  
  [`docs/architecture/HARNESS_PHILOSOPHY.md:44-54`] places cache-related concerns in both L1 and L2; [`docs/product/SOURCES.md:8-12`] calls Reasonix both cache co-owner and L2 primary. State which document wins when Deep Code session semantics conflict with Reasonix byte-stability rules.

## Missing docs/specs before code

1. Amend [`docs/product/PRD-v1.md`], [`docs/product/MILESTONES.md`], and [`docs/specs/00-overview.md`] to resolve gates, statuses, and M1/M6 contradictions.
2. Accept the toolchain/config ADR: language, package/binary name, supported platforms, state directory, project config, credential storage, and secret handling.
3. Write the DeepSeek provider contract: models, API fields, streaming, thinking, effort, errors, cancellation, retries, rate limits, and cache usage reporting.
4. Write spec 10: canonical context/message serialization, stable-prefix byte rules, volatile-tail rules, epoch/invalidation policy, replay, compaction, and golden fixtures.
5. Write spec 15: schema-aware tool-call repair, tool/result pairing repair, retry limits, auditability, and side-effect dispatch barriers.
6. Write specs 20 and 30: Flash/Pro routing, user overrides, escalation thresholds, budgets, fallback behavior, thinking display, and effort mapping.
7. Write spec 120: project instruction discovery, precedence, deterministic ordering, file-change invalidation, and interaction with the cache prefix.
8. Write spec 70: skill discovery paths, index format, matching, opt-out, body loading, deduplication, size limits, and cache placement.
9. Write spec 45 before spec 40: snippet identity, ranges, versions, encodings, scope, ambiguity candidates, atomic replacement, external mutation, concurrency, and `write` semantics.
10. Write spec 90 before enabling shell/file/network tools: path classification, shell effect declarations, approval policy, deny behavior, symlinks, Git, MCP, and audit records.
11. Write spec 40: the complete small built-in tool registry, argument schemas, outputs, errors, truncation, cancellation, and which operations are prohibited.
12. Write spec 50: parallel dispatch, ordering, cancellation, timeouts, partial failure, background task lifecycle, output collection, and edit conflict handling.
13. Define the adversarial acceptance suite: exact-prefix golden tests, replay repair, malformed tool JSON, stale snippets, ambiguous matches, shell bypasses, permission races, parallel conflicts, Flash/Pro routing, and cache telemetry.
14. Before M4/M5 code, write specs 60, 80, 100, and 110 for subagents, MCP lifecycle, session new/resume/fork, and light plan mode. Their absence must remain a hard gate for those milestones.

## Contradictions

| Location A | Location B | Conflict | Required resolution |
|---|---|---|---|
| [`docs/product/MILESTONES.md:36-50`] | [`docs/product/PRD-v1.md:104-111`] | Tool-call repair is an M1 work item but an M6 “should have.” | Choose M1 or M6; the architecture currently requires it for the tool loop. |
| [`docs/specs/00-overview.md:14-16,41-51`] | [`docs/product/PRD-v1.md:104-111`] | Spec 15 is in the M1 sequence, but repair is deferred to M6. | Align the MVP cut and PRD capability tier. |
| [`docs/product/PRD-v1.md:36-39`] and [`:143-147`] | [`docs/product/PRD-v1.md:104-107`] | Cache/cost visibility is a goal and part of the long-session journey, but UI indicators are deferred to M6. | Define whether M1 measures only provider telemetry or whether user-visible indicators are required earlier. |
| [`docs/product/PRD-v1.md:89-92`] and [`docs/product/MILESTONES.md:31-50`] | [`docs/product/MILESTONES.md:74-89`] | Thinking/effort is an M1 spec/work item and a must-have, but the DeepSeek surface is assigned to M3. | Put provider wiring in M1 and UX completion in M3, explicitly. |
| [`docs/specs/00-overview.md:3-7`] | [`docs/product/PRD-v1.md:228-234`] | Empty specs prohibit silent invention, but the launch definition permits starting M1 implementation with specs unchecked. | Make merged ready-for-implementation specs a prerequisite, not a checkbox after work starts. |
| [`docs/architecture/HARNESS_PHILOSOPHY.md:149-153`] | [`docs/product/MILESTONES.md:54-80`] | Bash must declare effects and policy must decide, but shell is enabled in M2 before permissions are delivered in M3. | Move a minimum permission contract into M2 or prohibit mutating shell until M3. |
| [`docs/architecture/HARNESS_PHILOSOPHY.md:44-68`] | [`docs/product/MILESTONES.md:6,54-107`] | L1/L2 are declared authoritative, but M2/M4 success is framed primarily around Grok speed and fan-out. | Add explicit L1/L2 invariant gates and failure criteria to every L3 milestone. |
| [`docs/product/SOURCES.md:103-112`] | [`docs/architecture/HARNESS_PHILOSOPHY.md:118-126,229-246`] | Sources says specs sit below the philosophy, but the philosophy treats specs as the place where cache, repair, skills, and permissions become testable; all are currently absent. | Do not treat the prose hierarchy as enforcement; require concrete spec-to-test links before implementation. |
