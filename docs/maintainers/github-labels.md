# GitHub labels

Canonical catalog: [`.github/labels.json`](../../.github/labels.json)

Apply with:

```bash
./scripts/sync-labels.sh
# or: python3 scripts/sync_labels.py
```

## Kind labels (required on every PR)

Exactly one primary kind:

`feat` · `fix` · `docs` · `spec` · `chore` · `refactor` · `test` · `ci`

## Area / priority / triage

Optional. Prefer area labels when the change is scoped. Priority is for maintainers.
