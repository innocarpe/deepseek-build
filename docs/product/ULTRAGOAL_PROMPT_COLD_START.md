# Ultragoal cold-start prompt (copy below the line)

The fenced block is the **full prompt** to paste into a new session (ultragoal / autonomous agent).  
Repo root must be: `deepseek-build` on `main` (post PR #6).

---

```text
# ROLE

You are an autonomous coding agent implementing **DeepSeek Build** (`dsb`).
This is a **cold start**. Do not assume prior chat memory. Load truth only from this
repository and the env.

# REPO

- Path: current workspace = deepseek-build git root
- Branch base: `main` (pull latest first)
- Remote: origin (innocarpe/deepseek-build)
- Gates ledger: `docs/GATES.md` — **G0, G1, G1b, G2 are GREEN**. G3–G6 stay RED.

# MISSION (single goal)

Complete **M1 — Provider + cache + routing** end-to-end:

1. Rust Cargo **workspace** under `crates/` per `docs/adr/0004-toolchain.md`
   - CLI binary name: **`dsb`**
   - Suggested crates: `dsb-cli`, `dsb-agent`, `dsb-provider-deepseek`,
     `dsb-context`, `dsb-config` (split only if needed; do not invent a monorepo zoo)
2. DeepSeek **Chat Completions** client per `docs/adr/0005-deepseek-provider-contract.md`
   - Base URL: `https://api.deepseek.com`
   - Wire models: **`deepseek-v4-flash`** (default), **`deepseek-v4-pro`** (escalate)
   - Auth: `DEEPSEEK_API_KEY` (or `~/.deepseek-build/credentials.json` mode 0600 — never commit secrets)
   - Streaming SSE; separate `reasoning_content` vs `content`
   - Thinking: `extra_body.thinking.type` enabled/disabled; `reasoning_effort` low|high|max
   - **With tools:** always pass back `reasoning_content` on subsequent API calls or API 400s
3. Stable prefix builder + **golden byte tests** per `docs/specs/10-cache-contract.md`
4. Tool-call repair + tool/result pairing per `docs/specs/15-tool-call-repair.md` (M1 **must**)
5. Flash/Pro routing + visibility per `docs/specs/20-model-routing.md`
6. Thinking/effort request shape per `docs/specs/30-thinking-effort.md`
7. **Headless or thin CLI** loop:
   - user message → build messages (stable prefix + tail) → model → print assistant
   - Multi-turn REPL or `dsb run "..."` + session file under `~/.deepseek-build/` or project temp
8. Smoke when `DEEPSEEK_API_KEY` is set:
   - multi-turn chat works on flash
   - one-shot pro escalate works and is **visible** in output/logs
   - golden prefix tests pass without network
   - cache evidence: parse usage cache fields if present, else dual-call substitute protocol (ADR 0005)

# DESIGN SPINE (must obey — not optional)

Read in this order before coding:

1. `docs/architecture/HARNESS_PHILOSOPHY.md`  (L1 Deep Code / L2 Reasonix / L3 Grok)
2. `docs/product/SOURCES.md`
3. `docs/GATES.md`
4. `docs/adr/0004-toolchain.md`
5. `docs/adr/0005-deepseek-provider-contract.md`
6. `docs/specs/10-cache-contract.md`
7. `docs/specs/15-tool-call-repair.md`
8. `docs/specs/20-model-routing.md`
9. `docs/specs/30-thinking-effort.md`
10. `docs/product/MILESTONES.md` (M1 section only)
11. `AGENTS.md` + `skills/pr-authoring/SKILL.md`
12. `docs/contributing/pr-body-standard.md` (Orca-level PR bodies)

**Layer rule:** L3 (speed/parallel/subagents) must **never** override L1/L2
(cache, tool repair, DeepSeek-native contracts). M1 does not implement L3 fan-out.

# HARD NON-GOALS (do not implement)

- Spec **45** snippet edit, free-form whole-file edit as primary path
- Shell / bash mutating tools, permissions product (spec **90**) — no shell tool in M1
- Parallel tool dispatch (spec **50**), subagents/worktrees (spec **60**)
- Skills product (70), MCP (80), sessions UX polish (100), plan mode (110) beyond minimal chat
- Gajae-style multi-stage planning (deep-interview / ralplan / ultragoal product features)
- Process-police CI (PR title regex, kind-label counters, markdown path inventories)
- Hard-fork of Grok Build / shipping xAI branding
- Committing API keys, session dumps with secrets

If tempted to build M2+ “because it’s useful,” **stop** and open a `spec` PR instead, or leave a Notes follow-up.

# WORK STYLE

1. `git fetch && git checkout main && git pull`
2. One **meaningful unit per PR** (prefer small vertical slices), e.g.:
   - PR: workspace + hello `dsb --version`
   - PR: provider client + streaming unit tests (mock HTTP)
   - PR: context/prefix + golden tests (spec 10)
   - PR: agent loop + repair (spec 15) + routing (20) + thinking wire (30)
   - PR: smoke/docs update
3. Branch names: `feat/...`, `test/...`, `chore/...`
4. PR titles: Conventional Commits (`feat(provider): ...`)
5. **Exactly one kind label** on ready PRs: feat|fix|docs|spec|chore|refactor|test|ci
6. PR body: Orca-level narrative — Problem / What changed / Out of scope /
   Testing (real commands) / AI review / Security / Notes / Cache-impact
7. Squash-merge policy on this repo; do not force-push `main`
8. After each merge: pull main before next branch
9. Prefer tests that fail on `main` before feature if TDD is natural; at minimum
   land automated tests listed in specs 10/15/20 with the implementing PR

# IMPLEMENTATION HINTS (non-binding, but preferred)

- `tokio` + `reqwest` for HTTP; `serde_json` with a **single** canonical serialize path for prefix bytes
- Config: `~/.deepseek-build/config.toml` optional; env overrides win for API key
- Logging: show `model=` every turn; log `prefix_epoch=` (sha256 hex prefix)
- M1 tools: **none required**. If you add tools, only **read-only** and still obey repair/pairing; do not add edit/shell
- Windows: best-effort; primary targets macOS/Linux per ADR 0004

# SUCCESS CRITERIA (all required)

- [ ] `cargo test` (workspace) passes including golden prefix + repair + routing tests
- [ ] `cargo build -p dsb-cli --release` (or dev) produces runnable `dsb`
- [ ] `dsb --help` / `--version` works
- [ ] With `DEEPSEEK_API_KEY`: multi-turn chat works on **deepseek-v4-flash**
- [ ] Explicit Pro escalate uses **deepseek-v4-pro** and is user-visible
- [ ] Spec 10/15 automated tests exist and pass offline
- [ ] No secrets in git; `.gitignore` covers credentials if needed
- [ ] README quickstart updated: install/build, set key, run chat
- [ ] `docs/GATES.md` still accurate (do not mark G3+ green)

# STOP / ESCALATE

Stop and report clearly if:

- DeepSeek API rejects pinned model ids (open ADR supersession note; do not invent ids)
- You need network and key is missing — finish offline tests + document smoke steps
- Scope pressure to implement M2+ — refuse and list follow-ups

# FINAL REPORT (when M1 done)

Write a short report covering: PRs merged, how to run, test commands, known limits,
and recommended next gate (**G3**: specs 45 + 90 minimum).

Begin by pulling main, reading the design spine list above, then open the first
small PR (workspace + `dsb --version`).
```

---

## Operator checklist (you, human)

1. `cd` into `deepseek-build`, `git checkout main && git pull`
2. Optional: `export DEEPSEEK_API_KEY=...` in the ultragoal environment
3. Paste the fenced prompt into a **new** session with ultragoal enabled
4. Do not paste prior chat history (cold start)
