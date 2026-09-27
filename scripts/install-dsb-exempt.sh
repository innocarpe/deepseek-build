#!/usr/bin/env bash
# Install the DsbExempt runner (macOS): an app bundle whose executable runs a
# command with the app as its responsible process.
#
#   scripts/install-dsb-exempt.sh [--force] [--app-dir DIR] [--bin-dir DIR]
#
# Defaults: /Applications/DsbExempt.app (~/Applications when /Applications is
# not writable) and a ~/.local/bin/dsb-exempt link. /Applications is where the
# Developer Tools + picker opens.
# scripts/dsbdev.sh and scripts/vendor-cargo.sh use the runner when it is
# installed (DSB_EXEMPT=0 turns that off).
#
# Why: macOS assesses every freshly built executable and dylib on its first
# run — 300-590 ms for a small binary, 10 s for the 512 MB debug pager
# (measured 2026-09-28). Developer Tools exempts processes whose responsible
# app is listed there, but Orca spawns its terminal daemon with responsibility
# disclaimed, so nothing in an Orca tab is attributed to Orca and listing Orca
# (or restarting it) changes nothing. The runner is what gets listed; see
# scripts/lib/dsb-exempt.c.
#
# One manual step after the first install: System Settings > Privacy &
# Security > Developer Tools > add DsbExempt.app and keep it switched on.
# The bundle is ad-hoc signed, so that entry is tied to this exact binary: an
# installed app built from the same source is left alone, and --force (a
# rebuild) means adding it again.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC="$ROOT/scripts/lib/dsb-exempt.c"
ID="dev.wooseong.dsb-exempt"
app_dir=/Applications
[[ -w /Applications ]] || app_dir="$HOME/Applications"
bin_dir="$HOME/.local/bin"
force=0

while (($#)); do
  case "$1" in
    --force) force=1 ;;
    --app-dir) app_dir="${2:?--app-dir needs a value}"; shift ;;
    --bin-dir) bin_dir="${2:?--bin-dir needs a value}"; shift ;;
    -h | --help) sed -n '2,9p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
    *) echo "install-dsb-exempt: unknown argument: $1" >&2; exit 2 ;;
  esac
  shift
done

if [[ "$(uname -s)" != Darwin ]]; then
  echo "install-dsb-exempt: macOS only (Developer Tools is a macOS setting)" >&2
  exit 2
fi

app="$app_dir/DsbExempt.app"
exe="$app/Contents/MacOS/dsb-exempt"

if ((!force)) && [[ -x "$exe" ]] && cmp -s "$SRC" "$app/Contents/Resources/dsb-exempt.c"; then
  echo "install-dsb-exempt: $app is already built from this source; left alone (its Developer Tools entry stays valid)"
else
  if [[ -e "$app" ]]; then
    echo "install-dsb-exempt: rebuilding $app — add it under Developer Tools again afterwards" >&2
  fi
  tmp="$(mktemp -d)"
  trap 'rm -rf "$tmp"' EXIT
  /usr/bin/clang -O2 -Wall -Wextra -o "$tmp/dsb-exempt" "$SRC"
  mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
  cp "$tmp/dsb-exempt" "$exe"
  cp "$SRC" "$app/Contents/Resources/dsb-exempt.c"
  cat >"$app/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleIdentifier</key><string>$ID</string>
  <key>CFBundleName</key><string>DsbExempt</string>
  <key>CFBundleDisplayName</key><string>DsbExempt</string>
  <key>CFBundleExecutable</key><string>dsb-exempt</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>1.0.0</string>
  <key>CFBundleVersion</key><string>1</string>
  <key>LSUIElement</key><true/>
  <key>LSMinimumSystemVersion</key><string>13.0</string>
</dict>
</plist>
EOF
  codesign --force --sign - --identifier "$ID" "$app" 2>/dev/null
  echo "install-dsb-exempt: built $app"
fi

mkdir -p "$bin_dir"
ln -sf "$exe" "$bin_dir/dsb-exempt"
"$exe" --check

cat <<EOF

Once, by hand: System Settings > Privacy & Security > Developer Tools >
add $app (drag it in, or + then Cmd-Shift-G) and keep it switched on.
  open "x-apple.systempreferences:com.apple.preference.security?Privacy_DevTools"
EOF
