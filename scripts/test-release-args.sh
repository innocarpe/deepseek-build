#!/usr/bin/env bash
# Regression: `release.sh <ver>` with no `--desc` must reach the bump step.
#
# The release lane runs under `#!/usr/bin/env bash`, which on macOS is
# /bin/bash 3.2 — and there an empty array expanded under `set -u` is an
# "unbound variable" error (bash 4.4+ tolerates it):
#
#   $ ./scripts/release.sh 6.1.7
#   ./scripts/release.sh: line 73: B_ARGS[@]: unbound variable
#
# `B_ARGS` only ever gains `--desc <note>`, so a release with no `--desc` left
# it empty and died before the bump started. The guard is the bash 3.2 idiom
# `"${B_ARGS[@]+"${B_ARGS[@]}"}"` (same shape as dsbdev.sh).
#
# Cases:
#   1. no --desc      — the real release.sh reaches the bump stub under /bin/bash
#   2. --desc "a b c" — the note stays one argument (the outer quotes matter)
#   3. shape          — the guard on that expansion is still in release.sh
#
# Hermetic: a temp sandbox holds a copy of the real release.sh, and the two
# scripts it calls before the expansion site (the primary-checkout guard, the
# bump) are stubs — nothing is bumped, committed, pushed or published, and no
# network or credentials are touched. Local-only by contract; not wired into CI.
#
# Usage: ./scripts/test-release-args.sh
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
RELEASE="$ROOT/scripts/release.sh"
SYSTEM_BASH="/bin/bash"

PASS=0
FAIL=0
ok()  { printf '  ok   %s\n' "$*"; PASS=$((PASS + 1)); }
bad() { printf '  FAIL %s\n' "$*" >&2; FAIL=$((FAIL + 1)); }
head_() { printf '\n== %s ==\n' "$*"; }

[[ -f "$RELEASE" ]] || { echo "error: $RELEASE missing" >&2; exit 1; }
if [[ ! -x "$SYSTEM_BASH" ]]; then
  echo "skip: $SYSTEM_BASH not found; the empty-array failure mode is macOS bash 3.2" >&2
  exit 0
fi

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

# --- sandbox -----------------------------------------------------------------
mkdir -p "$TMP/sandbox/scripts/lib" "$TMP/bin"
cp "$RELEASE" "$TMP/sandbox/scripts/release.sh"

# The guard is a policy check pinned by test-refuse-primary-checkout.sh; the
# sandbox is not a repo, so it is stubbed and release.sh keeps going.
printf '#!/usr/bin/env bash\nexit 0\n' > "$TMP/sandbox/scripts/lib/refuse-primary-checkout.sh"
# Recording stub: the real bump would edit the tree, so this one writes its
# argv (one argument per line) where the test can read it and stops release.sh.
cat > "$TMP/sandbox/scripts/bump-version.sh" <<'STUB'
#!/usr/bin/env bash
printf '%s\n' "$@" > "$BUMP_ARGS_OUT"
echo "STUB-BUMP reached" >&2
exit 33
STUB
chmod +x "$TMP/sandbox/scripts/lib/refuse-primary-checkout.sh" "$TMP/sandbox/scripts/bump-version.sh"

# release.sh requires gh, node, npm on PATH before it expands anything.
for c in gh node npm; do
  printf '#!/usr/bin/env bash\nexit 0\n' > "$TMP/bin/$c"
  chmod +x "$TMP/bin/$c"
done

run_release() { # <args-outfile> <release args...>; sets RC
  local out="$1"; shift
  RC=0
  BUMP_ARGS_OUT="$out" PATH="$TMP/bin:$PATH" \
    "$SYSTEM_BASH" "$TMP/sandbox/scripts/release.sh" "$@" \
    > "$TMP/stdout" 2> "$TMP/stderr" || RC=$?
}

# ---------------------------------------------------------------------------
head_ "1. no --desc reaches the bump step ($("$SYSTEM_BASH" --version | head -1))"
run_release "$TMP/args1" 8.8.8
if grep -q 'unbound variable' "$TMP/stderr"; then
  bad "release.sh died before the bump: $(cat "$TMP/stderr")"
elif [[ "$RC" -eq 33 && "$(cat "$TMP/args1" 2>/dev/null)" == "8.8.8" ]]; then
  ok "bump called with exactly the version argument"
else
  bad "unexpected outcome rc=$RC: $(cat "$TMP/stderr")"
fi

# ---------------------------------------------------------------------------
head_ "2. --desc keeps a multi-word note as one argument"
run_release "$TMP/args2" 8.8.9 --desc "a note with spaces"
printf '8.8.9\n--desc\na note with spaces\n' > "$TMP/want2"
if [[ "$RC" -eq 33 ]] && cmp -s "$TMP/args2" "$TMP/want2"; then
  ok "argv order and word integrity hold"
else
  bad "bump argv differs (rc=$RC):"
  cat -A "$TMP/args2" >&2 2>/dev/null || true
fi

# ---------------------------------------------------------------------------
head_ "3. the guard is still on the expansion line"
if grep -q 'B_ARGS\[@\]+' "$RELEASE"; then
  ok 'release.sh expands B_ARGS through ${B_ARGS[@]+"${B_ARGS[@]}"}'
else
  bad "release.sh lost the bash 3.2 guard on B_ARGS; a release without --desc will die on macOS"
fi

printf '\n%s passed, %s failed\n' "$PASS" "$FAIL"
[[ "$FAIL" -eq 0 ]] || exit 1
