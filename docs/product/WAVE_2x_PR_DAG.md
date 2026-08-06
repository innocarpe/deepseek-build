# Wave 2.x PR DAG — Grok base product train

**Normative for product work after [REPLAN_2.0.md](./REPLAN_2.0.md).**  
**Plan id:** `grokbase-2x` — full story board: [GROKBASE_2X_GOALS.md](./GROKBASE_2X_GOALS.md)  
**Do not invent overnight units** — extend this file in a docs PR if the DAG must change.

Scaffold Waves A–D (`WAVE_A_PR_DAG.md`, `WAVE_B_PR_DAG.md`) are **historical**.

---

## Legend

| Field | Meaning |
|-------|---------|
| **Unit** | Mergeable PR-sized story |
| **Depends** | Must merge before this unit starts (or stack on top) |
| **Band** | Target SemVer band when unit ships |
| **Evidence** | Required for “done” (not vibes) |

Default merge policy: **serial** unless unit says parallel-safe.

---

## W0 — Research (docs + spike evidence)

| ID | Unit | Depends | Band | Evidence |
|----|------|---------|------|----------|
| **2x-W0-1** | ADR-0008 Grok Build base (A fork vs B subtree) + Apache-2.0 / NOTICE / SOURCE_REV | replan merge | docs | ADR merged on `main` |
| **2x-W0-2** | Research note: crate map, auth/provider plug points, config entry | 2x-W0-1 (or parallel with draft ADR) | docs | `docs/architecture/GROK_BASE_SPIKE.md` (or equivalent) |
| **2x-W0-3** | Local build spike: `cargo check -p xai-grok-pager-bin` on `../grok-build` | — | docs | Command + pass/fail + toolchain notes in spike doc |

**W0 exit:** ADR accepted + spike doc lists concrete injection points for DeepSeek auth/models.  
No product tag required.

---

## W1 — Shell (alpha)

| ID | Unit | Depends | Band | Evidence |
|----|------|---------|------|----------|
| **2x-W1-1** | Integrate Grok tree per ADR (fork layout or subtree pin) | W0 exit | `2.0.0-alpha.N` | Tree builds in CI or documented CI plan |
| **2x-W1-2** | Binary rename / dual bin: `deepseek-build` + `dsb` entry → Grok pager composition root | 2x-W1-1 | alpha | `dsb` with no args opens full-screen agent (TTY) |
| **2x-W1-3** | Branding strings + DeepSeek identity (not “Grok” as product name in UI chrome) | 2x-W1-2 | alpha | Screenshot or headless smoke note |
| **2x-W1-4** | First-run setup/auth path wired into new entry (reuse 1.x credentials story) | 2x-W1-2 | alpha | Missing key → setup; key stored 0600 |

**W1 exit:** Dogfoodable “open the agent shell” without DeepSeek chat necessarily perfect.

---

## W2 — DeepSeek wire (beta)

| ID | Unit | Depends | Band | Evidence |
|----|------|---------|------|----------|
| **2x-W2-1** | Default provider/models = DeepSeek (base URL, model ids) | W1 exit | `2.0.0-beta.N` | Live or recorded chat turn |
| **2x-W2-2** | Port/adapt `dsb-provider-deepseek` or equivalent into Grok HTTP path | 2x-W2-1 | beta | Unit/integration test or dogfood log |
| **2x-W2-3** | Edit/tool loop works on a real repo (read/edit/bash via Grok tools) | 2x-W2-2 | beta | Dogfood note on this monorepo or sample repo |

**W2 exit:** Owner can `dsb` → chat → get real code changes with DeepSeek.

---

## W3 — L1 / L2 overlays (beta)

| ID | Unit | Depends | Band | Evidence |
|----|------|---------|------|----------|
| **2x-W3-1** | Snippet-safe edit policy under Grok tools (fail-closed) | W2 exit | beta | Contract tests ported/adapted from Spec 20/30 family |
| **2x-W3-2** | Permission model (ask/deny/allow; headless fail-closed) | 2x-W3-1 | beta | Tests + TTY vs headless matrix |
| **2x-W3-3** | Prefix/cache epoch discipline (Reasonix L2) | 2x-W3-1 | beta | Tests showing stable prefix or documented Grok-equivalent |
| **2x-W3-4** | Optional P1: skills index, Flash/Pro routing | 2x-W3-3 | beta | May slip post-2.0.0 if P0 already green |

**W3 exit:** REPLAN §2 P0 items 4–5 green under real shell.

---

## W4 — Product cut (`2.0.0`)

| ID | Unit | Depends | Band | Evidence |
|----|------|---------|------|----------|
| **2x-W4-1** | Install path: npm and/or install.sh produces `dsb` that opens agent | W3 exit (P0) | **2.0.0** | Fresh-machine install notes |
| **2x-W4-2** | README / package.json / KNOWN_LIMITS / user-guide rewrite for 2.0 | 2x-W4-1 | 2.0.0 | Docs match reality |
| **2x-W4-3** | Mark 1.x legacy in npm description + changelog | 2x-W4-2 | 2.0.0 | Published package metadata |
| **2x-W4-4** | Tag `v2.0.0` only after §2 P0 checklist signed | 2x-W4-1..3 | **2.0.0** | Release PR + tag |

**W4 exit = product done** per REPLAN success feeling. Not earlier.

---

## Explicitly out of this DAG

- 1.x whale banner / thin REPL cosmetics  
- Greenfield “Grok vibes” without Grok tree  
- Multi-vendor identity work  
- Forward-looking “wait for more dogfood days” as a gate substitute for P0 engineering  

---

## Status snapshot template

```text
W0 research:     ?/3
W1 shell:        not started | n/m
W2 deepseek:     not started | n/m
W3 l1/l2:        not started | n/m
W4 cut 2.0.0:    not started | n/m
Cargo/npm:       1.x scaffold | 2.0.0-alpha.* | 2.0.0-beta.* | 2.0.0
```
