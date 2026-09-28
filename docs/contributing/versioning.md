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

## 1b. Major product lines (PRD map)

| Line | Meaning | PRD |
|------|---------|-----|
| **1.x** | Scaffold / legacy thin agent | [PRD-v1.md](../product/PRD-v1.md) |
| **2.x** | Grok base + DeepSeek product shell | [PRD-v2.md](../product/PRD-v2.md) |
| **3.x** | Heart fusion L1+L2 under Grok shell — **owner-bar NOT MET** | [PRD-v3.md](../product/PRD-v3.md) |
| **4.x** | L3 productization — **owner-bar NOT MET** | [PRD-v4.md](../product/PRD-v4.md) |
| **5.x** | **Owner-bar complete product** (`5.0.0`) + vision-complete (`5.5.0`) — **current shipped line** | [PRD-v5.md](../product/PRD-v5.md) |
| **6.x** | **Base refresh**: Grok Build `1.0.0` → `1.0.41`, overlay re-derived; continuation `6.1.0` = DeepSeek-native depth | [PRD-v6.md](../product/PRD-v6.md) |

Index: [docs/product/versions/README.md](../product/versions/README.md).  
New majors require a **PRD-vN** + versions index update **before** coding the train.
Minors do **not** get a new PRD unless behavior identity shifts — they extend the
line's existing PRD (§Rules 1 of the index).

## 1c. Bump level — MINOR / PATCH / none (fail-close)

**Normative.** Which digit moves is read from what shipped, not chosen by
habit. [`scripts/next-version.sh`](../../scripts/next-version.sh) renders the
judgment from the git graph alone (no `gh`, no network), and
[`scripts/release.sh`](../../scripts/release.sh) refuses a version below it
unless the caller names a reason.

### The distribution surface

A change reaches users only through what the release artifacts carry: the
`dsb` / `deepseek-build` CLI, the agent binary the npm `postinstall` fetches,
and the npm package itself. In a merge, that surface is exactly these paths:

| Path | Ships as |
|------|----------|
| `crates/` | the `dsb` / `deepseek-build` CLI and agent crates |
| `third_party/` | the vendored Grok Build tree the agent binary is built from |
| `npm/` | the npm wrapper, postinstall, version checks |
| `package.json` | the npm manifest (`bin`, `files`, `postinstall`) |
| `Cargo.toml`, `Cargo.lock` | the workspace and lock state both binaries build from |

Everything else — `scripts/`, `skills/`, `docs/`, `.github/`, tests, root
markdown — is repository harness: a merge touching only those paths changes
nothing a user installs.

### Levels

| Level | Judgment since the last release tag |
|-------|-------------------------------------|
| **MINOR** | a `feat/` merge changed the distribution surface |
| **PATCH** | the surface changed without a `feat/` (fix, perf, refactor, chore, …) |
| **none** | only repository harness changed — no release is owed |
| **MAJOR** | unchanged from §1b: a declared product line, gated by the version log. Never computed from the merges. |

Raising above the judgment (`none` → PATCH → MINOR → MAJOR) is a product
decision and needs no flag. Shipping below it needs the override below.

### Inputs

- **the merges**: the first-parent merges on the release ref since the newest
  `v*` tag merged into it (the ref is `origin/main`, then `main`, then `HEAD`;
  `--ref` names another). A merge whose subject is not the GitHub shape is
  labelled `(unparsed)`; a direct commit on the ref is labelled `(direct)`.
- **the type**: the `<type>/` prefix of the branch in the merge subject
  (`Merge pull request #N from <owner>/<type>/<slug>`, [branches.md](./branches.md)).
  An unreadable type is never `feat`.
- **the paths**: `git diff --name-only <merge>^1 <merge>`. A merge with no
  readable type, and a direct commit, floor at PATCH when they touch the
  surface.

### The tool

```bash
./scripts/next-version.sh                # per-merge lines, then "proposed: 6.1.11 (PATCH)"
./scripts/next-version.sh --level        # none | patch | minor
./scripts/next-version.sh --version      # 6.1.11; empty when the level is none
./scripts/next-version.sh --check 6.2.0  # exit 0 at or above the judgment,
                                         # 1 below it or not newer than the last tag,
                                         # 2 when no judgment can be made
```

Measured 2026-09-28: run at each of the ten releases' pre-release merge
(published as `6.1.1`–`6.1.10`, renamed `6.2.0`–`6.8.0` below), the tool
proposes MINOR for `6.2.0`–`6.7.0` and `6.8.0` (feat merges on the surface,
e.g. `#321` phone-band-composer) and PATCH for `6.7.1`–`6.7.3`. The ten
shipped as PATCH because no rule chose the digit; that is what this section
replaces.

### Fail-close and the override

`release.sh` runs `--check` before its bump (a fetch of `origin/main` first).
Exit 1 stops the release and prints the deciding merges. Anything else nonzero
means no judgment was rendered — no `v*` tag merged yet (a first release), no
repository, the tool missing — and the release continues with the reason
printed. `--publish-only` skips the check, because the released work would
read as unreleased there.

Ship below the judgment only with the reason; it rides in the release PR body:

```bash
./scripts/release.sh 6.7.0 --level-override "the vendored build-script change is not user-visible"
```

A path rule cannot see intent. Measured example: `#301`
(`feat/dsbdev-worktree-target`) touched
`third_party/grok-build/crates/…/build.rs` — cargo rerun and protoc plumbing —
while its product was a `scripts/` dev tool. The rule counts that as surface;
the override is where a person records why the lower number is right anyway.

Pinned by [`scripts/test-next-version.sh`](../../scripts/test-next-version.sh)
(hermetic: throwaway repos and a fake `gh`; local-only by contract).

### Limits

- The judgment reads `origin/main` as it is when the release runs. A merge
  that lands after the check is outside it; a resumed release re-checks unless
  it is `--publish-only`.
- Tags are immutable with one dated exception: the renumbering below moved
  the ten 6.1.x tags to this rule's numbers at the same commits, leaving the
  npm versions, the assets and the binaries untouched. The rule applies from
  the next release on.

### Renumbering (2026-09-28)

The ten releases the measurement above reads shipped before this rule existed,
so nothing picked their digits. On 2026-09-28 their GitHub tags and release
names were moved to the numbers this rule picks, at the commits they already
had. The npm versions, the release assets and the binaries keep the published
strings — a shipped artifact is not rebuilt or republished for a rename — and
the repository record (the CHANGELOG headings, the version-log rows and prose
that names a release) uses the new number with the published one in
parentheses.

| Published tag | Renamed tag | Commit |
|---------------|-------------|--------|
| `v6.1.1` | `v6.2.0` | `1ef1e36b8c37897807a090a4c089fb9a71204865` |
| `v6.1.2` | `v6.3.0` | `825cd115a2b32bb7fe57e0f1d830085cd92380fc` |
| `v6.1.3` | `v6.4.0` | `c28fad5e76eafc35946cca423c763fa3b02e7387` |
| `v6.1.4` | `v6.5.0` | `83e2af5c80a39d0aad27ccf0670138d1bc532ca1` |
| `v6.1.5` | `v6.6.0` | `72bea610937215ca7e87ecca288e7b3130b471f4` |
| `v6.1.6` | `v6.7.0` | `3635488f21b3d316d763f5c3dedc85430c77ccdf` |
| `v6.1.7` | `v6.7.1` | `0d491b0493f4785f852124dfdc4c986102f3641b` |
| `v6.1.8` | `v6.7.2` | `e6593f254b5093f539a8c18967ed12305c9fec30` |
| `v6.1.9` | `v6.7.3` | `55bf8def886b082fefd5139a6137ba212f4808ff` |
| `v6.1.10` | `v6.8.0` | `278f82ea1b56ffd8bc9b453198695577a46ea191` |

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
| `0.y.z` | Historical pre-1.x development (Wave A–D scaffold train) |
| **`1.0.0` – `1.x.y`** | **Legacy scaffold line** (already published). Thin agent + contracts. **Not** the Grok Build–class product. See [REPLAN_2.0.md](../product/REPLAN_2.0.md). |
| **`2.0.0`** | **First real product**: `dsb` opens Grok Build–class coding agent; Grok open source as base; DeepSeek-native. |
| `2.0.0-alpha.*` / `2.0.0-beta.*` | Optional previews while integrating Grok base |
| `2.x.y` (after 2.0.0) | Compatible evolution of the real product line |

**Important:** Tags **`1.0.0` / `1.1.0` already shipped on npm.** Do not rewrite history. Product success is measured by **`2.0.0` DoD**, not by prior 1.x claims.

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
