## Summary

<!--
Write a real engineering narrative — not a file list.

Match the bar used on mature product repos (e.g. Orca): explain the problem,
what was wrong in the previous approach (assumptions vs reality when useful),
what changed and why those changes belong together, and what is deliberately
out of scope.

Prefer short tables, before/after evidence, and concrete commands/paths over
vague bullets like "improve docs" or "fix stuff".
-->

### Problem

<!-- Who hits this? What breaks or is missing? Link issue if any. -->

### What changed

<!-- Bullet the actual design/code/doc deltas. Group by concern. -->

-

### Out of scope / non-goals for this PR

-

## Screenshots / evidence

<!--
UI: screenshots or recording.
CLI/TUI: terminal captures, before/after tables, log snippets (redact secrets).
Docs-only: say "No visual change" and point at the key doc paths a reviewer
should read end-to-end.
-->

- No visual change / see …

## Testing

<!--
List what you actually ran. Unchecked items need a one-line reason.
As the project grows, replace placeholders with real commands (cargo test, …).
-->

- [ ] Lint / format (command: …) — or N/A because …
- [ ] Typecheck / build (command: …) — or N/A because …
- [ ] Tests (command + which suites) — or N/A because …
- [ ] Manual smoke (steps) — or N/A because …
- [ ] Added or updated tests that would catch this regression — or explained why not
- [ ] Docs/spec consistency walkthrough (which sections)

## Kind

<!-- Exactly one GitHub kind label must match the PR title type. -->

- [ ] `feat` · [ ] `fix` · [ ] `docs` · [ ] `spec` · [ ] `chore` · [ ] `refactor` · [ ] `test` · [ ] `ci`

## Related

- **Milestone:** <!-- M1 … M6 or n/a -->
- **Spec / ADR:** <!-- paths under docs/specs or docs/adr, or n/a -->
- **Issues:** <!-- Closes #N / Refs #N / n/a -->

## Cache impact

<!--
Required for anything that can touch prompts, tool schemas, skills index,
standing memory, or model routing. Otherwise `none`.
-->

`none` | `low` | `medium` | `high` — 

## AI review report

<!--
Summarize the review you ran with a coding agent (or self-review if solo).
Include main risks checked, what was flagged, and what you changed or verified.
This section is required for non-trivial PRs; for tiny typo docs you may write
"Self-review only — <one line why>".
-->

- **Focus areas:**
- **Flags / fixes:**
- **Cross-cutting:** permissions, secrets, cache-stable prefix, platform (macOS/Linux/Windows) if relevant

## Security audit

<!--
Basic security pass (agent-assisted OK). Call out input handling, command
execution, path handling, auth, secrets, dependency, or tool-permission risks.
Write "No security-sensitive surface" only when truly true.
-->

-

## Notes

<!--
Platform quirks, known limitations, follow-ups, intentional non-fixes, how to
regenerate fixtures, credit for prior PRs superseded, etc.
-->

-

## Checklist

- [ ] PR title is Conventional Commits (`type: summary` / `type(scope): summary`)
- [ ] Exactly one **kind** label applied (matches title type)
- [ ] Diff is one meaningful unit (see `docs/contributing/pull-requests.md`)
- [ ] Summary is a narrative with problem + what changed (not only bullets of files)
- [ ] Testing section lists real commands or honest N/A reasons
- [ ] Aligns with `docs/product/NON_GOALS.md` (or includes superseding ADR)
- [ ] No secrets, API keys, or private session dumps
- [ ] I would accept this PR from an external contributor as-is
