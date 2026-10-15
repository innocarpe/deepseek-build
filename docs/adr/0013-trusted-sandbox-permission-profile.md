# ADR 0013 — Trusted-sandbox permission profile and a configurable tool-round cap

- **Status:** Accepted
- **Date:** 2026-10-15
- **Amends:** [Spec 90 — Permissions](../specs/90-permissions.md) §1.4 (names the shipped opt-in profile)
- **Evidence (off-repo scratch):** `/tmp/dsb-bench/jobs-tb/tb-smoke2-dsb/`,
  `/tmp/dsb-bench/jobs-deepswe/deepswe-smoke2-dsb/` (2026-10-15 benchmark session)

## Context

On 2026-10-15 the benchmark session ran the product's own headless entry
(`dsb run --dogfood --no-ask-permissions --preset flash --effort max`) against
Terminal-Bench 4.0 and DeepSWE tasks. All three trials scored 0. Two causes
were read from the trial logs:

1. **Eight tool rounds is a hard cap.** `AgentConfig::max_tool_rounds` defaults
   to 8 (`crates/dsb-agent/src/loop_.rs`) and neither a CLI flag nor a config
   key could raise it; each trial ended with
   `[warn] max tool rounds reached; stopping`.
2. **The headless policy denied commands the task needs.** The bash classifier
   emits `unknown`/`network` for ordinary commands, and headless ask → deny
   turned them into tool errors: `bash denied scopes=["unknown"] …` and
   `bash denied scopes=["network"] …`. `--dogfood` (the trusted local profile)
   still denies out-of-workspace write/delete and cannot run these tasks.

The same model and tasks pass when the harness drives the vendored agent with
`--always-approve` (DeepSWE smoke3: 3/3). The model was not the problem; the
product's own headless execution surface was.

## Decision

Two additive, default-preserving changes. No existing profile changes meaning.

1. **`--max-tool-rounds <N>`** (global; the native `run` / `chat` loop) raises
   the per-turn tool-round cap. The default stays 8 and `0` is rejected.
2. **`--dangerously-allow-all`** (global, explicit, per-invocation) builds
   `trusted_sandbox_policy`: all ten spec-90 scopes allowed, no ask/deny set,
   and bash executes. It exists for disposable sandboxes (benchmark / CI
   containers) where the sandbox boundary — not the tool-layer scope gate — is
   the safety boundary.
   - Flag only: never a config key, never implied by another profile, so the
     grant cannot persist into a later session.
   - `--dogfood`, `--allow-workspace-write`, the grants file, the product
     defaults, and the full-screen TUI path are untouched; without the flag
     behavior is byte-identical.
3. Spec 90 already permits an opt-in power-user profile (§1.4); this ADR names
   the one the product ships so spec and code stay aligned.

## Consequences

- The fail-closed defaults stay in force and stay covered by tests:
  `default_coding_policy` (headless ask → deny) and dogfood's out-of-cwd
  denial are unchanged.
- A user who passes `--dangerously-allow-all` on a real machine gets no scope
  protection: network egress, out-of-workspace write/delete, and unclassified
  commands all run. The help text and `docs/user-guide/08-permissions.md` say
  so, and the welcome banner names the profile `allow-all`.
- The product's own loop can run sandboxed benchmark tasks; the recorded repro
  case (`terminal-bench/wal-recovery-ordering`) completes with the flag set.
- `path_a_permissions` (Path A `yolo`) is a different surface and is not
  amended; Path A's yolo remains workspace-write only.

## Alternatives considered

| Option | Verdict |
|--------|---------|
| Raise the default cap above 8 | Rejected — the cap is fail-safe for interactive use; the ask is configurability, not a new default |
| Make the allow-all opt-in a config key | Rejected — persistence lets an accidental grant survive the sandbox |
| A `permissions.default = "allow"` config surface (spec 90 §1.4) | Rejected for now — a whole-file policy config is larger than the need; the CLI flag is the smallest explicit opt-in |
| Allow only `network` + `unknown`, keep out-of-cwd denied | Rejected — sandbox tasks legitimately touch paths outside the workspace root; a half-open profile reproduces the same failure class later |
| Change the vendored-agent path instead | Rejected — that path already has `--always-approve`; the missing surface is the product's own headless loop |
