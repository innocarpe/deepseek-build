# What changes in DeepSeek Build `6.1.0`

**The one file.** If you read nothing else about this minor, read this: what
the DeepSeek-native depth train made true, what it measured and then declined,
and what it does not claim.

**Why there is a minor.** `6.0.0` moved the vendored base to Grok Build
`1.0.41`. This train spends that base. It reflects DeepSeek's official harness
(`dsh`) only where this product was still wrong: a cached prefix should survive
a session change, a cache miss should say what moved, and the request should
match the log. It does not add a surface, a pillar, or a new identity.
The board is [DEEPSEEK_NATIVE_DEPTH_6X_GOALS.md](./DEEPSEEK_NATIVE_DEPTH_6X_GOALS.md).
The evidence note is [dsh-deepseek-harness.md](../research/dsh-deepseek-harness.md).

**Where the bytes are.** The behavior below is on `main` at `e914dfb`, which
is also the commit `v6.0.2` tags. That tag was cut before this summary existed.
`v6.1.0` is the name this train reserved for the cut. It is not published yet.

---

## 1. What you can see

| What | Before → after |
|---|---|
| **A cache miss** | The turn could say only that the prefix epoch changed. On Path A, `turn.rs` now logs which of the five assembled documents moved (`prefix_change=`). Categories the assembly cannot see are called `unattributed`, not invented. |
| **The session's cache total** | Hits and misses were per turn. The in-memory ledger now logs one `cache_session=` line per turn (`turn_end.rs`). The status chip is still the per-turn percentage. The counter is not written to disk. |
| **A request that does not match the log** | A divergent non-system item could still be sent. `check_request_projects_log` runs before `run_turn_via_sampler`. Divergence fails the turn. An item that cannot be serialized fails closed. |
| **A later change of the stable system body** | The earlier system message stays byte-for-byte. The new body is appended. Replacing the `tools` array is not treated as a history event. |
| **A deny that a later allow reopened** | `DenyOnly` abstains or denies. `combine_decisions` will not let an allow replace a deny. |

## 2. Measured, then not rebuilt

These were on the `1.0.41` tree before this train. They were checked and left
alone. Board §1 has the file paths.

- Spill of an oversized tool result (head, tail, and a path to the rest).
- Compaction that keeps the warm prefix aligned.
- A snippet staleness guard stricter than dsh's path-scoped token.
- `get_task_output` / `list_tasks` for background work.
- A model-visible note beside a tool result (`wrap_hook_note`), not inside the tool's own output.

Spill was not re-run on a session that had already blown the context. The
exit line that asks for that demonstration is **not claimed**.

## 3. Deliberately not taken

The ledger is board §7. Short form:

- Request-series bookkeeping (`initial` / `resume` / `change`). This product's instrument is the prefix hash.
- Agent teams, mailboxes, task boards. L3 here is a worktree plus subagent fan-out.
- Sandbox escalation. This product has allow / deny / ask, not sandbox modes.
- A plugin host. This product is a Rust pager and an overlay.
- Moving the live API to Anthropic Messages. Re-affirmed 2026-09-26: official Chat Completions cached a stable prefix (1280 hit / 228 miss on a 1508-token prompt) and kept an appended system message (84%). Effort levels, signed thinking, and image handling were not measured, and they are not a reason to switch.
- A new TUI pane for the cache total. The new line is a log.

## 4. What this file does not say

- It does not say `v6.1.0` is on npm. It is not.
- It does not say the status chip shows the session total. It does not.
- It does not say the session total survives a restart. It does not.
- It does not say a live cache-hit percentage was re-tuned. Spec 10 §1.9's mock threshold stays 90%.
