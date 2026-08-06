# GitHub labels

Canonical catalog: [`.github/labels.json`](../../.github/labels.json)

```bash
./scripts/sync-labels.sh
```

## Kind labels (required on every non-draft PR)

Exactly **one**:

`feat` · `fix` · `docs` · `spec` · `chore` · `refactor` · `test` · `ci`

Must match the Conventional Commits **type** in the PR title.  
Ready PRs must carry exactly one kind label (process / review harness — not CI).

## Size (optional)

`size/S` · `size/M` · `size/L` — soft signal; prefer S/M. See [pull-requests.md](../contributing/pull-requests.md).

## Area / priority / triage

Optional. Prefer `area/*` when the change is scoped. `priority/*` is for maintainers.

| Family | Examples |
|--------|----------|
| Area | `area/provider`, `area/cache`, `area/docs`, `area/infra` |
| Priority | `priority/p0` … `priority/p3` |
| Triage | `bug`, `enhancement`, `needs-design`, `ready`, `blocked` |
| Process | `milestone-aligned`, `good first issue`, `help wanted` |

## Sync after catalog edits

1. Edit `.github/labels.json`  
2. Merge PR  
3. Run `./scripts/sync-labels.sh` (or include a note for the maintainer)
