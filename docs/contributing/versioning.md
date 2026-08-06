# Versioning (SemVer harness — fail-close)

**Normative.** Every agent and human working in this repo must treat product
and package versions as **Semantic Versioning 2.0.0** only.

Spec: <https://semver.org/>

---

## 1. Canonical form

| Rule | Correct | Forbidden |
|------|---------|-----------|
| Full triple | `0.1.0`, `1.0.0`, `2.3.1` | `1.0`, `v1`, `1`, `one point oh` |
| Optional `v` prefix in **git tags only** | tag `v1.0.0` | tag `v1.0`, `1.0` |
| Pre-release | `1.0.0-rc.1`, `0.2.0-alpha.1` | `1.0-rc`, `rc1` alone as “the version” |
| Build metadata | `1.0.0+build.7` (rare) | using metadata as the only version story |

**In prose, PRs, ultragoal goals, milestones, chat, and docs:** always write the
**full** `MAJOR.MINOR.PATCH` (and pre-release label when needed).

### Examples

| Bad (do not write) | Good |
|--------------------|------|
| “ship 1.0” | “ship **1.0.0**” |
| “after v1” | “after **1.0.0**” or “after the **1.x** line” (line family OK; release ID still full SemVer) |
| “bump to 0.2” | “bump to **0.2.0**” |
| cargo/npm version `1.0` | workspace / package version **`1.0.0`** |

Calling a **major line** “the 1.x series” is fine. Calling a **release** `1.0`
is not.

---

## 2. Where the version lives

| Surface | Source of truth |
|---------|-----------------|
| Rust workspace | root `Cargo.toml` → `[workspace.package] version` (e.g. `0.1.0`) |
| CLI `--version` | same via `clap` / `CARGO_PKG_VERSION` |
| npm (when published) | `package.json` `"version"` **must match** workspace SemVer for that release |
| GitHub Release / tag | `vMAJOR.MINOR.PATCH` (leading `v` allowed **only** on tags) |
| CHANGELOG | headings use `## MAJOR.MINOR.PATCH` |

Single product version for a release: do not ship CLI `0.1.0` and npm `0.1.1`
for the same intended release without an explicit ADR.

---

## 3. Meaning (product defaults)

Until **1.0.0**, the public contract may break between minors with a `BREAKING CHANGE`
footer / changelog note. Prefer not to; if you must, document migration.

| Range | Meaning for this project |
|-------|---------------------------|
| `0.y.z` | Development / preview; API and CLI may change |
| `1.0.0` | First **installable, dogfoodable** release meeting the release checklist in [releases](./releases.md) (when present) or M6 + packaging DoD |
| `1.x.y` | Compatible evolution of the 1.x CLI/user contract |
| `2.0.0` | Intentional break of that contract |

Do **not** market or tag **1.0.0** until install path + documented smoke work.

---

## 4. Agent harness rules (mandatory)

1. Never write bare `1.0` / `0.2` / `v1` as a **version identifier** in commits, PR titles/bodies, specs, ADRs, gates, or ultragoal evidence.  
2. Ultragoal / milestone language: use `1.0.0`, not “v1 product”.  
3. When bumping version, update **all** of: workspace `Cargo.toml`, lockfile if needed, npm `package.json` (if exists), and mention full SemVer in the PR.  
4. PR kind for version bumps alone: `chore` (or `chore(release)`).  
5. If a contributor uses `1.0` in a PR body, **correct to `1.0.0`** before merge.

---

## 5. Quick check

```bash
# Workspace version must match MAJOR.MINOR.PATCH
rg -n '^version = "[0-9]+\.[0-9]+\.[0-9]+' Cargo.toml

# Reject incomplete forms in the version field
! rg -n '^version = "[0-9]+\.[0-9]+"' Cargo.toml
```

Optional helper: `scripts/check-semver.sh` (when present).
