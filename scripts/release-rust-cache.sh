#!/usr/bin/env bash
# Pack or unpack the directories a release build compiles into, so
# seed-release-cache.yml can save them as a default-branch Actions cache.
#
# A tag workflow can read caches from its own ref and from the default branch.
# The cache it saves is invisible to the next tag. The tag build therefore
# only restores; this archive is how the result is re-saved on main.
#
# Usage: release-rust-cache.sh pack|unpack ARCHIVE
# ARCHIVE is gzip (.tar.gz). Pack refuses archives larger than 6 GiB and
# exits 0 after deleting them, so a huge target cannot fail the release.
set -euo pipefail

usage() {
  echo "usage: release-rust-cache.sh pack|unpack ARCHIVE" >&2
  exit 2
}

cmd="${1:-}"
archive="${2:-}"
[[ -n "$cmd" && -n "$archive" ]] || usage

ROOT="${RELEASE_RUST_CACHE_ROOT:-$(cd "$(dirname "$0")/.." && pwd)}"
CARGO_HOME="${CARGO_HOME:-$HOME/.cargo}"
MAX_BYTES=$((6 * 1024 * 1024 * 1024))

link_if_dir() {
  local src="$1" name="$2" stage="$3"
  if [[ -d "$src" ]]; then
    ln -s "$src" "$stage/$name"
    echo "$name"
  fi
}

pack() {
  local stage names=() name
  stage="$(mktemp -d)"
  trap 'rm -rf "$stage"' RETURN
  while IFS= read -r name; do
    [[ -n "$name" ]] && names+=("$name")
  done < <(
    link_if_dir "$CARGO_HOME/registry" cargo-registry "$stage"
    link_if_dir "$CARGO_HOME/git" cargo-git "$stage"
    link_if_dir "$ROOT/target" workspace-target "$stage"
    link_if_dir "$ROOT/third_party/grok-build/target" grok-target "$stage"
  )
  if [[ ${#names[@]} -eq 0 ]]; then
    echo "release-rust-cache: nothing to pack" >&2
    exit 1
  fi
  # -h dereferences the stage symlinks. incremental/ is empty under
  # CARGO_INCREMENTAL=0 and is the part rust-cache would drop anyway.
  tar -chzf "$archive" -C "$stage" --exclude incremental "${names[@]}"
  local size
  size="$(wc -c < "$archive" | tr -d ' ')"
  echo "release-rust-cache: packed ${size} bytes -> $archive"
  if [[ "$size" -gt "$MAX_BYTES" ]]; then
    echo "release-rust-cache: archive exceeds ${MAX_BYTES} bytes; not uploading" >&2
    rm -f "$archive"
    exit 0
  fi
}

unpack() {
  local stage name src dest
  [[ -f "$archive" ]] || { echo "release-rust-cache: missing $archive" >&2; exit 1; }
  stage="$(mktemp -d)"
  trap 'rm -rf "$stage"' RETURN
  tar -xzf "$archive" -C "$stage"
  copy_tree() {
    local name="$1" dest="$2"
    [[ -d "$stage/$name" ]] || return 0
    mkdir -p "$dest"
    # Trailing /. copies the directory's contents, including dotfiles.
    cp -a "$stage/$name/." "$dest/"
    echo "release-rust-cache: unpacked $name -> $dest"
  }
  copy_tree cargo-registry "$CARGO_HOME/registry"
  copy_tree cargo-git "$CARGO_HOME/git"
  copy_tree workspace-target "$ROOT/target"
  copy_tree grok-target "$ROOT/third_party/grok-build/target"
}

case "$cmd" in
  pack) pack ;;
  unpack) unpack ;;
  *) usage ;;
esac
