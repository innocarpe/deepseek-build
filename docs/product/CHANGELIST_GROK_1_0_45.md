# What changed with Grok Build 1.0.45

The vendored base moved from **1.0.41** to **1.0.45** on 2026-10-01.
`SOURCE_REV` is `559751fdcec02d413e4c57c8832ab275e4f44980`.
This file is the owner-facing note for that sync. It is not a product release.

## Fixed

- A paste that did not come from the clipboard no longer attaches a leftover image.
- Restoring a stashed draft leaves the cursor at the end of the text.
- Pressing Ctrl+C twice while writing a plan comment cancels the comment.
- Minimal mode reprints history after a resize, and MCP prompts show up there.
- Inline images are drawn again after a full repaint.
- Two edits of the same file in one parallel batch run one after the other. Unrelated tools still run together.
- The subagent wait line counts the subagents the parent is actually blocked on.

## New, when the model or the host has it

- macOS sandbox enforcement can use Seatbelt (`sandbox-exec`).
- A model that advertises more than one context window can be switched with `/context-window`, or from `/model` before the effort picker.
- A plugin or config custom agent can be chosen in `spawn_subagent`.
- An MCP server can read a token file on every request.
- A selected model can show a notice banner above the prompt. The welcome screen reserves those rows.

## Not a DeepSeek Build surface

- The footer label for xAI smart-auto's actually served model.
- `grok update` printing a WinGet command. This product updates through npm.

## Left where it was

Deep Code `v0.4.2`, Reasonix `c9daeddf`, and DeepSeek Harness `0.2.0-rc.2` all moved after the 2026-09-25 reads. They are not vendored trees. The new commits are PLUS routing, the Electron studio, and the desktop harness, which `SOURCES.md` already leaves.
