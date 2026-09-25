# Grok Build sync runbook

**Status:** Normative procedure for refreshing `third_party/grok-build/`
**Skill:** [`skills/grok-sync`](../../skills/grok-sync/SKILL.md) · **Layout:**
[GROK_VENDOR.md](../architecture/GROK_VENDOR.md) · **ADR:**
[0008](../adr/0008-grok-build-base.md) · **Ledger:**
[UPSTREAM_SYNC_LEDGER.md](../product/UPSTREAM_SYNC_LEDGER.md)

The vendored Grok Build tree is the product's runtime base. Upstream ships
near-daily and this product falls behind whenever nobody syncs, so syncing is a
recurring unit of work, not a one-off. This runbook is the procedure; the skill
is the agent-facing checklist; the ledger is what past syncs learned.

---

## 0. Why this is not `rsync` anymore

The refresh procedure in `GROK_VENDOR.md` describes `rsync -a --delete` plus
re-applying the patch series under `patches/grok-build/`. **That works only
while the overlay is small enough for upstream not to touch the same lines.**

Measured on the 1.0.0 → 1.0.41 gap:

| Reading | Value |
|---|---|
| Overlay size vs the pin | 134 files, ~5.3k changed lines + ~3.3k new-file lines |
| Patches that still apply | **0 of 13** |

When the patches conflict, a raw tree swap would silently drop the overlay:
the DeepSeek status line, the theme skins, the prompt identity, the sampling
mapping. So the merge is the procedure above a size threshold — see §2.

---

## 1. Establish ground truth

```sh
./scripts/grok-sync-inventory.sh
```

Prints the pin, the upstream commit it resolves to, upstream HEAD, the release
delta, the file/line gap, and a clustered inventory of every changelog bullet in
the covered range. Use `--bullets` for the full list, `--json` for automation.

Then confirm two things the script cannot:

```sh
# Does the tree on disk still equal the pinned commit plus the known overlay?
git -C third_party/grok-build ...        # the tree is not a git repo; compare trees
mkdir -p /tmp/base && git -C "$UP" archive "$PIN_COMMIT" | tar -x -C /tmp/base
diff -rq /tmp/base third_party/grok-build -x target -x .git | wc -l   # ≈ overlay size

# Do the carried patches still apply?
./scripts/apply-grok-build-patches.sh --check
```

A tree diff in the low hundreds with a mostly-clean patch check means the small
path (§2) is still viable. Anything larger, or any conflict, means the merge.

---

## 2. Choose the method

| Method | When | Cost |
|---|---|---|
| **Patch re-apply** | `apply-grok-build-patches.sh --check` passes | minutes |
| **Three-way merge** | Patches conflict, or the tree drifted | hours |

**Three-way merge** is the general case:

| Role | Tree |
|---|---|
| merge base | the upstream commit the tree was pinned to |
| ours | `third_party/grok-build/` as it stands |
| theirs | the new upstream commit |

Work in a scratch clone outside the product tree; the product tree is only
replaced once the merge is reconciled. Conflicts are design decisions: for each,
decide whether the change is upstream's to take or the overlay's to keep, and
record anything surprising in the ledger.

---

## 3. Classify every changelog bullet

Read all changelogs in the range (`--bullets` prints them). Give each bullet one
verdict and a reason:

**Take** — carry it, with a dsb regression.
**Hold** — right idea, wrong time; name what would change the answer.
**Reject** — against `NON_GOALS.md` or `HARNESS_PHILOSOPHY.md`; name which.
**N/A** — xAI-hosted surface with no dsb meaning.

Two rules that are not negotiable:

1. **"Upstream has it" is not a reason.** The ladder is L1 Deep Code > L2
   Reasonix > L3 Grok, and **L3 never overrides L1/L2** (`SOURCES.md`). A change
   that breaks snippet-edit semantics, prefix-cache stability, or permission
   honesty is a Reject however nice it looks.
2. **Breaking changes need explicit verdicts.** A removed parameter or changed
   default can contradict a dsb spec; the spec wins until revised on purpose.

---

## 4. Gates

```sh
./scripts/build-grok-pager.sh check
./scripts/test-owner-bar.sh
./scripts/check-path-a-linkage.sh
./scripts/test-heart-regression.sh
./scripts/test-grok-vendor-offline.sh
./scripts/test-product-offline.sh
./scripts/check-semver.sh
```

Vendored builds are **serial across all worktrees** (cold build 30–60+ min).
Check `pgrep -fl 'grok-build/target'` first. A gate that did not run is not
green — the PR's Testing section states commands and real outcomes.

---

## 5. Artifacts

| Artifact | Audience |
|---|---|
| `docs/product/UPSTREAM_SYNC_LEDGER.md` | the next sync session |
| `docs/product/CHANGELIST_<VER>.md` | the owner — one file, plain language |
| `docs/research/grok-build.md` | the durable record; the sync matrix |
| `patches/grok-build/` | regenerated series, or empty when the overlay is in-tree |
| `docs/architecture/GROK_VENDOR.md` | updated overlay tables |

## 6. PR shape

One PR per sync, `chore(vendor): refresh grok-build to <version>`. It is large
by nature; the narrative bar still applies — Problem (the gap, measured), What
changed (the method and the overlay), Testing honesty (vendor check + gates),
Security (no new egress / identity / telemetry enablement), Notes (what was held
or rejected, and why).
