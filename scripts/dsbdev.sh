#!/usr/bin/env bash
# Build the vendored Grok pager for one worktree and open dsb on it.
#
#   dsbdev.sh [--release] [--no-run] [--jobs N] [<worktree>] [-- <dsb args>...]
#
# <worktree> defaults to $PWD (any directory inside a checkout of this repo).
#
# Each worktree builds into its own target, third_party/grok-build/target
# (git-ignored, removed with the worktree). A shared CARGO_TARGET_DIR does not
# work for this: cargo names a path package's artifacts by its path relative to
# the workspace root, so all worktrees write the same files, overwrite each
# other, and one worktree can get another's code from a no-op build (mtime-only
# freshness). DSBDEV_TARGET_DIR overrides the target.
#
# A worktree's first debug build is seeded from the sibling worktree target
# whose recorded sources differ least (scripts/lib/dsbdev.py), so only the
# crates whose sources differ are compiled. Every build prints how many units
# it compiled and how long it took, and writes that to
# <target>/<profile>/dsbdev/last-build.json.
#
# Jobs: --jobs N, else DSBDEV_JOBS, else CARGO_BUILD_JOBS, else 4 (memory is
# the scarce resource on the maintainer's machine).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
HELPER="$ROOT/scripts/lib/dsbdev.py"

usage() {
  sed -n '2,5p' "$0" | sed 's/^# \{0,1\}//'
}

release=0
run=1
jobs="${DSBDEV_JOBS:-${CARGO_BUILD_JOBS:-4}}"
dir=""
dsb_args=()
while (($#)); do
  case "$1" in
    --release) release=1 ;;
    --no-run) run=0 ;;
    --jobs) jobs="${2:?--jobs needs a value}"; shift ;;
    --jobs=*) jobs="${1#*=}" ;;
    -h | --help) usage; exit 0 ;;
    --) shift; dsb_args=("$@"); break ;;
    -*) echo "dsbdev: unknown flag: $1" >&2; usage >&2; exit 2 ;;
    *) dir="$1" ;;
  esac
  shift
done

dir="$(cd "${dir:-$PWD}" && pwd -P)"
if [[ -f "$dir/third_party/grok-build/SOURCE_REV" ]]; then
  vendor="$dir/third_party/grok-build"
elif [[ -f "$dir/SOURCE_REV" && -d "$dir/crates/codegen" ]]; then
  vendor="$dir"
elif top="$(git -C "$dir" rev-parse --show-toplevel 2>/dev/null)" \
  && [[ -f "$top/third_party/grok-build/SOURCE_REV" ]]; then
  vendor="$top/third_party/grok-build"
else
  echo "dsbdev: no third_party/grok-build under $dir" >&2
  exit 1
fi

export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:/usr/local/bin:$PATH"
for c in cargo python3 git; do
  command -v "$c" >/dev/null 2>&1 || { echo "dsbdev: $c is not on PATH" >&2; exit 1; }
done

# macOS assesses every freshly built executable and dylib on its first run
# (the pager: 10 s). Under the DsbExempt runner, listed under Developer Tools,
# cargo and dsb skip that (scripts/lib/dsb-exempt.c). DSB_EXEMPT=0 turns it
# off; a path picks that runner.
exempt=()
if [[ "$(uname -s)" == Darwin && "${DSB_EXEMPT:-}" != 0 ]]; then
  if [[ -n "${DSB_EXEMPT:-}" && "$DSB_EXEMPT" != 1 ]]; then
    runner="$DSB_EXEMPT"
  else
    runner="$(command -v dsb-exempt || true)"
    for app in /Applications "$HOME/Applications"; do
      [[ -n "$runner" ]] || { [[ -x "$app/DsbExempt.app/Contents/MacOS/dsb-exempt" ]] && runner="$app/DsbExempt.app/Contents/MacOS/dsb-exempt"; } || true
    done
  fi
  if [[ -n "$runner" && -x "$runner" ]]; then
    exempt=("$runner")
  fi
fi

target="${DSBDEV_TARGET_DIR:-$vendor/target}"
profile=debug
build_args=(-p xai-grok-pager-bin)
if ((release)); then
  profile=release
  build_args+=(--release)
fi
state="$target/$profile/dsbdev"
mkdir -p "$state"

if ((!release)); then
  python3 "$HELPER" seed --vendor "$vendor" --target "$target" \
    || echo "dsbdev: seeding failed; building without it" >&2
fi

started="$(python3 -c 'import time; print(time.time())')"
if ((!release)); then
  python3 "$HELPER" snapshot --vendor "$vendor" --out "$state/snapshot.json"
fi

echo "dsbdev: building ($profile, -j $jobs${exempt[0]:+, under DsbExempt}) — $vendor" >&2
set +e
(
  cd "$vendor" \
    && CARGO_TARGET_DIR="$target" ${exempt[@]+"${exempt[@]}"} cargo build "${build_args[@]}" -j "$jobs" \
      --message-format=json-render-diagnostics
) | python3 "$HELPER" summarize --out "$state/last-build.json" --started "$started"
statuses=("${PIPESTATUS[@]}")
set -e
if ((statuses[0] != 0)); then
  echo "dsbdev: build failed" >&2
  exit "${statuses[0]}"
fi

if ((!release)); then
  python3 "$HELPER" record --vendor "$vendor" --target "$target" \
    --snapshot "$state/snapshot.json" --started "$started" --last-build "$state/last-build.json" \
    || echo "dsbdev: could not record the seed manifest" >&2
fi

bin="$target/$profile/xai-grok-pager"
if [[ ! -x "$bin" ]]; then
  echo "dsbdev: no build output at $bin" >&2
  exit 1
fi
if ((!run)); then
  echo "$bin"
  exit 0
fi
echo "dsbdev: running dsb with DEEPSEEK_BUILD_AGENT_BIN=$bin" >&2
DEEPSEEK_BUILD_AGENT_BIN="$bin" exec ${exempt[@]+"${exempt[@]}"} dsb ${dsb_args[@]+"${dsb_args[@]}"}
