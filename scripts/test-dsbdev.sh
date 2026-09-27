#!/usr/bin/env bash
# Test for scripts/dsbdev.sh on a throwaway repository: four path crates laid
# out like the vendored tree (third_party/grok-build, package
# xai-grok-pager-bin, bin xai-grok-pager), real cargo, several git worktrees.
#
# Offline: the fixture has no registry dependencies. It never touches this
# repository's targets or ~/.cache; everything lives in a temp dir.
#
# Why (measured 2026-09-27): with one CARGO_TARGET_DIR shared by worktrees,
# a worktree got another worktree's binary from a no-op build, and a new
# worktree recompiled the whole workspace. Each case checks the unit set a
# build compiled (<target>/debug/dsbdev/last-build.json) and what the binary
# prints, so "fast" never passes by running someone else's code.
#
#   1. cold      — first build in worktree A compiles all four crates
#   2. no-op     — a second build compiles nothing
#   3. seed      — new worktree B, same sources: seeded from A, compiles nothing
#   4. old edit  — worktree C edits `other` with a year-2000 mtime before its
#                  first build: only other + the bin compile, C's code runs
#   5. no donor  — seeding does not overwrite a target that has a manifest
#   6. stale     — A rebuilds `core` outside dsbdev after its manifest; D seeds
#                  from A, recompiles core's chain, and runs D's code, not A's
#   7. checkout  — a build script watching the checkout's git HEAD (like the
#                  pager's version string) is not cloned: worktree E, one commit
#                  ahead of A, prints E's commit
#
# Usage: ./scripts/test-dsbdev.sh
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SCRIPT="$ROOT/scripts/dsbdev.sh"
export PATH="$HOME/.cargo/bin:$PATH"

PASS=0
FAIL=0
TMP="$(cd "$(mktemp -d)" && pwd -P)"
trap 'rm -rf "$TMP"' EXIT

ok()  { printf '  ok   %s\n' "$*"; PASS=$((PASS + 1)); }
bad() { printf '  FAIL %s\n' "$*" >&2; FAIL=$((FAIL + 1)); }
head_() { printf '\n== %s ==\n' "$*"; }

for c in cargo git python3; do
  command -v "$c" >/dev/null 2>&1 || { echo "skip: $c not on PATH" >&2; exit 0; }
done
if [[ "$(uname -s)" != Darwin ]]; then
  echo "skip: seeding needs APFS clonefile (macOS)" >&2
  exit 0
fi

# --- fixture ---------------------------------------------------------------
A="$TMP/a"
V=third_party/grok-build
mkdir -p "$A/$V/crates/codegen"
crate() { # name deps... ; lib.rs returns "<name>" joined with its deps
  local name="$1"; shift
  local dir="$A/$V/crates/codegen/$name"
  mkdir -p "$dir/src"
  {
    printf '[package]\nname = "%s"\nversion = "0.1.0"\nedition = "2021"\n\n[dependencies]\n' "$name"
    for d in "$@"; do printf '%s = { path = "../%s" }\n' "$d" "$d"; done
  } >"$dir/Cargo.toml"
  printf 'pub fn name() -> String { "%s".to_string() }\n' "$name" >"$dir/src/lib.rs"
}
crate core
crate mid core
crate other
mkdir -p "$A/$V/crates/codegen/xai-grok-pager-bin/src"
cat >"$A/$V/crates/codegen/xai-grok-pager-bin/Cargo.toml" <<'EOF'
[package]
name = "xai-grok-pager-bin"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "xai-grok-pager"
path = "src/main.rs"

[dependencies]
core = { path = "../core" }
mid = { path = "../mid" }
other = { path = "../other" }
EOF
cat >"$A/$V/crates/codegen/xai-grok-pager-bin/src/main.rs" <<'EOF'
fn main() { println!("{} {} {}", core::name(), mid::name(), other::name()); }
EOF
cat >"$A/$V/Cargo.toml" <<'EOF'
[workspace]
resolver = "2"
members = ["crates/codegen/*"]
EOF
echo fixture >"$A/$V/SOURCE_REV"
echo "$V/target/" >"$A/.gitignore"
git -C "$A" init -q -b main
git -C "$A" -c user.name=t -c user.email=t@t add -A
git -C "$A" -c user.name=t -c user.email=t@t commit -q -m fixture

build() { # worktree -> prints the binary path; build log to $TMP/log
  "$SCRIPT" --no-run --jobs 2 "$1" 2>"$TMP/log"
}
compiled() { # worktree -> sorted compiled package names, space-separated
  python3 -c 'import json,sys; print(" ".join(sorted(json.load(open(sys.argv[1]))["compiled"])))' \
    "$1/$V/target/debug/dsbdev/last-build.json"
}
expect_compiled() { # label worktree expected
  local got
  got="$(compiled "$2")"
  if [[ "$got" == "$3" ]]; then ok "$1: compiled [${got}]"; else bad "$1: compiled [${got}], want [$3]"; cat "$TMP/log" >&2; fi
}
expect_output() { # label binary expected
  local got
  got="$("$2")"
  if [[ "$got" == "$3" ]]; then ok "$1: runs \"$got\""; else bad "$1: runs \"$got\", want \"$3\""; fi
}

# --- cases -----------------------------------------------------------------
head_ "1. cold"
bin="$(build "$A")"
expect_compiled cold "$A" "core mid other xai-grok-pager-bin"
expect_output cold "$bin" "core mid other"
if [[ -f "$A/$V/target/debug/dsbdev/manifest.json" ]]; then ok "manifest recorded"; else bad "no manifest"; fi

head_ "2. no-op"
build "$A" >/dev/null
expect_compiled no-op "$A" ""

head_ "3. seed, same sources"
git -C "$A" worktree add -q "$TMP/b" -b b
bin="$(build "$TMP/b")"
expect_compiled seed "$TMP/b" ""
expect_output seed "$bin" "core mid other"
grep -q "seeded 4 units" "$TMP/log" && ok "seed log names 4 units" || { bad "seed log"; cat "$TMP/log" >&2; }

head_ "4. old-mtime edit before the first build"
git -C "$A" worktree add -q "$TMP/c" -b c
f="$TMP/c/$V/crates/codegen/other/src/lib.rs"
printf 'pub fn name() -> String { "other-c".to_string() }\n' >"$f"
touch -t 200001010000 "$f"
bin="$(build "$TMP/c")"
expect_compiled old-edit "$TMP/c" "other xai-grok-pager-bin"
expect_output old-edit "$bin" "core mid other-c"
build "$TMP/c" >/dev/null
expect_compiled old-edit-then-no-op "$TMP/c" ""

head_ "5. a target with a manifest is not reseeded"
before="$(stat -f %m "$TMP/c/$V/target/debug/dsbdev/manifest.json")"
python3 "$ROOT/scripts/lib/dsbdev.py" seed --vendor "$TMP/c/$V" --target "$TMP/c/$V/target" 2>"$TMP/log"
after="$(stat -f %m "$TMP/c/$V/target/debug/dsbdev/manifest.json")"
if [[ "$before" == "$after" && ! -s "$TMP/log" ]]; then ok "seed is a no-op"; else bad "seed touched a seeded target"; cat "$TMP/log" >&2; fi

head_ "6. donor rebuilt after its manifest"
printf 'pub fn name() -> String { "core-a".to_string() }\n' >"$A/$V/crates/codegen/core/src/lib.rs"
(cd "$A/$V" && CARGO_TARGET_DIR="$A/$V/target" cargo build -q -p xai-grok-pager-bin)
git -C "$A" stash -q # A's sources match its manifest again; its target does not
rm -rf "$TMP/b/$V/target" "$TMP/c/$V/target" # leave A as the only donor
git -C "$A" worktree add -q "$TMP/d" -b d
bin="$(build "$TMP/d")"
expect_output stale "$bin" "core mid other"
got="$(compiled "$TMP/d")"
case " $got " in
  *" core "*) ok "stale: core recompiled [${got}]" ;;
  *) bad "stale: core not recompiled [${got}]"; cat "$TMP/log" >&2 ;;
esac

head_ "7. a build script that watches the checkout's git HEAD"
# Like xai-grok-pager's build.rs: the commit in the version string comes from
# absolute paths in the donor's checkout, so that unit must not be cloned.
mkdir -p "$A/$V/crates/codegen/stamp/src"
printf '[package]\nname = "stamp"\nversion = "0.1.0"\nedition = "2021"\n' >"$A/$V/crates/codegen/stamp/Cargo.toml"
cat >"$A/$V/crates/codegen/stamp/build.rs" <<'EOF'
use std::process::Command;
fn git(a: &[&str]) -> String {
    String::from_utf8(Command::new("git").args(a).output().unwrap().stdout).unwrap().trim().to_string()
}
fn main() {
    println!("cargo:rerun-if-changed={}/HEAD", git(&["rev-parse", "--absolute-git-dir"]));
    let common = git(&["rev-parse", "--path-format=absolute", "--git-common-dir"]);
    println!("cargo:rerun-if-changed={common}/{}", git(&["rev-parse", "--symbolic-full-name", "HEAD"]));
    println!("cargo:rustc-env=COMMIT={}", git(&["rev-parse", "--short", "HEAD"]));
}
EOF
printf 'pub fn commit() -> &%sstatic str { env!("COMMIT") }\n' "'" >"$A/$V/crates/codegen/stamp/src/lib.rs"
printf 'stamp = { path = "../stamp" }\n' >>"$A/$V/crates/codegen/xai-grok-pager-bin/Cargo.toml"
cat >"$A/$V/crates/codegen/xai-grok-pager-bin/src/main.rs" <<'EOF'
fn main() {
    if std::env::args().nth(1).as_deref() == Some("commit") {
        println!("{}", stamp::commit());
        return;
    }
    println!("{} {} {}", core::name(), mid::name(), other::name());
}
EOF
git -C "$A" -c user.name=t -c user.email=t@t add -A
git -C "$A" -c user.name=t -c user.email=t@t commit -q -m stamp
build "$A" >/dev/null
rm -rf "$TMP/d/$V/target" # leave A as the only donor
git -C "$A" worktree add -q "$TMP/e" -b e
git -C "$TMP/e" -c user.name=t -c user.email=t@t commit -q --allow-empty -m "e's own commit"
bin="$(build "$TMP/e")"
want="$(git -C "$TMP/e" rev-parse --short HEAD)"
got="$("$bin" commit)"
if [[ "$got" == "$want" ]]; then ok "checkout-bound: prints e's commit $got"; else bad "checkout-bound: prints $got, want e's $want"; cat "$TMP/log" >&2; fi
grep -q "1 checkout-bound" "$TMP/log" && ok "seed log counts 1 checkout-bound unit" || { bad "seed log"; cat "$TMP/log" >&2; }
expect_compiled checkout-bound "$TMP/e" "stamp xai-grok-pager-bin"

printf '\n%d passed, %d failed\n' "$PASS" "$FAIL"
((FAIL == 0))
