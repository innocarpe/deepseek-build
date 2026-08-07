#!/usr/bin/env bash
# DeepSeek Build tab icon for iTerm2 (macOS) — manual check/remove.
#
# The `dsb` CLI auto-installs this mapping on first launch (logo embedded in
# the binary), so no setup is needed after install. This script is for:
#   - `check`  — verify the iTerm2 mapping is present
#   - `remove` — uninstall (only our files/entries; never touches other
#                iTerm2 settings)
#   - `install` — manual re-install (also useful before the first CLI run)
#
# iTerm2 (Tahoe tab style, macOS 15+) renders a per-process icon in each tab
# from a process-name -> icon mapping (graphic_icons.json), colored by
# graphic_colors.json. When the DeepSeek Build agent runs, its foreground
# process name is `deepseek-build-agent`, so iTerm2 shows the official
# DeepSeek whale logo in the tab — no border or background.
#
# Files written under iTerm2's Application Support dir:
#   ~/Library/Application Support/iTerm2/graphic_deepseek.png
#   ~/Library/Application Support/iTerm2/graphic_icons.json   (merged)
#   ~/Library/Application Support/iTerm2/graphic_colors.json  (merged)
#
# Usage:
#   ./scripts/install-iterm-tab-icon.sh            # install (idempotent)
#   ./scripts/install-iterm-tab-icon.sh check      # report state
#   ./scripts/install-iterm-tab-icon.sh remove     # uninstall
#   ./scripts/install-iterm-tab-icon.sh --help
#
# Requires: bash, python3 (macOS ships /usr/bin/python3).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ASSET="$ROOT/crates/dsb-cli/assets/graphic_deepseek.png"

APP_SUPPORT="${HOME}/Library/Application Support/iTerm2"
ICON_FILE="graphic_deepseek.png"
ICON_JSON="graphic_icons.json"
COLOR_JSON="graphic_colors.json"

# Logical icon name (prefixes the image file: graphic_<name>.png).
LOGICAL_NAME="deepseek"
# Process names the agent can appear as in the terminal tab.
COMMANDS='["deepseek-build-agent", "deepseek-build", "dsb", "xai-grok-pager"]'
# Brand blue; iTerm2 tints the tab icon monochrome with this color.
COLOR="#4D6BFE"

usage() {
  sed -n '2,30p' "$0" | sed 's/^# \{0,1\}//'
}

require_asset() {
  if [[ ! -f "$ASSET" ]]; then
    echo "install-iterm-tab-icon: missing asset: $ASSET" >&2
    exit 1
  fi
}

# Merge a key into a tolerant JSON object file. iTerm2's bundled JSON files
# use trailing commas (invalid strict JSON), so strip them before parsing.
merge_json_key() {
  local path="$1" key="$2" value="$3"
  python3 - "$path" "$key" "$value" <<'PYEOF'
import json, re, sys
path, key, value = sys.argv[1], sys.argv[2], sys.argv[3]
if __import__("os").path.exists(path):
    with open(path, encoding="utf-8") as f:
        text = f.read()
    text = re.sub(r",\s*([}\]])", r"\1", text)
    try:
        data = json.loads(text) if text.strip() else {}
    except json.JSONDecodeError as e:
        print(f"install-iterm-tab-icon: failed to parse {path}: {e}", file=sys.stderr)
        sys.exit(1)
else:
    data = {}
if not isinstance(data, dict):
    data = {}
data[key] = json.loads(value)
with open(path, "w", encoding="utf-8") as f:
    json.dump(data, f, indent=2, ensure_ascii=False)
    f.write("\n")
print(f"install-iterm-tab-icon: updated {path} ({key})")
PYEOF
}

# Remove a key from a tolerant JSON object file; deletes the file if empty.
remove_json_key() {
  local path="$1" key="$2"
  python3 - "$path" "$key" <<'PYEOF'
import json, os, re, sys
path, key = sys.argv[1], sys.argv[2]
if not os.path.exists(path):
    sys.exit(0)
with open(path, encoding="utf-8") as f:
    text = f.read()
text = re.sub(r",\s*([}\]])", r"\1", text)
try:
    data = json.loads(text) if text.strip() else {}
except json.JSONDecodeError:
    data = {}
if not isinstance(data, dict):
    data = {}
if key in data:
    del data[key]
    if data:
        with open(path, "w", encoding="utf-8") as f:
            json.dump(data, f, indent=2, ensure_ascii=False)
            f.write("\n")
        print(f"install-iterm-tab-icon: removed {key} from {path}")
    else:
        os.remove(path)
        print(f"install-iterm-tab-icon: removed empty {path}")
else:
    print(f"install-iterm-tab-icon: {key} not present in {path} (ok)")
PYEOF
}

cmd_install() {
  require_asset
  mkdir -p "$APP_SUPPORT"
  cp "$ASSET" "$APP_SUPPORT/$ICON_FILE"
  echo "install-iterm-tab-icon: copied $ICON_FILE -> $APP_SUPPORT/$ICON_FILE"

  merge_json_key "$APP_SUPPORT/$ICON_JSON" "$LOGICAL_NAME" "$COMMANDS"
  merge_json_key "$APP_SUPPORT/$COLOR_JSON" "$LOGICAL_NAME" "\"$COLOR\""

  echo
  echo "Done. iTerm2 picks up the tab icon on the next session; if a tab is"
  echo "already open, start a new tab or run 'dsb' again. To uninstall:"
  echo "  ./scripts/install-iterm-tab-icon.sh remove"
}

cmd_check() {
  local icon_ok="missing" icons_ok="missing" colors_ok="missing"
  [[ -f "$APP_SUPPORT/$ICON_FILE" ]] && icon_ok="present"
  [[ -f "$APP_SUPPORT/$ICON_JSON" ]] && icons_ok="present"
  [[ -f "$APP_SUPPORT/$COLOR_JSON" ]] && colors_ok="present"
  echo "iTerm2 tab icon for DeepSeek Build:"
  echo "  $ICON_FILE        : $icon_ok ($APP_SUPPORT/$ICON_FILE)"
  echo "  $ICON_JSON mapping: $icons_ok"
  echo "  $COLOR_JSON color : $colors_ok"
  if [[ "$icon_ok" == "present" && "$icons_ok" == "present" && "$colors_ok" == "present" ]]; then
    echo "Status: installed"
    exit 0
  fi
  echo "Status: not installed"
  exit 1
}

cmd_remove() {
  remove_json_key "$APP_SUPPORT/$ICON_JSON" "$LOGICAL_NAME"
  remove_json_key "$APP_SUPPORT/$COLOR_JSON" "$LOGICAL_NAME"
  if [[ -f "$APP_SUPPORT/$ICON_FILE" ]]; then
    rm "$APP_SUPPORT/$ICON_FILE"
    echo "install-iterm-tab-icon: removed $APP_SUPPORT/$ICON_FILE"
  fi
  echo "install-iterm-tab-icon: uninstalled."
}

case "${1:-install}" in
  install)  cmd_install ;;
  check)    cmd_check ;;
  remove)   cmd_remove ;;
  -h|--help|help) usage ;;
  *) echo "install-iterm-tab-icon: unknown command: $1" >&2; usage >&2; exit 1 ;;
esac
