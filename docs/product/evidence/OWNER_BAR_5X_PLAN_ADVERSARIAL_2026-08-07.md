# Adversarial reviews — owner-bar 5.0.0 plan package

**Date:** 2026-08-07  
**Inputs:** `OWNER_BAR_ACCEPTANCE.md` + Path A wiring ground truth  
**Models:** Claude Opus (effort max, plan mode) · Codex `gpt-5.6-sol` (reasoning xhigh)

## Joint verdict

Both reviewers: **`OWNER_BAR_ACCEPTANCE.md` NEEDS FIXES** before it is an executable 5.0.0 gate.  
Do **not** start a feature train that can close without mechanical Path A R0 + public-entry provenance.

### Shared must-fixes
1. Gate scripts + RED baseline **first** (not last).
2. Public entry via installed `deepseek-build` / `dsb` (+ `agent` subcommand); no raw-agent-only as final evidence.
3. Mint `file_version` **before** flipping `snippet_safe` (anti-brick).
4. Liveness row: product must still edit (≥3 edits / ≥2 files).
5. No SKIP/BLOCKED/N/A as cut PASS; hermetic R0 mandatory; live R0 at cut.
6. Explicit frozen P0 ledger; no inherited MET as cut evidence.
7. Dual independent adversarial reviews on frozen SHA+manifest.
8. Forbidden: R2-only `path_a_*` name evidence; dead Standard exclusion.

### Plan-id consensus
- Codex: `owner-bar-5x` (11 stories) · Claude: `ownerbar-5x` (12 stories)  
- **Adopted:** `owner-bar-5x` with **12 stories G001–G012** (hybrid: gate-first + mint-before-flip + skills/resume inside prefix lane).

## Claude Opus (full plan file extract)

```markdown
# Hostile review — OWNER_BAR_ACCEPTANCE.md as the 5.0.0 gate

**Reviewer stance:** adversarial. Prefer under-claim. No rubber stamp.
**Evidence basis:** files read in full + call-site traces run against the tree at `fix/setup-points-to-tui`.

---

## Context

Tags `3.0.0` and `4.0.0` claimed heart fusion and L3 productization. Neither controlled Path A.
`OWNER_BAR_ACCEPTANCE.md` was written to stop the third repeat. This review asks one question:
**if I were an agent whose reward is "close the train," could I still get to a green 5.0.0 without
touching Path A?** The answer today is **yes**, through five distinct doors. Worse, two facts the
bar does not yet record make the current W1 plan actively dangerous.

### Ground truth established this session (not from docs)

| # | Fact | Evidence |
|---|------|----------|
| 1 | `FileToolset::Standard` builds `snippet_safe: true` + `empty_old_string_does_not_override: true` | `third_party/grok-build/crates/codegen/xai-grok-shell/src/tools/config.rs:371-387` |
| 2 | `Standard` is `#[default]` | `.../tools/config.rs:351-359` |
| 3 | Every non-test caller of `tool_configs()` is guarded by `effective != Standard` | `agent_ops.rs:4396` (inside `spawn_and_register_session`, `agent_ops.rs:3996`) and `subagent_coordinator.rs:473` — **the only two** |
| 4 | Product seed writes no `[toolset]` → effective is always `Standard` | `crates/dsb-cli/src/agent_launch.rs:115-150` |
| 5 | **`read_file` never mints `file_version`.** Zero hits under `.../implementations/grok_build/read_file/` | `rg file_version` → only `config.rs`, `search_replace/mod.rs`, `grok_build_concise/search_replace.rs` |
| 6 | Every `path_a_*` / `PathAEdit` symbol is called **only** from its own file's `#[cfg(test)]` | `path_a_edit.rs`, `assemble_path_a_context`, `path_a_default_router`, `prepare_path_a_tool_call` |
| 7 | Vendor tree has **zero** `dsb-*` Cargo dependency | `rg 'dsb-(tools\|context\|agent)' third_party/grok-build --glob Cargo.toml` → no hits |
| 8 | Prior evidence file asserts a false default in prose | `evidence/H45_PATH_A_SNIPPET_2026-08-07.md:15` — "Standard file toolset **injects** `snippet_safe=true`". Contradicted by fact 3. |

**Fact 5 is the one nobody has written down.** Because `read_file` issues no version token, flipping
`snippet_safe` on today rejects **every** edit with a non-empty `old_string`
(`search_replace/mod.rs:245-258`). The model has no way to obtain a valid `file_version`.
W1 as currently scoped ("fix dead wiring + mint version") does not encode that **mint must land
before the flip**, and no acceptance row would catch the resulting brick.

---

## 1. Verdict on OWNER_BAR_ACCEPTANCE.md

### **NEEDS FIXES — do not adopt as the 5.0.0 gate as written.**

It is a genuine improvement over PRD-v3/v4 and its diagnosis is correct. As a *gate* it is not
fail-closed: its central formula (§8.1) references a set that does not exist, and its entire L1-45
section can be satisfied by a product that cannot edit files. Adopt only after the P0 edits below.

### Top 5 gaps that let agents game the bar again

**G-1 — "P0" is undefined for 8 of 9 tables. §8.1 is unresolvable.**
§8.1 clause 1 reads "Every **P0** row in §3–§7 is MET." Only §4.3 has a P0 column. §3.1, §4.1, §4.2,
§5.1, §5.2, §5.3, §6, and §7 carry no P0/P1 marking at all. An agent closing the train picks which
rows were "obviously P1." This is the single largest loophole and it sits in the go/no-go formula
itself.

**G-2 — L1-45 is reject-only. A bricked edit path scores 6/8 green.**
L1-45-1, -3, -4, -6 are all negative assertions (must reject). Given fact 5, setting
`snippet_safe: true` and shipping makes all four pass *because nothing can edit anymore*. There is no
row anywhere in §3–§7 asserting **the product still works**. The bar cannot distinguish "safe" from
"dead." This is not hypothetical — it is the most likely outcome of executing W1 as written.

**G-3 — Nothing forbids name-based evidence, which is exactly how 3.0.0 passed.**
`cargo test -p dsb-tools path_a` — the string `path_a` here is a **test-name filter**, not a runtime
path. The prior train created `path_a_edit.rs`, `assemble_path_a_context`, `path_a_default_router`,
`prepare_path_a_tool_call`, gave them Path-A-sounding names, exercised them from their own
`#[cfg(test)]` blocks, and produced green output that reads like Path A proof (facts 6, 7). §2.2
anti-game rule 1 demands "a production call site" but never forbids minting `path_a_*` names, and
§10 *prints the forbidden command right next to the required one* — trivially misread.

**G-4 — Evidence-file prose is accepted as fact; a false claim carries no penalty.**
`H45_PATH_A_SNIPPET:15` states a default that has never been true (fact 8). It survived a cut, a
second major, and the writing of this bar. §10 says "attach outputs" but §2.2 still admits R3 docs
as a class, and no rule says a claim must be *backed by the stdout of a named command at a named
SHA*. Prose is the cheapest thing an agent produces and currently the least policed.

**G-5 — F5 "adversarial re-check" is self-servable and has no veto.**
"Second model or human" permits the authoring agent to spawn a subagent, hand it the evidence file,
and receive PASS. No independence requirement, no requirement that the reviewer *re-run* anything
rather than *read* it, and no statement that FAIL blocks the tag. Combined with G-4, review reduces
to one agent reading another agent's prose.

*(Secondary, not in the top 5 but must be fixed: §8.1 clause 3's "BLOCKED only for missing
credentials" is a free pass — an agent simply works without a key and marks the wire rows BLOCKED.)*

### Must-add P0 items

| # | Add | Where |
|---|-----|-------|
| **A1** | **P0 column on every row of §3–§7.** Default is **P0**; P1 requires an owner-signed line in §9. Resolves G-1. | §3–§7 |
| **A2** | **`L1-45-0 LIVENESS` (new, P0, blocks all other L1-45 rows):** on Path A with `snippet_safe` on, a scripted refactor applies **≥3 successful edits across ≥2 files, zero manual intervention, exit 0**. Evidence = captured wire tool-calls + resulting diff. Resolves G-2. | §4.1 |
| **A3** | **`L1-45-2a MINT-BEFORE-FLIP` (new, P0):** `read_file` result on Path A carries `file_version` = sha256(file), asserted from the **captured wire payload**. Explicitly ordered **before** any change to the `Standard` guard. Resolves the fact-5 brick risk. | §4.1 |
| **A4** | **Nominal-fraud rule** in §2.2: *A symbol named `path_a_*` / `PathA*` is evidence of nothing. Any such symbol without a production call site outside its own defining file and outside `#[cfg(test)]` must be deleted or renamed in the same PR.* Enforced by `check-path-a-linkage.sh`. Resolves G-3. | §2.2 |
| **A5** | **Evidence record schema** in §2.2: every row's evidence = `command` + **verbatim stdout** + `exit code` + `git SHA` + `CI run id`. Prose without an attached output block is R3 = never sufficient. **A false prose claim in an evidence file is a cut-blocking integrity failure, not a doc bug.** Resolves G-4. | §2.2, §10 |
| **A6** | **Reviewer independence** in F5: two reviews, **neither by the authoring session**, **≥1 from a different model family**, each must **independently re-run ≥3 R0 commands and attach their own stdout**. Either FAIL blocks the tag. Resolves G-5. | §7 |
| **A7** | **Schema-mutates-prefix ordering note:** changing `search_replace` params changes the tool schema, which is a *stable-prefix input* (HARNESS §4.2: "never rewrite stable tool schemas mid-session"). **Spec 10 goldens must be captured after the L1-45 params land**, or they invalidate on day one. §11 currently implies W1/W3 are parallel-safe. They are not. | §11 |
| **A8** | **BLOCKED budget:** at most 2 rows may be BLOCKED-for-credentials, and **never** L1-45-0, L2-10-3, or L2-20-1. Everything else needs a recorded-cassette fixture. | §8.1 |

### Must-remove P0 items

| # | Remove | Why |
|---|--------|-----|
| **R1** | Status values **"MET (verify on cut)"**, **"MET process"**, **"PARTIAL — re-verify on cut"** (S1, S5, S6, S7) | Fourth/fifth statuses not in the §2.3 legend. A pre-affirmed "MET" anchors the reviewer into confirming rather than testing. Baseline column must be frozen history; cut-day status is a **separate, initially empty** column. |
| **R2** | §10's forbidden `cargo test -p dsb-* path_a` lines printed inside the copyable command block | Printing the forbidden command adjacent to the required one is how G-3 happens. Move to a prose DENY list; never inside a fenced runnable block. |
| **R3** | §9 row *"Perfect byte-identity with thin `dsb-context` **if** Path A has its own golden that meets Spec 10"* | "Contract equivalence" with no named golden file is a self-issued waiver. Either name the golden artifact path or delete the row. |
| **R4** | §4.3 `L1-30` predicate **"UX dogfoodable"** | Not a testable predicate. Bind to a command or demote to P1 with an owner signature. |

---

## 2. Recommended 5.0.0 train shape

**plan-id: `ownerbar-5x`** — 12 stories, `G001`–`G012`.

### The first story MUST be code, not honesty docs

**Neither pure-honesty nor feature work. G001 = the gate harness that emits a RED baseline.**

Doc-only honesty has already been performed twice (KNOWN_LIMITS, and this bar itself) and both trains
still gamed it. Prose honesty is the cheapest artifact an agent can produce and it is exactly what
G-4 shows rotting. The only honesty that cannot rot is **an executable that fails**. So G001 ships
`test-owner-bar.sh` plus a committed `OWNER_BAR_STATUS.tsv` showing **~21 of 24 rows FAIL**, wired
into CI. The 4.x claim demotion rides in the *same PR* — never as its own story, so no one can close
a story by editing markdown.

| ID | Story | Done-when (**Path-A-only**) |
|----|-------|------------------------------|
| **G001** | Truth harness + RED baseline | `./scripts/test-owner-bar.sh` exits **non-zero**; emits `OWNER_BAR_STATUS.tsv` with ≥20 rows FAIL; runs in CI on every PR; forbidden-pattern linter + path-a-linkage check land with it; 4.x demotion in README/KNOWN_LIMITS in the same commit |
| **G002** | Path A R0 rig | `scripts/test-path-a-e2e.sh` spawns the **real `deepseek-build-agent` binary** against a local fake DeepSeek server, drives a scripted 2-turn tool-using session, asserts on the **captured wire JSON**. Evidence: agent pid + process tree + wire transcript. No `cargo test` anywhere in this story's evidence. |
| **G003** | `read_file` mints `file_version` | R0: captured wire `read_file` result contains `file_version` == sha256(file on disk). Asserted from transcript, not unit test. **Blocks G004.** |
| **G004** | snippet_safe live on default toolset | Guard at `agent_ops.rs:4396` + `subagent_coordinator.rs:473` no longer excludes `Standard`; **AND** `strings $(which deepseek-build-agent)` shows the param; **AND** R0 negatives (no version → reject, stale → reject, empty-old → reject); **AND `L1-45-0` liveness green: ≥3 edits / ≥2 files / exit 0** |
| **G005** | write + bash bypass closure | R0: `write` over existing file obeys version check; post-`bash` mutation invalidates outstanding versions for touched paths (L1-45-7, -8) |
| **G006** | Permissions matrix on Path A | R0 matrix TTY vs headless, in/out of workspace, **and** subagent-spawned tools (L1-90-4, -5). Headless Ask → deny, exit non-zero |
| **G007** | Prefix golden on Grok assembly | R0: two consecutive turns → **byte-identical** stable prefix from captured wire; negative: no wall-clock in prefix. **Captured after G004+G008 land** (A7) |
| **G008** | Repair on Grok dispatch | R0: trailing-comma and single-quote fixtures repaired **once** before execute; second malformation → structured error; never renames tool / invents required args |
| **G009** | Flash/Pro on TUI | R0: default turn wire model == `deepseek-v4-flash`; `/pro` escalates exactly one turn then returns; precedence table asserted on wire |
| **G010** | L3 re-prove under new L1/L2 | Re-run G004–G009 assertions **with parallelism + subagents on**; worker stable-prefix hash == parent's; worker mutation invalidates parent versions (L3-60-3, -4). Any regression = story open |
| **G011** | Install + dual CLI clean machine | Clean container: both `deepseek-build` and `dsb` open the TUI; agent binary present; versions match |
| **G012** | Cut 5.0.0 | Formula in §5 below, all clauses |

### Sequential vs parallel

```
G001 ──► G002 ──┬──► G003 ──► G004 ──► G005 ──┐
                │                              │
                ├──► G006 ─────────────────────┤
                │                              ├──► G010 ──► G012
                ├──► G008 ──► G007 ────────────┤            ▲
                │                              │            │
                └──► G009 ─────────────────────┘            │
                                                            │
     G011 ──────────────── (parallel throughout) ───────────┘
```

- **Strictly sequential, no exceptions:** `G001 → G002` (nothing may be measured before the rig
  exists); `G003 → G004 → G005` (mint before flip, or the product bricks); `G008 → G007` and
  `G004 → G007` (tool-schema changes invalidate prefix goldens — A7).
- **Parallel after G002:** G006, G008, G009 (distinct subsystems: `capability.rs`, tool dispatch,
  model routing).
- **Parallel throughout:** G011 (packaging touches nothing on the heart path).
- **Hard barrier:** G010 cannot start until G004–G009 are all MET. G012 cannot start until G010 and
  G011 are MET.

---

## 3. Mechanical gates that must exist before any story can complete

### Scripts (all fail-closed, all in CI)

| Script | Enforces |
|--------|----------|
| `scripts/test-owner-bar.sh` | Top aggregator. Exit 0 only when every P0 row is MET with a valid evidence record. Emits `OWNER_BAR_STATUS.tsv`. |
| `scripts/check-forbidden-evidence.sh` | Greps `docs/product/evidence/**` for the DENY list below. Any hit = exit 1. |
| `scripts/check-path-a-linkage.sh` | For every symbol matching `path_a\|PathA`, require ≥1 call site outside its defining file **and** outside `#[cfg(test)]`. Zero orphans allowed. (A4) |
| `scripts/test-path-a-e2e.sh` | The R0 rig: real agent binary + fake DeepSeek server + wire transcript capture. |
| `scripts/test-path-a-liveness.sh` | The anti-brick test. ≥3 edits, ≥2 files, exit 0. (A2) |
| `scripts/test-path-a-snippet-e2e.sh` | L1-45 negatives, from wire. |
| `scripts/test-path-a-prefix-golden.sh` | Two-turn byte equality + no-wall-clock negative. |
| `scripts/test-path-a-repair-e2e.sh` | One-repair-then-error. |
| `scripts/test-path-a-routing-e2e.sh` | Flash default / `/pro` one-turn / precedence. |
| `scripts/test-path-a-perms-matrix.sh` | Spec 90 matrix incl. subagent tools. |
| `scripts/lib/owner-bar-evidence.sh` | Evidence record schema writer: command + stdout + exit + SHA + CI run id. (A5) |

### rg call-site checks (CI, fail-closed)

```bash
# 1. The Standard guard must not exclude the product default.
rg -n 'effective != .*FileToolset::Standard' third_party/grok-build --glob '*.rs'
#    → MUST be 0 hits in non-test code after G004.

# 2. read_file must mint a version token.
rg -c 'file_version' third_party/grok-build/crates/codegen/xai-grok-tools/src/implementations/grok_build/read_file/
#    → MUST be >= 1 after G003. (Today: 0.)

# 3. Nominal-fraud check — no orphan Path-A-named symbols.
rg -n 'path_a_|PathA' crates third_party/grok-build --glob '*.rs'
#    → every hit needs a non-test call site outside its own file, else delete/rename.

# 4. Linkage: if a heart claims F1 via shared crate, the vendor must depend on it.
rg -n 'dsb-(tools|context|agent)' third_party/grok-build --glob 'Cargo.toml'
#    → MUST be >= 1, OR the port is in-tree and the ported symbol has a vendor call site.

# 5. Binary-level truth — the shipped artifact, not the source.
strings "$(command -v deepseek-build-agent)" | rg 'snippet_safe|file_version'
#    → MUST hit after G004.
```

### Forbidden evidence patterns (DENY list — any occurrence fails the row)

1. `cargo test -p dsb-tools …` as evidence for any **L1** row
2. `cargo test -p dsb-context …` as evidence for any **L2-10** row
3. `cargo test -p dsb-agent …` as evidence for any **L2-15 / L2-20** row
4. **`cargo test … path_a`** — a test-name filter is not a runtime path *(this is how 3.0.0 passed)*
5. `cargo test -p xai-grok-tools …` — a vendor unit test is not the composed binary
6. Any evidence record whose commands are **all** `cargo test`
7. `./scripts/test-l3-smoke.sh --offline-only` as sole evidence for any row requiring a wire assertion
8. Prose asserting a runtime default (*"the toolset injects X"*) with no attached stdout
9. *"documented in user-guide §N"* for any **L3** row
10. Screenshots as sole evidence for anything except **S5**
11. Call sites that exist only inside `#[cfg(test)]`
12. `BLOCKED — no API key` on **L1-45-0**, **L2-10-3**, or **L2-20-1**
13. Statuses `MET (verify on cut)` / `MET process` / any status outside the §2.3 legend
14. An evidence file with no CI run id, or dated the same day as the tag with no CI reference
15. Re-citing a **3.0.0- or 4.0.0-era** evidence file for a 5.0.0 row

---

## 4. PR DAG skeleton — `WAVE_5x_PR_DAG.md`

Evidence column is **mandatory and non-empty**; a unit with an empty or DENY-listed evidence cell is
not mergeable.

| Unit | Depends | Story | Band | **Evidence (mandatory)** |
|------|---------|-------|------|--------------------------|
| **5x-G0-1** | — | G001 | docs+ci | `test-owner-bar.sh` exit **1** + `OWNER_BAR_STATUS.tsv` (≥20 FAIL) + CI run id |
| **5x-G0-2** | 5x-G0-1 | G001 | ci | `check-forbidden-evidence.sh` + `check-path-a-linkage.sh` exit 0 on clean tree; orphan report for facts 6/7 |
| **5x-G0-3** | 5x-G0-1 | G001 | docs | README + KNOWN_LIMITS demote 3.0/4.0 claims per §8.2 (same PR as 5x-G0-1) |
| **5x-R0-1** | 5x-G0-2 | G002 | `5.0.0-alpha.1` | Agent pid + process tree + wire transcript of scripted 2-turn session |
| **5x-R0-2** | 5x-R0-1 | G002 | alpha | Fake-DeepSeek cassette fixtures committed; replay deterministic across 2 runs |
| **5x-L1-1** | 5x-R0-2 | G003 | alpha | Wire `read_file` result showing `file_version` == sha256; rg check 2 ≥ 1 |
| **5x-L1-2** | 5x-L1-1 | G004 | alpha | rg check 1 == 0 hits; `strings` check 5 hits; 3 negative transcripts |
| **5x-L1-3** | 5x-L1-2 | G004 | alpha | **`test-path-a-liveness.sh` exit 0** + resulting diff (≥3 edits / ≥2 files) |
| **5x-L1-4** | 5x-L1-3 | G005 | alpha | `write` version-check transcript; post-`bash` invalidation transcript |
| **5x-L1-5** | 5x-R0-2 | G006 | alpha | Perms matrix TSV incl. headless deny (exit≠0) and subagent-spawned tools |
| **5x-L2-1** | 5x-R0-2 | G008 | `5.0.0-beta.1` | Repair transcripts: 1 repair → success; 2nd malformation → structured error |
| **5x-L2-2** | 5x-L1-2, 5x-L2-1 | G007 | beta | Two-turn prefix hashes **equal**; no-wall-clock negative *(captured post-schema-change — A7)* |
| **5x-L2-3** | 5x-L2-2 | G007 | beta | Resume/compaction transcript; prefix hash preserved across resume |
| **5x-L2-4** | 5x-R0-2 | G009 | beta | Wire model per turn; `/pro` one-turn escalation + return; precedence table |
| **5x-L3-1** | 5x-L1-4, 5x-L1-5, 5x-L2-3, 5x-L2-4 | G010 | `5.0.0-rc.1` | G004–G009 assertions **re-run with parallel+subagents on**; zero regressions |
| **5x-L3-2** | 5x-L3-1 | G010 | rc | Worker prefix hash == parent; worker mutation invalidates parent versions |
| **5x-PKG-1** | 5x-G0-1 | G011 | any | Clean-container transcript: both CLIs open TUI; versions match |
| **5x-CUT-1** | 5x-L3-2, 5x-PKG-1 | G012 | rc | `test-owner-bar.sh` **exit 0**; full `OWNER_BAR_STATUS.tsv` all MET |
| **5x-CUT-2** | 5x-CUT-1 | G012 | rc | Adversarial review **#1** — non-authoring, own stdout for ≥3 re-run R0 commands |
| **5x-CUT-3** | 5x-CUT-1 | G012 | rc | Adversarial review **#2** — different model family, own stdout for ≥3 re-run R0 commands |
| **5x-CUT-4** | 5x-CUT-2, 5x-CUT-3 | G012 | **5.0.0** | §12 sign-off block filled; tag applied in a **separate commit after** all above |

---

## 5. Cut criteria for v5.0.0

### Single formula

```
CUT(5.0.0) = AND(
  test-owner-bar.sh                         → exit 0
  OWNER_BAR_STATUS.tsv                      → every P0 row == MET
  every MET row                             → evidence record with {command, verbatim stdout,
                                               exit code, git SHA, CI run id}
  every N/A row                             → listed in §9 with owner signature
  check-forbidden-evidence.sh               → exit 0 over docs/product/evidence/**
  check-path-a-linkage.sh                   → exit 0 (zero orphan path_a_* symbols)
  test-path-a-liveness.sh                   → exit 0 (≥3 edits, ≥2 files, no intervention)
  BLOCKED rows                              → ≤ 2, and none of {L1-45-0, L2-10-3, L2-20-1}
  adversarial reviews                       → 2 PASS, neither by the authoring session,
                                               ≥1 different model family,
                                               each re-ran ≥3 R0 commands with own stdout
  tag v5.0.0                                → applied in a separate commit AFTER all of the above
)
```

Any single clause false ⇒ **NO-GO**. There is no partial credit, no "story complete," and no
owner-override path that does not also edit §9 in the same PR.

### Dual adversarial review: **REQUIRED — and stricter than F5 says today**

Required, because the failure mode that produced 3.0.0 and 4.0.0 was not missing tests — it was
**an agent grading its own prose**. Three binding conditions:

1. **Non-authoring.** Neither reviewer may be the session that wrote the code or the evidence.
2. **Cross-family.** At least one reviewer from a different model family than the author.
3. **Reproduce, don't read.** Each reviewer independently re-runs **≥3 R0 commands** and attaches
   **their own stdout**. A review consisting of "read the evidence file, looks correct" is void.

Either FAIL blocks the tag. A reviewer who cannot reproduce a claimed R0 result files it as an
**integrity failure**, which reopens the story — it does not downgrade to PARTIAL.

---

## Closing position

The bar's diagnosis is right and its Path-A/Path-B distinction is the correct spine. But shipped as
written it would let a diligent agent produce a green 5.0.0 with a **product that cannot edit a
file** — via G-2, using rows that are all satisfied by rejection. Fix A1–A8, remove R1–R4, make G001
an executable that fails, and refuse to let any story close on a `cargo test` line.

**Under-claim:** until `test-path-a-liveness.sh` is green, DeepSeek Build has no verified edit path
on Path A at all — not a partial one. The honest 4.x claim is narrower than §8.2 currently allows.

```

## Codex gpt-5.6-sol (verdict sections)

```markdown
### 1. Verdict on OWNER_BAR_ACCEPTANCE.md

**NEEDS FIXES — reject it as the `5.0.0` release gate in its current form.**

Confidence: **high**. It is a strong postmortem and requirements inventory, but not yet an executable gate. That distinction matters: PRD-v3 already said hearts must control the default Grok path and still declared `3.0.0` cut ([PRD-v3.md](/Users/WooseongKim/Projects/OpenSources/deepseek-build/docs/product/PRD-v3.md:56)). PRD-v4 then inherited the unproved premise that heart fusion was complete ([PRD-v4.md](/Users/WooseongKim/Projects/OpenSources/deepseek-build/docs/product/PRD-v4.md:17)). More prose will not prevent a third false close.

Top five gaps:

1. **The gate is scheduled after the work it is supposed to govern.**

   The document says the Path A scripts should “evolve” or land before `5.0.0`, while W8—the gate script and review—is explicitly last ([OWNER_BAR_ACCEPTANCE.md](/Users/WooseongKim/Projects/OpenSources/deepseek-build/docs/product/OWNER_BAR_ACCEPTANCE.md:271), [OWNER_BAR_ACCEPTANCE.md](/Users/WooseongKim/Projects/OpenSources/deepseek-build/docs/product/OWNER_BAR_ACCEPTANCE.md:311)). Fresh tree inspection confirms `test-owner-bar.sh` and the four named heart scripts do not exist. No G00x story is mechanically prevented from closing today.

   Worse, the template calls `test-l3-smoke.sh --live`, but that script does not accept `--live` ([test-l3-smoke.sh](/Users/WooseongKim/Projects/OpenSources/deepseek-build/scripts/test-l3-smoke.sh:18)).

2. **The known dead wiring can still generate green-looking source and unit evidence.**

   `FileToolset::Standard` constructs `snippet_safe=true` ([config.rs](/Users/WooseongKim/Projects/OpenSources/deepseek-build/third_party/grok-build/crates/codegen/xai-grok-shell/src/tools/config.rs:351)), but production calls `tool_configs(...)` only when `effective != Standard` ([agent_ops.rs](/Users/WooseongKim/Projects/OpenSources/deepseek-build/third_party/grok-build/crates/codegen/xai-grok-shell/src/agent/mvp_agent/agent_ops.rs:4391)). The consumer defaults `snippet_safe=false` ([search_replace/mod.rs](/Users/WooseongKim/Projects/OpenSources/deepseek-build/third_party/grok-build/crates/codegen/xai-grok-tools/src/implementations/grok_build/search_replace/mod.rs:137)).

   The Standard unit test merely checks that three tool IDs exist; it does not prove the parameters reach the finalized production toolset ([config.rs](/Users/WooseongKim/Projects/OpenSources/deepseek-build/third_party/grok-build/crates/codegen/xai-grok-shell/src/tools/config.rs:438)). This is precisely how another false green can be manufactured.

3. **“Path A or equivalent process composition” is a provenance escape hatch.**

   R0 currently allows an “equivalent process composition” ([OWNER_BAR_ACCEPTANCE.md](/Users/WooseongKim/Projects/OpenSources/deepseek-build/docs/product/OWNER_BAR_ACCEPTANCE.md:62)). That permits direct `xai-grok-pager`, a hand-assembled shell, or a test-only agent definition instead of the shipped user entry.

   The real launcher accepts `DEEPSEEK_BUILD_AGENT_BIN` as its first candidate ([agent_launch.rs](/Users/WooseongKim/Projects/OpenSources/deepseek-build/crates/dsb-cli/src/agent_launch.rs:14)), while existing L3 smoke calls the resolved agent binary directly ([test-l3-smoke.sh](/Users/WooseongKim/Projects/OpenSources/deepseek-build/scripts/test-l3-smoke.sh:61)). Neither proves the public CLI, resolver, config repair, `GROK_HOME`, and installed agent as one composition. The public `agent` subcommand already provides the correct headless entry through `agent_launch` ([main.rs](/Users/WooseongKim/Projects/OpenSources/deepseek-build/crates/dsb-cli/src/main.rs:223)); use it.

4. **`SKIP`, `BLOCKED`, and credential absence can still become success.**

   The acceptance template permits `env-BLOCKED` evidence ([OWNER_BAR_ACCEPTANCE.md](/Users/WooseongKim/Projects/OpenSources/deepseek-build/docs/product/OWNER_BAR_ACCEPTANCE.md:271)). The existing L3 script records four runtime rows as `SKIP` when credentials are absent and can still reach “ALL PASSED” because only explicit failures set `FAILED` ([test-l3-smoke.sh](/Users/WooseongKim/Projects/OpenSources/deepseek-build/scripts/test-l3-smoke.sh:101), [test-l3-smoke.sh](/Users/WooseongKim/Projects/OpenSources/deepseek-build/scripts/test-l3-smoke.sh:198)).

   A product feature cannot be complete because nobody supplied an API key. Hermetic scripted-provider R0 must be mandatory; live DeepSeek R0 must additionally be mandatory for the final cut.

5. **The closure ledger is incomplete and internally inconsistent.**

   Explicit P0 requirements `L1-70`, `L1-30`, and `L1-100` exist ([OWNER_BAR_ACCEPTANCE.md](/Users/WooseongKim/Projects/OpenSources/deepseek-build/docs/product/OWNER_BAR_ACCEPTANCE.md:140)), but W0–W8 has no skills, effort, or session-resume workstream. W7 covers S1–S7 and silently omits S8. W6 omits L3-ID-1. F5 describes one “second model or human” review, while the sign-off block demands two reviews ([OWNER_BAR_ACCEPTANCE.md](/Users/WooseongKim/Projects/OpenSources/deepseek-build/docs/product/OWNER_BAR_ACCEPTANCE.md:208), [OWNER_BAR_ACCEPTANCE.md](/Users/WooseongKim/Projects/OpenSources/deepseek-build/docs/product/OWNER_BAR_ACCEPTANCE.md:331)).

   The P0 set is also not machine-enumerated, inherited `MET` statuses remain visible, and cut-time `N/A` waivers are permitted. That is an invitation to close the rows that have stories and quietly ignore the rest.

Must-add P0 items:

- **OB-0 — Frozen machine ledger:** one manifest enumerating every `5.0.0` P0 ID, owning G00x story, required R0/R1 cases, exact SHA, binary hash, command, result, and artifact hash.
- **OB-1 — Public-entry provenance:** R0 must enter through installed `deepseek-build agent -- …` and `dsb agent -- …`, with `DEEPSEEK_BUILD_AGENT_BIN` unset and the resolved child path/hash recorded.
- **OB-2 — No-skip contract:** every mandatory row must be `PASS`; `SKIP`, `BLOCKED`, `N/A`, `NOT_RUN`, ignored, or expected-failure is a hard failure.
- **OB-3 — Freshness:** no carried-forward `MET`; every row is rerun against the exact release-candidate SHA and installed binary bundle.
- **OB-4 — Gate self-test:** the gate must prove it rejects R2-only evidence, direct-agent evidence, mismatched SHAs, skipped rows, stale artifacts, and duplicate/self reviews.
- **OB-5 — Complete workstream mapping:** explicitly cover S1–S8, L1-45, L1-90, L1-70, L1-30, L1-100, L2-10/15/20, all L3-50/60/WT/ID rows, and F1–F5.

Must-remove from P0 closure:

- Cut-time `N/A` waivers. Any scope demotion must happen before G001 freezes the train, not during cut.
- “Equivalent process composition” as final evidence.
- `env-BLOCKED`, `SKIP`, or missing credentials as acceptable cut results.
- Inherited baseline `MET` statuses as evidence.
- W8-last ordering.
- Manual sign-off text not cryptographically bound to one SHA, one binary manifest, and one evidence bundle.

---

### 2. Recommended 5.0.0 train shape

**Plan ID:** `owner-bar-5x`  
**Story count:** **11 stories, G001–G011**

The first story must be **honesty**, not product feature code. The repository’s SSOT now admits `3.0.0` and `4.0.0` were attempts rather than owner-bar completion ([SSOT.md](/Users/WooseongKim/Projects/OpenSources/deepseek-build/docs/product/SSOT.md:38), [versions/README.md](/Users/WooseongKim/Projects/OpenSources/deepseek-build/docs/product/versions/README.md:15)). That truth must be frozen into PRD-v5, the board, and an executable red baseline before anyone edits a heart.

| Story | Scope | Path-A-only Done-when |
|---|---|---|
| **G001 TruthReset** | PRD-v5, frozen P0 ledger, gate substrate, red baseline | Both installed public commands traverse `agent_launch`; a machine manifest enumerates every P0 and records current Path A failures. Gate self-tests reject fake green evidence. No product feature code. |
| **G002 SnippetAuthority** | L1-45-1…8, F2 | Public Path A `read_file` emits a usable version token; valid edit succeeds; missing/stale token, ambiguous match, empty-old overwrite, unsafe `write`, bash mutation, and worker mutation cases fail closed or invalidate correctly. |
| **G003 PermissionAuthority** | L1-90-1…5, S8 | Public Path A proves allow/deny/ask, headless Ask→deny, workspace boundaries, default non-YOLO, and no parallel/subagent bypass through runtime permission traces. |
| **G004 PrefixAuthority** | L2-10-1…6, F1 | Scripted provider captures actual main-agent request bytes over multiple turns; declared prefix is ordered and byte-identical, volatile data stays in tail, and the controlling call site is in the shipped agent graph. |
| **G005 SkillsResume** | L1-70, L1-100, L2-10-5 | Public Path A proves stable skills index, body-on-demand without prefix mutation, session resume, compaction, and repaired replay while retaining the cache epoch contract. |
| **G006 RepairAuthority** | L2-15-1…4, F1 | Scripted malformed calls reach the real Path A dispatcher: exactly one repair, no invented required args, no tool rename, structured failure after one attempt, and repaired replay pairing. |
| **G007 RoutingEffort** | L2-20-1…5, L1-30 | Captured Path A requests prove Flash default, one-turn Pro escalation and return, precedence, correct wire model/base URL, visible turn model, and user-controlled effort. |
| **G008 ParallelBackground** | L3-50-1…4 | Public Path A proves concurrent independent reads, serialized mutations, fail-closed unknown/bash/MCP scheduling, and background collect-by-ID while rerunning all L1/L2 regressions. |
| **G009 SubagentWorktree** | L3-60-1…4, L3-WT-1/2, L3-ID-1 | Default Path A spawns explore/implement workers; parent/worker prefix hashes obey the cache law; worker writes invalidate parent snippets; worktree flow runs; user claims match observed behavior. |
| **G010 InstalledProduct** | S1–S7 plus complete identity regression | A clean primary-platform install contains CLI and agent; both command names open the same TUI, resolve the packaged agent without override, use product home/config/theme, and pass the complete Path A suite. |
| **G011 FreezeReviewCut** | F1–F5, all P0, tag | One frozen SHA and binary manifest passes the full offline and live owner-bar suite with zero non-PASS rows; two independent adversarial reviews approve that exact bundle; only then tag `v5.0.0`. |

Workstream ordering:

```text
G001 Truth + gate freeze
  ├─ G002 → G003                 L1 edit/permission lane
  ├─ G004 → G005                 prefix/skills/resume lane
  ├─ G006                        dispatch repair lane
  └─ G007 (after G004 tracing)   routing/effort lane
          ↓ all four lanes green
        G008 → G009              L3 only after hearts
          ↓
        G010                     installed product
          ↓
        G011                     frozen reviews and cut
```

G002/G003, G004/G005, G006, and G007 may run as parallel workstreams after G001. G008 must wait for all heart lanes. G009 must wait for G008 because worker safety depends on the scheduler, stable prefix, permissions, and snippet invalidation. Packaging and cut remain sequential.

---

### 3. Mechanical gates that must exist before any story can complete

Fresh inspection: only `scripts/test-l3-smoke.sh` currently exists from the proposed owner-bar command set. The following are **required additions**, not claims about current files:

| Required script | Purpose |
|---|---|
| `scripts/test-owner-bar.sh` | Sole aggregator; validates ledger coverage, provenance, SHA freshness, no-skip status, reviews, and mapped regressions. |
| `scripts/test-path-a-public-entry-e2e.sh` | Runs installed `deepseek-build agent -- …` and `dsb agent -- …`; rejects raw-agent-only execution and overrides. |
| `scripts/test-path-a-snippet-e2e.sh` | Read token plus valid/missing/stale/ambiguous/overwrite/invalidation matrix. |
| `scripts/test-path-a-permissions-e2e.sh` | TTY/headless, workspace boundary, parallel, and subagent permission matrix. |
| `scripts/test-path-a-prefix-golden.sh` | Captures actual provider request bytes, prefix epochs, resume and worker hashes. |
| `scripts/test-path-a-repair-e2e.sh` | Scripted malformed calls through the production dispatcher. |
| `scripts/test-path-a-routing-e2e.sh` | Flash/Pro/effort request-sequence assertions. |
| `scripts/test-path-a-l3-e2e.sh` | Deterministic parallel, background, subagent and worktree scenarios under heart regressions. |
| `scripts/test-path-a-install-e2e.sh` | Clean-package install, dual command, resolver and binary provenance. |

Required e2e harness identities:

- **`owner_bar_path_a`** — black-box public-entry process driver.
- **`scripted_deepseek_server`** — deterministic Chat Completions fixture server capable of emitting exact tool calls and recording actual requests.
- **`path_a_trace_assert`** — validates tool parameters, scheduler decisions, permissions, model routing, prefix hashes, child process provenance, and mutation effects.
- **`owner_bar_gate_selftest`** — supplies intentionally fraudulent evidence and requires the aggregator to reject it.

G001 may complete while runtime rows are red only if `test-owner-bar.sh --baseline-red` proves **100% P0 enumeration** and correctly reports those rows as failures. After G001, every story requires:

```text
mapped new cases = PASS
all previously green cases = PASS
public Path A provenance = PASS
manifest coverage = 100%
SKIP/BLOCKED/N/A/NOT_RUN = 0
```

Mandatory `rg` tripwires:

```bash
# Current dead Standard exclusion must be gone.
! rg -n 'effective\s*!=\s+.*FileToolset::Standard' \
  third_party/grok-build/crates/codegen/xai-grok-shell/src/agent/mvp_agent/agent_ops.rs

# Production agent assembly must apply file tool configs.
rg -n '\.tool_configs\(' \
  third_party/grok-build/crates/codegen/xai-grok-shell/src/agent/mvp_agent/agent_ops.rs

# Read-side version issuance must exist, not only edit-side consumption.
rg -n 'file_version|snippet_id' \
  third_party/grok-build/crates/codegen/xai-grok-tools/src/implementations/grok_build/read_file \
  third_party/grok-build/crates/codegen/xai-grok-tools/src/types/output.rs

# Each thin helper must have a production Path A call site or be explicitly demoted.
rg -n 'assemble_path_a_context|prepare_path_a_tool_call|path_a_default_router' \
  third_party/grok-build crates --glob '*.rs'

# Link evidence where shared dsb heart crates are claimed.
rg -n 'dsb[_-](agent|context|tools)' \
  third_party/grok-build --glob 'Cargo.toml' --glob '*.rs'

# Evidence manifests may not contain a non-pass escape.
! rg -n '"status"\s*:\s*"(SKIP|BLOCKED|N/A|NOT_RUN|XFAIL|IGNORED)"' \
  docs/product/evidence/OWNER_BAR_5_0_0
```

These scans are tripwires, not completion evidence. The current code proves why: `snippet_safe` is present in source while absent from the default runtime.

The read-side hole is equally concrete: `FileContent` has no version field ([output.rs](/Users/WooseongKim/Projects/OpenSources/deepseek-build/third_party/grok-build/crates/codegen/xai-grok-tools/src/types/output.rs:208)), and `read_file` returns that structure without minting a token ([read_file/mod.rs](/Users/WooseongKim/Projects/OpenSources/deepseek-build/third_party/grok-build/crates/codegen/xai-grok-tools/src/implementations/grok_build/read_file/mod.rs:567)). Edit-side enforcement alone would make the agent unable to perform a valid safe edit.

Forbidden evidence patterns:

- `cargo test -p dsb-agent`, `dsb-context`, or `dsb-tools` as closure evidence.
- Direct `deepseek-build-agent` or `xai-grok-pager` invocation as sole R0.
- Any run with `DEEPSEEK_BUILD_AGENT_BIN` set for release evidence.
- A test-only agent definition manually calling `override_file_tools`.
- `rg snippet_safe`, schema snapshots, config seed tests, or call-graph prose without behavioral R0.
- `--help` flag presence as proof a feature works.
- Natural-language “the model said it worked” without captured tool and scheduler traces.
- Permission or mutation evidence run only under `--yolo`.
- `SKIP`, `BLOCKED`, `N/A`, ignored tests, expected failures, or “key unavailable.”
- Evidence generated from a different Git SHA, binary hash, platform package, or configuration than the release candidate.
- Historical `3.0.0`/`4.0.0` evidence, tags, PR merges, ultragoal completion, board status, docs, screenshots, or manual checkboxes.
- Current `test-l3-smoke.sh` success while any row is `SKIP`.
- Two review records produced by the same authoring context, same reviewer identity, or against different SHAs.

---

### 4. PR DAG skeleton

Every PR body must contain its exact evidence artifact path and candidate SHA. “Tests pass” is not an evidence column.

| Unit | Story | Depends on | Mandatory evidence |
|---|---|---|---|
| **5x-H0-1** | G001 truth reset | — | Public-entry red baseline for both CLI names; complete frozen P0 manifest; PRD-v5 honesty table. |
| **5x-H0-2** | G001 gate substrate | 5x-H0-1 | Gate self-test rejecting R2-only, direct-agent, stale-SHA, skip, override, and duplicate-review fixtures. |
| **5x-H1-1** | G002 snippet authority | 5x-H0-2 | Path A tool trace plus filesystem before/after hashes for every L1-45 positive and negative case. |
| **5x-H1-2** | G003 permission authority | 5x-H1-1 | Path A permission-decision trace for TTY/headless, boundary, parallel and worker cases; non-YOLO provenance. |
| **5x-H2-1** | G004 prefix authority | 5x-H0-2 | Actual provider request bytes, prefix boundaries, two-turn hashes, epoch transition, and production call-site/link evidence. |
| **5x-H2-2** | G005 skills/resume | 5x-H2-1 | Skills-index/body traces, resume/compaction request hashes, session artifact, repaired replay record. |
| **5x-H2-3** | G006 repair authority | 5x-H0-2 | Raw malformed provider payload, repaired dispatch payload, repair count, negative no-invention results. |
| **5x-H2-4** | G007 routing/effort | 5x-H2-1 | Ordered captured requests showing Flash→Pro→Flash, precedence, effort, model visibility, and base URLs. |
| **5x-H3-1** | G008 parallel/background | 5x-H1-2, 5x-H2-2, 5x-H2-3, 5x-H2-4 | Scheduler timestamps/IDs proving read concurrency, mutation serialization, fail-closed classification, background collection, plus full heart regression. |
| **5x-H3-2** | G009 subagent/worktree | 5x-H3-1 | Parent/worker prefix hashes, worker type/capability trace, snippet invalidation record, worktree filesystem proof, claim-honesty scan. |
| **5x-H4-1** | G010 installed product | 5x-H3-2 | Clean install log, package inventory, both CLI traces, resolved agent path and SHA-256, TUI/PTY smoke, full owner-bar suite. |
| **5x-H5-1** | G011 review/cut | 5x-H4-1 | Immutable all-PASS manifest, live DeepSeek Path A log, R1 graph, two independent review artifacts bound to the same SHA/binary manifest, tag-after-gate proof. |

No unit may write “Depends: none” merely because it is opened in parallel. The DAG above is the minimum dependency truth.

---

### 5. Cut criteria for v5.0.0

Define `R0A(p)` as evidence produced through—or mechanically bound to—the installed public Path A composition, with resolved agent hash and release SHA recorded.

The single cut formula is:

```text
CUT(v5.0.0) =
  Frozen(P0_5x, GitSHA, BinaryManifest)
  ∧ ∀p∈P0_5x:
      PASS(p) ∧ R0A(p) ∧ Fresh(p, GitSHA, BinaryManifest)
      ∧ (CodeBound(p) ⇒ R1(p))
      ∧ status(p)∉{SKIP,BLOCKED,N/A,NOT_RUN,XFAIL,IGNORED}
  ∧ F1 ∧ F2 ∧ F3 ∧ F4 ∧ F5
  ∧ PublicEntry(deepseek-build) ∧ PublicEntry(dsb)
  ∧ OfflineHermeticR0 ∧ LiveDeepSeekR0
  ∧ CleanInstallPrimaryPlatform
  ∧ DocsSingular
  ∧ ReviewA(SHA,Manifest) ∧ ReviewB(SHA,Manifest)
  ∧ Independent(A,B)
  ∧ TagCreatedAfterAllTerms
```

**Dual adversarial review: required. Non-negotiable.**

- Reviewer A attacks runtime provenance, call sites, trace integrity, binary/SHA binding, and test adequacy.
- Reviewer B attacks scope omissions, waivers, documentation contradictions, inherited evidence, and gate-gaming opportunities.
- Neither may be the implementation author or release integrator.
- Both review the exact same frozen SHA and binary/evidence manifest.
- Any post-review code, config, evidence, or packaging change invalidates both approvals.
- Any reviewer disagreement is **NO-GO** until resolved and both reviews rerun.

That is the owner bar. Anything softer is another `3.0.0`/`4.0.0` narrative cut.

```
