# VC007 — Spec 10 assembly on Grok Path A turns

| Field | Value |
|-------|--------|
| **Story** | **VC007** — Grok Path A message assembly uses Spec 10 stable prefix layout on turns |
| **Plan** | `vision-complete-5x` |
| **Date** | 2026-08-08 |
| **Status** | **PLAN** — implementation + evidence pending; **unversioned** |
| **SemVer** | **none** (this story does **not** bump product version) |
| **Depends on** | **VC006** Spec 45 Deep Code cut on stack (open PR **#138** `vc006-heart-r0a`) |
| **Board** | [`VISION_COMPLETE_5X_GOALS.md`](../VISION_COMPLETE_5X_GOALS.md) · DAG [`WAVE_5x_VISION_PR_DAG.md`](../WAVE_5x_VISION_PR_DAG.md) |
| **Normative** | [`docs/specs/10-cache-contract.md`](../../specs/10-cache-contract.md) · [`HARNESS_PHILOSOPHY.md`](../../architecture/HARNESS_PHILOSOPHY.md) §4.2 / §5 · [`HEART_3X_SPEC_BINDING.md`](../../architecture/HEART_3X_SPEC_BINDING.md) §3.4 |
| **Prior evidence** | G008 launch stamp + multi-turn wire ([`G008_PREFIX_SKILLS_RESUME_2026-08-07.md`](./G008_PREFIX_SKILLS_RESUME_2026-08-07.md)); H10 library ([`H10_PATH_A_PREFIX_2026-08-07.md`](./H10_PATH_A_PREFIX_2026-08-07.md)); VC006 stack honesty ([`VC006_PATH_A_HEART_R0A_2026-08-08.md`](./VC006_PATH_A_HEART_R0A_2026-08-08.md)) |

**This file is the mandatory ultragoal PR unit plan for VC007 plus implementation evidence.**
It does **not** claim VISION L2 Reasonix complete (that still needs VC008 effort-on-wire + VC009 cache visibility / cut). Thin Path B / library-only greens are **not** Path A turn proof.

---

## 0. Floor and dependency facts

### 0.1 Live floor (story open; 2026-08-08)

| Probe | Live result |
|-------|-------------|
| This worktree branch | `vc007-context-assembly` (forked at VC006 tip) |
| Stack base for feature commits / PR base | **`vc006-heart-r0a`** (open PR **#138**); **not** `origin/main` until after #138 merges |
| `git show origin/main:Cargo.toml` version | **`5.2.2`** |
| `npm view @innocarpe/deepseek-build version` | **`5.2.2`** |
| `gh release list` Latest | **`v5.2.2`** |
| Working tree product version (stack tip) | **`5.3.0`** (from VC006 cut on branch) |
| Board text residual | May still map Reasonix cut (VC009) to **`5.3.0`** — **stale vs live floor + VC006 already on 5.3.0** |
| Next free Reasonix cut minor residual | **`5.4.0`** unless a later floor re-check moves it |
| Thin Path B | `crates/dsb-context` / `dsb-agent` prefix builders remain **reference/oracle**, not Path A turn proof alone |

### 0.2 Floor interpretation (fail-close)

- **Live product floor is `origin/main` = `5.2.2`.** npm and GitHub Release **`v5.2.2`** are aligned at open.
- Stack already carries product **`5.3.0`** from VC006. **Do not reuse or re-cut `5.3.0`.**
- VC007 is **unversioned** — no SemVer bump, no npm, no GitHub Release packaging.
- Historical board mapping of Reasonix cut (VC009) → **`5.3.0`** is **stale**. Under current floor + VC006 cut, Reasonix cut residual is next free minor **`5.4.0`** unless floor re-check moves.
- **Open as a stacked PR** with base **`vc006-heart-r0a`** and body **`Depends on #138`**. **Do not merge** this PR in-story. Rebase / retarget after #138 merges.
- Fail closed if public Path A runtime cannot provide required wire/turn proof — record residual; do not invent green.

---

## 1. Why this PR (one sentence)

Close the Reasonix L2 gap where **Grok Path A turn message assembly** still bypasses Spec 10 stable-prefix discipline: launch-time `assemble_path_a_context` stamp (G008) proves library linkage, but **turns** must assemble (or mirror) Spec 10 ordered stable prefix + volatile tail with epoch isolation.

---

## 2. Call-path map (inspected before design)

| Layer | Path | Role today (pre-VC007) |
|-------|------|------------------------|
| Public product | `deepseek-build` / `dsb` → `deepseek-build-agent` (vendored Grok) | **Path A** product default |
| Product library Spec 10 | `crates/dsb-context` `assemble_path_a_context` / `PrefixBuilder` | Spec 10 layout + epoch; H10 unit greens |
| Launch stamp (G008) | `dsb-cli` `stamp_path_a_prefix_epoch` → `assemble_path_a_context` | **Once** before agent exec; writes `path_a_prefix_epoch.txt` |
| Thin Path B agent | `dsb-agent` loop `PrefixBuilder` + `assemble_messages` per turn | Correct Spec 10 spirit — **not** public Path A |
| Grok turn assembly | `xai-chat-state` `build_conversation_request` via shell `turn.rs` | Real Path A messages; Grok system + API tools + volatile tail |
| Grok Spec 15 pattern | `session/helpers/tool_input_parsing.rs` + tool dispatch call site | Port-without-dsb-dep heart fusion model to copy |
| Historical wire (G008) | multi-turn system stability analyzer | System stable across turns; **does not prove** Spec 10 library layout path on every turn |
| Dual CLI | `deepseek-build` / `dsb` | Must both keep working |

### Residual closed by this story

| ID | Gap | Close with |
|----|-----|------------|
| **V2-10-1** | Grok message assembly uses Spec 10 stable prefix layout | Grok-side Spec 10 assembly mirror + **turn** production call site |
| **V2-10-2** (partial) | Compaction/resume does not thrash stable prefix | Two-turn (or multi-turn) epoch isolation + existing resume honesty; full TUI compaction still Grok-owned residual |
| Board residual | `shell ≠ always assemble_path_a_context` | Turn-path assembly (or byte-equivalent mirror) on Path A |

### Target VC007 turn contract (Path A)

1. **Spec 10 layout** (stable prefix ordered sections): system body → tools document (canonical JSON / sorted keys) → skills index → environment (no wall-clock) → project instructions.
2. **Volatile tail** does not change `stable_prefix_bytes` / epoch.
3. **Production call site on Grok turn path** (not only `agent_launch` stamp; not only `#[cfg(test)]`).
4. **Unit goldens** for layout order, stability, tool-key sort, no timestamp in stable sections.
5. **Public Path A multi-turn wire** (or honest residual) showing system/stable head stability across ≥2 DeepSeek turns; thin oracle alone is insufficient for the Path A claim.
6. Gates stay green: owner-bar · path-a linkage · heart regression. Restore generated TSV side-effects to HEAD.

---

## 3. PR units (ordered atomic)

### PR unit 1 — `docs(product): VC007 Spec 10 Path A assembly plan + floor`

- **Intent:** English ultragoal unit plan with stack base, SemVer non-claim, acceptance matrix
- **Touches:** `docs/product/evidence/VC007_SPEC10_ASSEMBLY_PATH_A_2026-08-08.md`
- **Depends on:** none (first commit on branch)
- **SemVer:** none
- **Tests:** n/a (docs)

### PR unit 2 — `feat(context): Spec 10 assembly mirror on Grok Path A turns`

- **Intent:** Port Spec 10 stable-prefix assembly (layout + epoch + volatile isolation) into Grok shell helpers and invoke from the Path A turn request path
- **Touches:** `third_party/grok-build/.../session/helpers/spec10_path_a_assembly.rs` (new); `helpers/mod.rs`; turn / session call site(s); optional product stamp honesty if needed
- **Depends on:** unit 1
- **SemVer:** none
- **Tests:** `cargo test` scoped to new module + shell package filters; prove non-test production call site for linkage

### PR unit 3 — `test(context): Path A Spec 10 multi-turn / epoch evidence`

- **Intent:** Wire analyzer and/or Path A R0A harness proof that turn assembly keeps Spec 10 stable head across turns; record evidence under `docs/product/evidence/`
- **Touches:** `scripts/lib/analyze_path_a_prefix_wire.py` (if extended); evidence artifacts; this evidence file status update
- **Depends on:** unit 2
- **SemVer:** none
- **Tests:** analyzer + public-entry / R0A harness as applicable; three product gates

## Sequential

1. unit 1 → unit 2 (docs plan before code)
2. unit 2 → unit 3 (behavior before evidence)

## Parallel

- None (single stack lane; avoid parallel edits to shell turn path)

---

## 4. Acceptance matrix

| ID | Criterion | Evidence |
|----|-----------|----------|
| **VC007-A1** | Spec 10 ordered stable sections in assembly API | Unit golden (section markers / order) |
| **VC007-A2** | Identical stable inputs → equal `stable_prefix_bytes` / epoch | Unit (two builds) |
| **VC007-A3** | Volatile tail growth does not change epoch | Unit |
| **VC007-A4** | Tool schema object key permutation → same canonical tools document | Unit |
| **VC007-A5** | No wall-clock / `SystemTime` / random UUID in stable sections | Unit negative |
| **VC007-A6** | Production **turn** call site (non-test) on Grok Path A | `rg` + linkage / review of turn path |
| **VC007-A7** | Multi-turn Path A wire: system/stable head stable; volatile grows | Analyzer on wire JSONL or honest residual |
| **VC007-A8** | Gates green | `./scripts/test-owner-bar.sh` · `./scripts/check-path-a-linkage.sh` · `./scripts/test-heart-regression.sh` |
| **VC007-A9** | Unversioned; no SemVer / npm / Release packaging | Diff does not bump version |
| **VC007-A10** | Stacked PR base `vc006-heart-r0a` + body `Depends on #138` | `gh pr view` |

---

## 5. Security / cache boundaries

| Boundary | Rule |
|----------|------|
| Stable prefix | No wall-clock, no random IDs, no hostnames unless user opts in (Spec 10 §1.1–1.3) |
| Skills | **Index only** in stable sections; skill **bodies** stay on-demand / volatile (Spec 70) |
| Snippet table | Session state; **never** serialized into Spec 10 stable prefix (Spec 45) |
| Secrets | Do not put API keys, tokens, or credentials into stable prefix documents |
| Compaction | Drop oldest **volatile** only; never mutate stable prefix in place without epoch bump (Spec 10 §1.7) |
| Cache evidence | Epoch / prefix hash logging is allowed; provider cache-hit visibility is **VC009** |

---

## 6. Explicit non-claims / non-scope (fail-close)

| Out of scope | Why |
|--------------|-----|
| VC008 `reasoning_effort` / effort-on-wire | Separate story |
| VC009 cache-hit visibility / Reasonix packaging cut | Separate; residual cut **5.4.0** under current floor |
| VC010+ L3 dogfood | Track C |
| SemVer bump / release cut / npm / GitHub Release | VC007 unversioned; do not claim packaging |
| Reusing **5.3.0** for this story | Already used by VC006 on stack |
| Thin Path B alone as Path A proof | Library/oracle greens ≠ public agent turn proof |
| Full Cargo fusion of `dsb-context` into Grok (F1) | May remain residual; Grok-side mirror is valid heart fusion (Spec 15 pattern) |
| Claiming VISION L2 complete | Needs VC008 + VC009 + residual honesty |

---

## 7. Implementation notes (design sketch)

### 7.1 Pattern

Mirror Spec 15 fusion: pure Spec 10 assembly helpers under `xai-grok-shell` session helpers (no `dsb-*` Cargo dependency required), with a **non-test** call site on the Grok turn path that builds / validates Spec 10 stable prefix inputs for each main DeepSeek sample.

Product library `assemble_path_a_context` remains SSOT for thin Path B and launch stamp; Grok mirror must be **layout-compatible** (same section order and canonicalization rules).

### 7.2 Turn call site (target)

After Path A `build_request` (shell `turn.rs` / equivalent), compute Spec 10 assembly from:

- leading system prompt text (clock-free sections only for epoch inputs)
- tool definitions (canonical sorted-key JSON document)
- skills **index** (name + one-line description; sorted by name)
- environment summary (OS family + normalized cwd; **no** date/time)
- standing project instructions (discovered; deterministic order)

Volatile conversation tail is excluded from epoch. Best-effort stamp / log `prefix_epoch=` for evidence; failures must not break the turn.

### 7.3 Proof ladder

1. Unit goldens in Grok helper module  
2. Production call site grep / linkage  
3. Multi-turn wire analyzer (extend G008 analyzer if needed for Spec 10 section markers)  
4. Three product gates  

---

## 8. Stack / PR shape

```text
origin/main @ 5.2.2
    └── … stacked Deep Code (VC005 #137, …)
            └── vc006-heart-r0a  (PR #138, product 5.3.0)   ← PR base
                    └── vc007-context-assembly  (this story; unversioned)
```

- **PR title:** `feat(context): Spec 10 assembly on Grok Path A turns`
- **Base:** `vc006-heart-r0a`
- **Body must include:** `Depends on #138`
- **Labels:** existing repo labels (`feat` / kind + any required)
- **Merge:** **do not merge** in this story
- **Public text:** English only (GitHub gate)

---

## 9. Implementation log (filled as units land)

| Unit | Commit | Result |
|------|--------|--------|
| 1 docs plan | *(this file first)* | |
| 2 feat assembly + turn wire | | |
| 3 test/evidence + gates | | |

### Gate results (fill at READY)

| Gate | Result | Notes |
|------|--------|-------|
| `./scripts/check-path-a-linkage.sh` | | |
| `./scripts/test-owner-bar.sh` | | |
| `./scripts/test-heart-regression.sh` | | restore TSV to HEAD |

### Residuals after READY

- VC008 effort-on-wire  
- VC009 cache-hit visibility + Reasonix cut packaging (**5.4.0** residual under current floor)  
- Full `dsb-context` Cargo embed into Grok agent binary (F1) if still open  
- Deep Grok compaction byte-identity with library assembly (honest residual if not claimed)

---

## 10. READY checklist

- [ ] Unit 1 committed first  
- [ ] Spec 10 turn assembly implemented with units green  
- [ ] Path A multi-turn proof or documented residual  
- [ ] Three gates green; TSV side-effects not committed  
- [ ] Independent Grok adversarial review  
- [ ] Clean worktree (no accidental version bump / package-lock noise)  
- [ ] Stacked PR open: base `vc006-heart-r0a`, `Depends on #138`, English gate, labels  
- [ ] **Do not merge**  
