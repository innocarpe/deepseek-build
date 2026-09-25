# Upstream gap sweep — 2026-09-25

**Nature:** Non-binding evidence ([SOURCES.md](../product/SOURCES.md) precedence).
A judgment table, not a product commitment. Promote an item into
`docs/specs/`, `docs/product/`, or an ADR before building it.

**Question asked:** what do the two upstreams (Deep Code, Reasonix) have today
that DeepSeek Build does not, and which of it is worth taking?

**Method.** Shallow clones read at 2026-09-25 (`_upstream/deepcode`,
`_upstream/reasonix`), compared against this workspace's `crates/` and
`docs/specs/`. Both clones are shallow, so **no commit history was available**
and "recent" could not be reconstructed from `git log`; the inventory is the
tree at HEAD plus each project's release notes. Judgments follow
`NON_GOALS.md` and `HARNESS_PHILOSOPHY.md` §3 — L3 may never override L1/L2
"for speed", and "upstream has it" is not a reason.

---

## Judgment table

| Item | Upstream source | dsb today | Verdict |
|------|-----------------|-----------|---------|
| **Line-ending round-trip on edit** | Deep Code `common/file-utils.ts:18-88` (`normalizeContent` / `detectLineEndings` / `writeTextFile`); tests `write-handler-line-endings.test.ts`; new-file EOL from `git check-attr -z text eol` | Spec 45 §1.8 asked for preservation; `SnippetStore` split on `\n` and matched raw, so a multi-line `old_string` **failed on a CRLF file** (`no_match`) and a single-line edit **wrote LF into it** | **Take** — spec-vs-code divergence; fixed in this sweep |
| Cache-miss **attribution** | Reasonix `runtime/agent/cache_shape.go` (`CaptureShape` / `CompareShape`), `contract/event/cache_diagnostics.go` (`PrefixChangeReasons`, `body_unreported`) | Spec 10 §1.5 logs `prefix_epoch` only; an epoch change says *that* the prefix moved, never *what* moved | **Take (next)** — highest value on the L2 axis |
| Session-cumulative hit/miss rate | Reasonix `Controller.SessionCache()`, TUI status line | `CacheEvidence` parsed per turn and logged; no cumulative counter, no status surface | **Take (next)** — small, directly serves the cost culture |
| Scored cache regression bench | Reasonix `scripts/cache-guard.sh` + `cachehit_e2e_test.go` (8 scenarios, 3-turn tail average vs 90%) | Spec 10 has prefix-equality goldens, but no scenario bench and no threshold | **Take (next)** — makes "cache-first" testable rather than aspirational |
| Rewritten-test detection | Reasonix `safety/evidence/test_criteria.go` (141 LOC, Go) + `pytest_criteria.go` (388) | Nothing detects a turn that rewrote or deleted existing tests | **Take (later)** — cheap for the Go half; names a real failure mode |
| In-process client + `!` command | Reasonix `frontend/serve/inprocess.go` (130), `acp/shell_prompt.go` (35) | No `!` user-shell command; no transport seam | **Hold** — the grant story is elegant but it presumes a `serve` split dsb has not made; revisiting it is an ADR, not a patch |
| Egress allow-list + "ask the user" | Reasonix `safety/egress/policy.go`, `bash_egress.go` | No egress policy; bash permissions are scope-based only | **Hold** — the model is right, the implementation is bwrap/Seatbelt plumbing; the ~72-line policy half is the only cheap slice |
| Cost / pricing model (off-peak, holidays) | Reasonix `contract/pricing/*` (`window.go`, `rates.go`, `quote.go`, `catalog.go`) | No cost model anywhere in `crates/` | **Hold** — a hardcoded price table that goes stale annually and a vendor endpoint-identity rule; the cost *readout* problem is better solved by the attribution + cumulative-rate items above |
| Session store (CAS ledger, append-only log, leases, recovery forks) | Reasonix `state/sessionstore/save.go` (2,074 LOC) | Spec 100 + JSONL replay; no ledger | **Reject (now)** — over-build for a terminal harness; the minimal idea (anchor transcript vs append-only event log) is not worth the machinery yet |
| `best_of_n` judge | Reasonix `runtime/bestof/*` | — | **Reject** — child kernels × worktrees × a judge model, and it applies an *external* result, inverting the subagent contract in spec 60 |
| Electron desktop / multi-provider identity / every TOML dial | Reasonix `desktop/`, config surface | — | **Reject** — already `Leave` in [SOURCES.md](../product/SOURCES.md) §2; nothing found justifies reversing |
| VSCode companion v0.4.1 | Deep Code `packages/vscode-ide-companion` (imports core in-process, webview IPC) | — | **Reject** — "VS Code extension MVP" is an explicit non-goal; Deep Code already has one |
| Gajae-class multi-stage planning | — | Plan mode is light by design | **Reject** — explicit NON_GOAL |
| Static bash side-effect classifier | *(not present upstream)* | `permissions.rs::classify_bash` + declared `side_effects` | **No gap** — the brief's premise is wrong here: Deep Code is **declaration-first**, the model classifies and an `unknown` declaration collapses to `ask`. dsb's independent classifier is the stricter design |
| Tool-call repair | Deep Code `common/validate.ts`, `openai-message-converter.ts`; Reasonix `provider.go:415-500` | `dsb-agent/src/repair.rs` threads a JSON schema | **No gap** — dsb's spec 15 §1.1 promises schema defaults and `additionalProperties` stripping; Reasonix does JSON well-formedness only. Worth stealing one trick: truncation closing that degrades to `{}` |

---

## What this sweep changed

One row was already a **contract violation, not a missing feature**:
spec 45 §1.8 said "preserve original file newline style on write when
possible" while the code could not. Per the repo's precedence rule the spec
wins, so the code moved — spec 45 gained §1.9 (LF in, file's convention out),
`SnippetStore` gained `LineEnding`, and both edit entry points (thin `edit`
tool and Path A `search_replace`) now share one rule.

Measured before the change, on a CRLF file (`alpha\r\nbeta\r\ngamma\r\n`):

| Call | Before | After |
|------|--------|-------|
| `edit(old="beta\ngamma")` | `no_match` | applies |
| `edit(old="beta")` | applies, file becomes `alpha\r\nBETA\ngamma\r\n` (mixed) | `alpha\r\nBETA\r\ngamma\r\n` |
| `replace_all` on CRLF | count computed on raw text | same rule as `edit` |

---

## Limits of this sweep

- **Both clones are shallow**, so "recent activity" is not dated evidence. Any
  claim of the form "upstream just did X" needs a fresh, full clone before it
  is used to justify work.
- Reasonix's tree is far larger than its release notes; the eleven areas
  inventoried were chosen from the brief. Areas not looked at: `bestof`
  internals beyond the judge policy, `isolation/`, `benchmarks/`, `sdk/`,
  `workers/`.
- Verdicts of **Hold** carry a maintenance cost the reviewer should weigh: the
  pricing table and the holiday list go stale on a vendor's schedule, not ours.
