## Summary

<!--
What changed AND why (not a file list).
One coherent unit — see docs/contributing/pull-requests.md §3 and examples.md.
-->

-

## Kind

<!-- Exactly one GitHub kind label must match the PR title type. -->

- [ ] `feat` · [ ] `fix` · [ ] `docs` · [ ] `spec` · [ ] `chore` · [ ] `refactor` · [ ] `test` · [ ] `ci`

## Related

- **Milestone:** <!-- M1 … M6 or n/a -->
- **Spec / ADR:** <!-- paths under docs/specs or docs/adr, or n/a -->
- **Issues:** <!-- Closes #N / Refs #N / n/a -->

## Test plan

<!-- How should a reviewer verify this? Commands, doc walkthrough, CI jobs. -->

- [ ]
- [ ]

## Cache impact

<!-- Required if agent/runtime/prompt/tool schema might change. Else: none. -->

`none` | `low` | `medium` | `high` — 

## Checklist

- [ ] PR title is Conventional Commits (`type: summary` / `type(scope): summary`)
- [ ] Exactly one **kind** label applied (and matches title type)
- [ ] Diff is one meaningful unit (no unrelated drive-by changes)
- [ ] Aligns with `docs/product/NON_GOALS.md` (or includes superseding ADR)
- [ ] No secrets, API keys, or private session dumps
- [ ] I read [docs/contributing/pull-requests.md](../docs/contributing/pull-requests.md) for this change type
