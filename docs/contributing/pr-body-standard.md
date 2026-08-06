# PR body standard (narrative bar)

**Normative.** Thin checklists without engineering narrative fail this standard even if CI is green.

## Where this bar comes from

Mature product repos (notably **Orca** — `stablyai/orca` / local `OpenSources/orca`) treat the PR description as the primary review artifact:

| Orca template section | Purpose |
|----------------------|---------|
| **Summary** | User/engineering-visible change as a story, not a file dump |
| **Screenshots** | Visual proof or explicit “No visual change” |
| **Testing** | Real commands with checkboxes; honest skips |
| **AI Review Report** | What the agent/human review actually checked |
| **Security Audit** | Input, exec, paths, auth, secrets, IPC/deps |
| **Notes** | Limits, follow-ups, platform quirks |

Real Orca PRs go further inside Summary: **wrong assumptions vs reality tables**, **what changed** grouped by concern, **before/after evidence tables**, **why two fixes belong in one unit**, and **explicit non-goals**. Small fixes still get Problem / Solution, not “misc”.

DeepSeek Build adopts the **same narrative depth**, adapted to a docs-first coding-agent CLI (no Electron UI required; terminal captures and doc walkthroughs substitute screenshots).

## Required shape (template)

See [`.github/PULL_REQUEST_TEMPLATE.md`](../../.github/PULL_REQUEST_TEMPLATE.md).

Minimum sections for a **ready** PR:

1. **Summary** with subsections when non-trivial:
   - Problem  
   - What changed  
   - Out of scope  
2. **Screenshots / evidence**  
3. **Testing** (commands or justified N/A)  
4. **Kind** + **Related** + **Cache impact**  
5. **AI review report** (or explicit self-review one-liner for trivial docs)  
6. **Security audit**  
7. **Notes**  
8. **Checklist**

## Summary quality bar

### Fail (too thin)

```markdown
## Summary
- Update docs
- Fix CI
- Add labels
```

### Pass (Orca-like density, scaled to the change)

```markdown
## Summary

### Problem
PR #1 installed process *gates* (title regex, kind labels, short rule lists)
without enough process *substance*. A contributor (or agent) could still open a
PR that is CI-green and review-useless: empty Summary bullets, no test narrative,
no cache/security thinking.

### What changed
- Rewrite `docs/contributing/pull-requests.md` into an operating guide …
- Add worked examples modeled on Orca-level narrative density …
- Align the GitHub PR template with Orca’s section set (Summary / evidence /
  Testing / AI review / Security / Notes), adapted for CLI + specs.

### Out of scope
- Product specs 10–110
- Runtime crates
- Changing squash-merge or kind-label CI policy beyond required-path updates
```

### Techniques that make Summary reviewable

| Technique | When |
|-----------|------|
| Assumptions vs reality table | Bugfixes where prior code guessed wrong |
| Before / after table | Behavior or parser/output changes |
| “What changed” bullets grouped by subsystem | Multi-file but single unit |
| Why this is **one** PR | When a reviewer might ask to split |
| Explicit limitations | Frame cannot express X; we report null rather than guess |
| Credit / supersedes | Replacing an earlier attempt |

## Testing section quality bar

- Prefer **named commands** and **which suites**.  
- If you did not run the full suite, say what you ran and what you skipped **and why**.  
- For docs/spec: name the sections a reviewer must read; “docs look fine” fails.  
- For agent behavior: include a smoke scenario that would fail on `main`.

## AI review report quality bar

Not “Reviewed with Claude. LGTM.”

Include:

- Focus areas (false positives, cache prefix, permissions, …)  
- What was flagged and what you changed  
- Cross-platform / cross-runtime notes when relevant (macOS/Linux/Windows paths, shell)

## Security audit quality bar

Not “N/A” by default on tool/runtime PRs.

Call out:

- Untrusted input (model output, tool args, terminal text, web)  
- Command execution / path join  
- Secrets in logs or fixtures  
- Permission boundary changes  
- Or: **No security-sensitive surface** with a one-line reason (CSS-only, pure docs prose, …)

## Mapping: Orca → DeepSeek Build

| Orca | DeepSeek Build |
|------|----------------|
| `pnpm lint/typecheck/test/build` | Project commands as they exist (today: docs-hygiene CI + local scripts; later cargo/go/…) |
| Screenshots of Electron UI | TUI captures, CLI output, before/after tables |
| IPC / TCC / Electron notes | Tool permissions, sandbox, API keys, cache-stable prefix |
| STA-#### issue ids | GitHub issues + milestone M1–M6 |
| AI Review Report | Same section name — keep it |

## Anti-patterns

| Anti-pattern | Why it fails the bar |
|--------------|----------------------|
| File list as Summary | No problem statement; no design |
| All Testing unchecked with no reasons | Unverifiable |
| Empty AI Review / Security on non-trivial PRs | Review theater |
| “See commits” | Squash merge discards commit story on `main` — **PR body is the record** |
| Mixing three milestones “documented thoroughly” | Narrative cannot rescue wrong unit size |

## Related

- [examples.md](./examples.md)  
- [pull-requests.md](./pull-requests.md)  
- [review-checklist.md](./review-checklist.md)  
