# Release train — Wave A (`0.x.y` dogfood band)

**Status:** Active Wave A detail (see full vision board: [MASTER_PLAN.md](./MASTER_PLAN.md))  
**SemVer rule:** Always full `MAJOR.MINOR.PATCH` — see [versioning.md](../contributing/versioning.md).  
**CLI:** `deepseek-build` (primary) · `dsb` (alias) — [ADR 0006](../adr/0006-cli-names-and-semver.md).  
**After this train:** continue [ULTRAGOAL_CHAIN.md](./ULTRAGOAL_CHAIN.md) → `native-0x` (not stop forever).

---

## 1. Intent

We stay on the **`0.y.z` line for a long time.**

| Do | Do not |
|----|--------|
| Ship frequent **`0.y.z`** slices that a human can install and try | Rush a fake **`1.0.0`** |
| Define **dogfood-usable** as the near north star | Treat M6/`1.0.0` as the only meaningful finish line |
| Map each `0.y.0` minor to a **user-visible capability** | Bump versions with no usable delta |

**`1.0.0` is out of scope for this train.** It is only considered *after* dogfood-usable is true for weeks and packaging is boring. Until then every release is `0.y.z`.

Milestones **M0–M6** (feature themes) still apply. This file is the **SemVer release train** that sequences them into installable versions.

---

## 2. Where we are now

| Item | Value |
|------|--------|
| Current version | **`0.3.0`** |
| What works | PATH install; provider; cache; routing; `run`/`chat`; tools: **read/edit/write/grep/bash**; **`--dogfood`** profile (workspace write + bash execute; out-of-cwd still denied) |
| What does **not** | npm; sessions; skills; parallel/subagents; fully frictionless hour-long coding without reading flags once |

**Honest label for `0.3.0`:** installable + coding tools daily (`grep` + dogfood write/bash profile); dogfood-usable §3 still needs real-owner proof (**0.4.0**).

---

## 3. Dogfood-usable definition (train exit for “I can use this”)

Owner (you) can do **all** of the following on a real repo without reading the Rust tree:

1. **Install once** so `deepseek-build` and `dsb` are on `PATH` (script, `cargo install`, or npm — at least one supported path).  
2. **Auth once** (`DEEPSEEK_API_KEY` or `~/.deepseek-build/credentials.json`).  
3. From a project directory:  
   `deepseek-build chat` or `dsb chat`  
   multi-turn Flash chat works; `/pro` shows `deepseek-v4-pro`.  
4. Agent can **read** project files, **edit** via snippet contract, **create** new files, run **search/grep**, and run **bash** under permissions (not permanently dry-run for trusted local use).  
5. Default or one documented profile allows **workspace write** without remembering obscure flags every time (still fail-closed outside workspace).  
6. Documented smoke in README reproduces the above.  
7. Version string is full SemVer (e.g. `deepseek-build 0.4.0`).

When this holds, we call the train **dogfood-usable** (still **`0.y.z`**, not `1.0.0`).

---

## 4. Planned minors (`0.y.0` themes)

Patch versions (`0.y.z`, z>0) are bugfixes/docs on the same theme. Minors below are **capability jumps**. Dates are not promised — order is.

| Target SemVer | Theme | User can… | Maps roughly to |
|---------------|--------|-----------|-----------------|
| **`0.1.0`** | Engine preview | Build from source; API chat; tool core | M1 + tools start |
| **`0.2.0`** | **Installable CLI** | Put `deepseek-build`/`dsb` on PATH without remembering cargo flags | packaging slice of M6 early |
| **`0.3.0`** | **Coding tools daily** | grep/search; bash execute under policy; dogfood-friendly workspace write profile; agent loop hardened | M2 core (minus parallel) |
| **`0.4.0`** | **Dogfood proof** | Owner completes a real small change in this repo *using* the agent; notes in docs | M2 dogfood exit |
| **`0.5.0`** | **Sessions** | Resume a prior chat/session under `~/.deepseek-build/` | M5 partial |
| **`0.6.0`** | **Surface** | Skills index + load; thinking/effort user flags; basic `/model` or flags | M3 partial |
| **`0.7.0`** | **npm distribute** | `npm i -g …` exposes both bin names; version matches cargo | packaging |
| **`0.8.0`** | **Parallel tools** | Independent tools in one turn; bg shell collect (needs G4 / spec 50) | M2 parallel / M4 prep |
| **`0.9.0`** | **Hardening** | CI smoke; known-limits; cost/cache hints; changelog discipline | M6 partial |
| **`1.0.0`** | **Later** | Only after sustained dogfood + boring install | *not this train* |

### Explicit non-goals of the `0.x` train (until scheduled)

- Declaring **`1.0.0`**
- Full subagent/worktree product (may start after dogfood; still `0.x` if needed)
- Gajae multi-stage planning
- Process-police CI

---

## 5. Ultragoal mapping

Durable ultragoal plan id: **`dogfood-0x`** (see `.omc/ultragoal/plans/dogfood-0x/` when created).

| Story | SemVer target | Objective |
|-------|---------------|-----------|
| Install | **`0.2.0`** | Local install path + PATH + dual bin smoke |
| ToolsDaily | **`0.3.0`** | Search + bash execute + dogfood write profile + tests |
| DogfoodProof | **`0.4.0`** | Real task on this repo; document commands used |
| Sessions | **`0.5.0`** | Persist/resume session JSONL |
| Surface | **`0.6.0`** | Skills min + model/effort UX |
| Npm | **`0.7.0`** | npm package both bins; matching SemVer |
| Parallel | **`0.8.0`** | Spec 50 + G4 + parallel dispatch |
| Harden | **`0.9.0`** | CI smoke + limits + changelog |

After **DogfoodProof (`0.4.0`)** the owner re-evaluates: keep pushing `0.5.0+` or pause on daily use.

---

## 6. Rules for agents

1. Prefer **small vertical PRs** that can ship a `0.y.z` or progress one story.  
2. Every release PR: bump workspace SemVer (`MAJOR.MINOR.PATCH` full form), run `./scripts/check-semver.sh`, mention both CLI names.  
3. Do not mark **`1.0.0`** or “v1 done” in ultragoal evidence.  
4. Update this table checkboxes when a minor ships (in the same PR as the version bump when possible).  
5. GATES still gate features (G4 before parallel, etc.).

---

## 7. Progress log

| SemVer | Date | Notes |
|--------|------|--------|
| `0.1.0` | 2026-08-06 | Engine + dual CLI from source; tools core |
| `0.2.0` | 2026-08-06 | PATH install for `deepseek-build` + `dsb` (#18) |
| `0.2.0` | 2026-08-06 | Install path: `scripts/install.sh` + documented `cargo install`; both `deepseek-build` and `dsb` on PATH; README clean-shell smoke |
| `0.3.0` | 2026-08-06 | Tools daily: `grep` tool; bash execute under policy; `--dogfood` workspace-write+bash profile (out-of-cwd denied); tests green |

---

## 8. Related

- [MILESTONES.md](./MILESTONES.md) — feature themes M0–M6  
- [versioning.md](../contributing/versioning.md) · [releases.md](../contributing/releases.md)  
- [GATES.md](../GATES.md)  
