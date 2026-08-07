# VC007 — Spec 10 assembly on Grok Path A turns

| Field | Value |
|-------|--------|
| **Story** | **VC007** — Grok Path A message assembly uses Spec 10 stable prefix layout on turns |
| **Plan** | `vision-complete-5x` |
| **Date** | 2026-08-08 |
| **Status** | **READY** — wire mutation + gates re-green; adversarial round 2 **READY** with residuals; **unversioned** |
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

Close the Reasonix L2 gap where **Grok Path A turn message assembly** still bypassed Spec 10 stable-prefix discipline: launch-time `assemble_path_a_context` stamp (G008) proves library linkage, but **turns** must assemble Spec 10 ordered stable prefix into the Path A request system message with volatile tail isolation and epoch stamping.

---

## 2. Call-path map (inspected before design)

| Layer | Path | Role today (pre-VC007) |
|-------|------|------------------------|
| Public product | `deepseek-build` / `dsb` → `deepseek-build-agent` (vendored Grok) | **Path A** product default |
| Product library Spec 10 | `crates/dsb-context` `assemble_path_a_context` / `PrefixBuilder` | Spec 10 layout + epoch; H10 unit greens |
| Launch stamp (G008) | `dsb-cli` `stamp_path_a_prefix_epoch` → `assemble_path_a_context` | **Once** before agent exec; writes `path_a_prefix_epoch.txt` |
| Thin Path B agent | `dsb-agent` loop `PrefixBuilder` + `assemble_messages` per turn | Correct Spec 10 spirit — **not** public Path A |
| Grok turn assembly | `xai-chat-state` `build_conversation_request` via shell `turn.rs` | Real Path A messages |
| Grok Spec 15 pattern | `session/helpers/tool_input_parsing.rs` + tool dispatch call site | Port-without-dsb-dep heart fusion model to copy |

### Target VC007 turn contract (Path A) — closed

1. **Spec 10 layout** rewritten onto leading **system** message on each main turn: base system → Tools (canonical JSON) → Skills index → Environment → Project instructions.
2. **Volatile** conversation items remain outside the stable body; epoch ignores volatile count.
3. **Production call site** on Grok turn path after `build_request` via `apply_spec10_to_conversation_request` (mutates wire).
4. **Live discovery** of skills index + project instructions from workspace (not empty hard-codes).
5. **Unit goldens** for layout, stability, tool-key sort, wall-clock absence, wire rewrite, discovery, idempotent re-apply.
6. Gates green; TSV side-effects restored.

---

## 3. PR units (ordered atomic)

### PR unit 1 — `docs(product): VC007 Spec 10 Path A assembly plan + floor`

- **Commit:** `e315c76`
- **SemVer:** none

### PR unit 2 — `feat(context): Spec 10 assembly on Grok Path A turns`

- **Commit:** `154bb8e` (initial helper + turn hook)
- **SemVer:** none

### PR unit 3 — `test(scripts): soft-check VC007 Path A turn prefix epoch stamp`

- **Commit:** `2c7fe52`
- **SemVer:** none

### PR unit 4 — `feat(context): rewrite Path A wire system to Spec 10 layout`

- **Commit:** `5199d6d` — adversarial fix: mutate wire system; discover skills/project
- **SemVer:** none

## Sequential

1. unit 1 → unit 2 → unit 3 → unit 4 (docs → feat → harness → wire-honest deepen)

---

## 4. Acceptance matrix

| ID | Criterion | Result |
|----|-----------|--------|
| **VC007-A1** | Spec 10 ordered stable sections | **PASS** unit `vc007_layout_order_sections` + wire rewrite unit |
| **VC007-A2** | Identical stable inputs → equal epoch | **PASS** unit |
| **VC007-A3** | Volatile tail does not change epoch | **PASS** unit |
| **VC007-A4** | Tool schema key permutation stable | **PASS** unit |
| **VC007-A5** | No wall-clock in stable body | **PASS** unit (fixture); production hashes Grok base system as-is |
| **VC007-A6** | Production turn call site mutates wire | **PASS** — see §7.1 wire-mutation proof |
| **VC007-A7** | Multi-turn stability | **RESIDUAL** — unit idempotent re-apply same epoch only; G008 historical multi-turn reaffirm is **not** post-VC007 Spec 10 section wire; live multi-turn JSONL needs rebuilt agent |
| **VC007-A8** | Gates green | **PASS** re-run 2026-08-08 (this session): linkage + owner-bar 60/60 + heart regression; TSV restored |
| **VC007-A9** | Unversioned | **PASS** product remains **5.3.0** from VC006; no version commits in this story |
| **VC007-A10** | Stacked PR | **OPEN** #139 base `vc006-heart-r0a` — refresh only after READY; **do not merge** |

---

## 5. Security / cache boundaries

| Boundary | Rule |
|----------|------|
| Stable prefix | Env = os_family + cwd only (no date); skills index only (no bodies) |
| Skills bodies | On-demand / volatile — not in Spec 10 stable sections |
| Snippet table | Session state; never in stable prefix |
| Secrets | No API keys/tokens in stable prefix documents |
| Wire tools | API `tools[]` retained; Spec 10 also embeds canonical tools document in system (cache-first layout) |
| Compaction | Grok compaction ownership residual; do not thrash stable sections without epoch bump |

---

## 6. Explicit non-claims / non-scope (fail-close)

| Out of scope | Why |
|--------------|-----|
| VC008 `reasoning_effort` / effort-on-wire | Separate story |
| VC009 cache-hit visibility / Reasonix packaging cut | Residual cut **5.4.0** under current floor |
| VC010+ L3 dogfood | Track C |
| SemVer bump / release cut / npm / GitHub Release | Unversioned |
| Reusing **5.3.0** | Already used by VC006 |
| Thin Path B alone as Path A proof | Forbidden |
| VISION L2 complete | Needs VC008 + VC009 |
| Byte-identical epoch vs `dsb-context` launch stamp | Residual (Grok mirror hashes stable body string; library hashes message JSON) — do not equate epoch files blindly |
| Full F1 `dsb-*` Cargo embed into Grok | Intentional mirror (Spec 15 pattern) |

---

## 7. What shipped

1. **`spec10_path_a_assembly.rs`** — Spec 10 ordered assembly, epoch, discovery, wire apply  
2. **`turn.rs`** — after `build_request`, `apply_spec10_to_conversation_request` mutates system + stamps turn epoch under `DEEPSEEK_BUILD_HOME`  
3. **Public-entry soft check** for `path_a_turn_prefix_epoch.txt`  
4. **12 unit tests** (`vc007_*`) green (9 core layout/epoch + wire rewrite, discovery, stamp, extract, apply helper)

### 7.1 Wire-mutation proof (V2-10-1 honesty)

**Production call site** (`turn.rs` after `build_request`):

```text
apply_spec10_to_conversation_request(&mut request, &cwd, Some(workspace), None)
```

**Mutation contract** (not stamp-only):

| Step | Behavior |
|------|----------|
| Extract | Leading `ConversationItem::System` content (or empty) |
| Strip prior Spec 10 block | `extract_base_system_prompt` cuts at `\n\n## Tools\n` (idempotent re-apply) |
| Discover | `discover_skills_index` + `discover_project_instructions` under workspace root |
| Assemble | Ordered body: base system → `## Tools` (canonical tool JSON) → `## Skills index` → `## Environment` → optional `## Project instructions` |
| **Mutate wire** | Overwrite `sys.content` with `assembled.stable_body` (or insert leading System if missing) |
| Stamp | Best-effort `DEEPSEEK_BUILD_HOME/path_a_turn_prefix_epoch.txt` |
| Volatile | Non-system items left on `request.items`; epoch ignores volatile count |

**Unit golden** `vc007_wire_rewrite_mutates_system_message`:

- Starts with system `"GROK_BASE_TEMPLATE"` + user `"hello"` + tools
- After apply: `req.items[0]` system contains `GROK_BASE_TEMPLATE`, `## Tools`, `## Skills index`, `## Environment`, `## Project instructions`, and `AGENTS.md` body from temp workspace
- `sys == assembled.stable_body`
- Second apply → **same epoch** (idempotent)

**Not claimed as live R0A JSONL:** installed prebuilt agent has not been rebuilt in this story; soft e2e only **warns** if turn stamp missing.

### Commands (re-run this session)

```bash
cargo test -p xai-grok-shell --manifest-path third_party/grok-build/Cargo.toml --lib vc007_
# 12 passed (incl. vc007_wire_rewrite_mutates_system_message)

cargo test -p dsb-context path_a
# 5 passed

python3 scripts/lib/analyze_path_a_prefix_wire.py \
  docs/product/evidence/PATH_A_R0_G008_PREFIX_WIRE_last.jsonl
# PASS (historical; system_stable + skills_stable + volatile_grows; not post-VC007 Spec 10 section capture)

./scripts/check-path-a-linkage.sh   # PASS
./scripts/test-owner-bar.sh         # PASS 60/60; TSV restored
./scripts/test-heart-regression.sh  # PASS; TSV restored
```

---

## 8. Stack / PR shape

```text
origin/main @ 5.2.2
    └── … stacked Deep Code
            └── vc006-heart-r0a  (PR #138, product 5.3.0)   ← PR base
                    └── vc007-context-assembly  (this story; unversioned)
```

- **PR title:** `feat(context): Spec 10 assembly on Grok Path A turns`
- **Base:** `vc006-heart-r0a`
- **Body must include:** `Depends on #138`
- **Labels:** `feat`, `area/cache` (existing)
- **Merge:** **do not merge** in this story
- **Public text:** English only

---

## 9. Adversarial review (independent Grok)

| Round | Verdict | Action |
|-------|---------|--------|
| 1 (critic) | **NOT READY** — stamp-only hollow path; empty skills/project; A7 over-claim on G008 wire | Fixed in `5199d6d` wire rewrite + discovery + honest residuals |
| 2 (fresh, post wire-rewrite) | **READY** | Confirms wire mutation + discovery; A7 must stay **RESIDUAL**; memory-in-base residual explicit |

### Residual honesty (fail-close) — round 2 required list

| Residual | Honesty |
|----------|---------|
| Live prebuilt agent | Must be **rebuilt/repackaged** before turn stamp / Spec 10 section markers appear on install dogfood |
| A7 live multi-turn JSONL | **Unit + G008 historical only** — not post-VC007 Spec 10 section wire capture |
| Epoch domains | Launch `path_a_prefix_epoch` (`dsb-context` message JSON hash) ≠ turn `path_a_turn_prefix_epoch` (Grok stable body string hash) — **do not equate** |
| Tools dual placement | Spec 10 tools **document** in system **and** API `tools[]` retained (hybrid) |
| Memory / Grok system injections | `build_request` may inject memory into System **before** Spec 10 apply; pure Spec 10 volatile isolation incomplete for memory |
| Chat-kind product REST skills | Disk `skills/*/SKILL.md` discovery only; `user_skills_root` is `None` on turn path |
| Grok base system content | Base template may still carry Grok product chrome; wall-clock negative is fixture-level |
| VISION L2 | **Not complete** — VC008 + VC009 remain |
| SemVer | **No bump**; stack stays **5.3.0** from VC006; Reasonix cut residual **5.4.0** |

---

## 10. READY checklist

- [x] Unit 1 committed first  
- [x] Spec 10 turn assembly implemented with units green (12/12 re-run)  
- [x] Wire system rewritten (not stamp-only) — §7.1  
- [x] Skills/project discovery on turn path  
- [x] Three gates green (this session re-run); TSV side-effects not committed  
- [x] Fresh independent Grok adversarial review of rewritten implementation (**READY**, round 2)  
- [x] No SemVer bump  
- [x] Stacked PR #139 already open — body refresh after READY (do not open a second PR)  
- [x] **Do not merge**  

### Commits

| SHA | Message |
|-----|---------|
| `e315c76` | docs(product): VC007 Spec 10 Path A assembly plan and floor |
| `154bb8e` | feat(context): Spec 10 assembly on Grok Path A turns |
| `2c7fe52` | test(scripts): soft-check VC007 Path A turn prefix epoch stamp |
| `5199d6d` | feat(context): rewrite Path A wire system to Spec 10 layout |
| `ec90a66` | docs(product): VC007 READY evidence and adversarial close-out |
| `f4a5cfe` | docs(product): VC007 wire-mutation proof and residual honesty |
| *(this)* | docs(product): VC007 adversarial-2 READY close-out |
