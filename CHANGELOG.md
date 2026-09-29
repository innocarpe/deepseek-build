# Changelog

## Unreleased

## 6.9.0 — 2026-09-29

- The `Jump to bottom` chip reads with half a cell of air on each side: the two padding columns paint the chip's colour over the canvas (`▐` left, `▌` right) instead of a full background cell, and both columns stay inside the click target. A wide glyph (Hangul, CJK, emoji) beginning on the chip's last column can no longer show a terminal-default cell immediately right of the chip: the chip repaints that exposed trailing cell in the glyph's own background instead of letting the content's hidden blank reach the terminal.
- A phone-width pane measures the permission, question and elicitation cards at the width they are drawn at. The cards span the frame like the composer, but their bodies were wrapped for the card's height at the inner width, two columns narrower, so a body line that filled the drawn row was counted as two and the card kept a blank row it never drew; the question list's scroll limit used the same narrower total, so a scroll to the end stopped one row above the last option. Desktop panes measure both widths the same and are unchanged.
- On a phone-width pane the turn row above the composer (spinner, elapsed time, `[stop]`) now sits on the footer's edges, one column in from the frame on each side, instead of one column further in on the left and two on the right. The composer's draft starts on the footer's column too, one column closer to the frame edge, on the welcome screen and in a conversation. Desktop widths are unchanged.
- The Tasks pane's right-hand strip — the elapsed time and the `[↗]` / `[✗]` buttons — no longer shows a box behind each glyph when the task label underneath is Korean or another wide (CJK) label. `ratatui` resets the cell after a wide grapheme to `Color::Reset`, and the overlay blanked its strip with a default-styled span before writing fg-only time/button spans, so those slots kept the reset background and the terminal painted its own default behind the glyph. The strip now carries the row's background and modifier: a focused selection keeps its band, a transient search-match invert no longer leaks into it, and the truncation ellipsis sits on the row background.
- On a phone-width pane the top status row (branch, working directory, and the right-hand items) keeps one column of air at each screen edge instead of touching it. Its text now opens on the same column as the transcript's text and closes one column before the right edge; the transcript, task list, and composer band still span the frame, and desktop panes are unchanged.
- The composer opens a new row only for the character that needs it. Typing to the end of a row used to drop the caret onto an empty row below before anything was typed there, and on a phone-width pane the box grew that empty row two columns early, because the band is drawn across the frame but was measured two columns narrower. A full row now keeps the caret right after its last glyph, and the next character — a one-column letter or a two-column Hangul syllable — opens the row and takes the caret with it. A two-column glyph that meets a single free column still moves to the next row, as before. On a phone the permission follow-up and the question's freeform answer are measured the same way. Desktop row sizing is unchanged.
- Resuming or continuing a full-screen session restores its cumulative billing and Path A cache hit/miss totals from `usage.json` before accepting new requests. Older sessions without saved cache history report it as unknown, and sessions with chat history but no usage file report incomplete totals instead of claiming zero. When the legacy main-loop split is unknown, observed post-upgrade main-loop counts also survive repeated resumes without claiming complete history or inferring them from billing calls.
- New-file `write` uses the target repository's `.gitattributes` `eol` policy
  only when its filesystem backend confirms the target is on the host. Other
  backends, including ACP client filesystems, use the platform default because
  their host-path identity and target Git policy are unknown. Creation preserves
  the supplied final-newline choice and reports the normalized content it
  writes. Existing-file edit safety and line-ending behavior are unchanged.
- Completed and cancelled Path A turns can report host-observed changes to supported Rust, Go, and pytest declarations when an in-process structured file write succeeds on host-backed `LocalFs`. Rust attributes and pytest decorators are included in declaration fingerprints. Existing dirty content is the baseline; inconsistent write metadata, client-backed ACP writes, formatting/comment-only changes, and same-signature file moves are omitted. The bounded note and turn record expose only relative path, language, category, and count—never test names, source text, literals, or hashes—and do not claim that tests ran or passed.

## 6.8.3 — 2026-09-28

- The `Jump to bottom` chip keeps its own opaque background and text style while scrolling over styled text, Hangul, or emoji. A wide glyph crossing its left edge can no longer suppress the chip's first cell. One column of background on each side gives the label a little air, and both padding cells hover and click along with the label at every supported width.

## 6.8.2 — 2026-09-28

- The prompt no longer shows Grok subscription quota warnings such as `Weekly limit left`, on the welcome screen or in a conversation, at any pane width or provider/status state. Account balance, cache, model, permission mode, dollar-denominated credit warnings, and the explicit `/usage` summary keep their existing behavior.

## 6.8.1 — 2026-09-28

- A folded prompt echo's last row runs to the edge before its ` …`. It used to stop where word wrapping had broken it: the word that did not fit had already moved to the hidden row, so the row ended early and, on a phone-width pane, the clock filled the gap (`BBB…B …   11:53 PM` for `AAA…A BBB…B CCC…C`). The row now carries on with the text that follows and is cut at the row's edge (`BBB…B CCC…C …`), so a full folded row shows no clock. A wide glyph (Hangul, CJK) that would straddle the cut stays out whole, a cut that lands on a space leaves one space before the ellipsis, and a line break in the prompt still ends the row. The echo keeps its two-row (three on a wide pane) fold. The row is cut by the cells the terminal paints, grapheme by grapheme, so Arabic lam-alef text, an emoji sequence and a tab do not push the ellipsis off the row or end it early; and the clock's room check counts those painted cells too, so a full row of Arabic text (a lam-alef pair is one column to the width table and two painted cells) no longer reads as short and gets the clock drawn over its end.
- `release.sh` waits for the release PR's checks before merging it. The merge fired the instant `gh pr create` returned, while GitHub was still computing mergeability — `6.2.0` (#258, published as `6.1.1`), `6.7.1` (#311, published as `6.1.7`) and `6.8.0` (#323, published as `6.1.10`) answered `GraphQL: Pull Request is not mergeable (mergePullRequest)` and left the PR open for a person to merge, and `6.7.0` (#303, published as `6.1.6`) stopped at the same place with `Base branch was modified`. The script now polls the PR's check rollup (`scripts/lib/pr_checks.py` — every check complete, none failed or cancelled, `MERGEABLE`) for up to `--checks-timeout` (default 3600 s), stops before the merge call when a check fails (printing the failing checks), and exits 1 with the PR and the resume command (`--skip-bump --skip-pr` after merging by hand) when `gh pr merge` itself fails. Pinned by the hermetic `scripts/test-release-pr-wait.sh` (a fake `gh`, a throwaway repo).
- Which digit a release moves is now a judgment of what shipped, not a habit. `scripts/next-version.sh` reads the first-parent merges on `origin/main` since the last tag — the `<type>/` prefix in the merge subject and the paths the merge changed — and proposes MINOR when a `feat/` merge touched the distribution surface (`crates/`, `third_party/`, `npm/`, `package.json`, `Cargo.toml`, `Cargo.lock`), PATCH when the surface changed without a feat, and no release when only the repository harness changed; `release.sh` runs it before the bump (fetching `origin/main` first) and refuses a version below the judgment, printing the deciding merges, unless `--level-override "<reason>"` records why — the reason is carried into the release PR body. Run at each of the ten releases' pre-release merge (`6.2.0`–`6.8.0`, published as `6.1.1`–`6.1.10`) the tool proposes MINOR for `6.2.0`–`6.7.0` and `6.8.0` (feat merges on the surface, e.g. #321 phone-band-composer) and PATCH for `6.7.1`–`6.7.3`, so the patch-digit habit those ten releases show is now a check; the rule is normative in `docs/contributing/versioning.md` §1c and pinned by the hermetic `scripts/test-next-version.sh` (throwaway repos, a fake `gh`, no network).

## 6.8.0 — 2026-09-28

Published to npm as `6.1.10`; the release asset and `dsb --version` carry `6.1.10`.

- On a phone-width pane the composer is a band across the frame in the prompt echo's colour (`bg_light`) instead of a rounded box: no rules, its top and bottom rows cut to three quarters of a row (`▆` above the text, `▂` below), and the text keeps the box's inset. The frame now ends on a two-row footer under it, each row split left and right: balance and cache against the model's full name (`$15.70 cache 24%` · `DeepSeek V4.1 Flash (max)`), then this session's tokens against the permission mode (`48.6k in · 236 out` · `always-approve`). The phone keeps no floor row: the footer's text sits the cell's own leading above the grid's edge. The welcome screen draws the same band and footer (its first row names the build), so the first keystroke's switch to the conversation view moves nothing. Desktop panes keep the boxed composer, the status row and the floor row.
## 6.7.3 — 2026-09-28

Published to npm as `6.1.9`; the release asset and `dsb --version` carry `6.1.9`.

- On a phone-width pane a prompt echo opens folded to two rows whenever its words wrap past two. The fold check counted `ceil(width / columns)` rows per line, a lower bound on word wrapping, so a prompt that word-wraps to three rows (a long English prompt, say) opened expanded at three; it now counts the rows the echo actually wraps to, with the renderer's own prefix and wrap. A prompt pushed before the first frame (a resumed or replayed session), a pass through width 0, and an appearance change such as compact mode keep or re-derive the fold the same way. Tapping still opens the echo, and a second tap folds it back to two rows.
- The release verification guidance asks npm 12 for the install script it blocks. `scripts/release.sh`'s "User verification" block, `scripts/npm-emergency-publish.sh`'s final hint, `skills/release`'s standard cycle and post-publish checklist, and `docs/contributing/release-cycle.md` now print `npm install -g --allow-scripts=@innocarpe/deepseek-build @innocarpe/deepseek-build@<ver>`; the plain `npm i -g` form is denied the `postinstall` on npm 12.0.0+ and still exits 0, leaving the previous agent binary in place (measured 2026-09-27 on npm 12.1.0: `npm warn install-scripts` for the package, then `npm package 6.1.8 and deepseek-build-agent 6.1.7 differ.` from the CLI; the flagged form installed 6.1.8 with all three binaries matching the release tarball). npm 11 and older accept the flag unchanged.

## 6.7.2 — 2026-09-27

Published to npm as `6.1.8`; the release asset and `dsb --version` carry `6.1.8`.

- On a phone-width pane the prompt echo's clock closes its last text row at the band's bottom-right, like a chat bubble, and only when that row leaves room for it; otherwise the echo shows no clock. The clock no longer takes a row of its own above the text (it did whenever the first row was full, the usual case on a phone), so a folded echo stays two text rows. Agent messages, `/btw` replies and wider panes keep their clock where it was.
- A pinned prompt echo no longer loses its last row while you scroll. The pinned header shrinks to a floor as you scroll on, and that floor was the prompt's Truncated height, capped at six rows: it left out a tapped clock's own row, and for a prompt the width-blind fold check keeps expanded while its words wrap past the fold budget, the extra text rows. A pinned prompt paints its whole output, so the header painted more rows than it had: its bottom pad landed on the last text row, and the band showed only under that row's glyphs. The floor is now the prompt's own height, and a prompt's off-screen height estimate counts the rows its text actually wraps to, so a prompt pinned before it was ever on screen gets the same height.
- On a phone-width pane a prompt echo row that ends on a wide glyph (Hangul, CJK) carries its band into the copy gutter too. The gutter copied the transcript's last column, which holds the glyph's trailing half in the terminal's default style, and showed a grey dot between the band and the scrollbar.

## 6.7.1 — 2026-09-27

Published to npm as `6.1.7`; the release asset and `dsb --version` carry `6.1.7`.

- The turn-status spinner centers on the row's activity label. The shared braille frames ink the six-dot cell's top three rows, and phone terminals pin that cell to the top of the line box, so the dots float above the label — measured on the iPhone screenshot that surfaced this: the cell's middle row sat 10px above the label's x-height center, its bottom row was empty, and the visible dot centroid ran 3px high. The turn-status row re-encodes the same rotation one braille row down in eight-dot rows 2-4 (`⠖⠲⢲⢰⣰⣠⣄⣆` for `⠋⠙⠹⠸⠼⠴⠦⠧`), which drops the pattern onto the label's optical center; every other spinner keeps the shared set. A unit test pins the shift (dots 1,2,3 → 2,3,7 and 4,5,6 → 5,6,8) and the cleared top row.
- The focused composer border on DeepSeek Night (classic) steps down from the official blue `#4D6BFE` to `#3248BE` (`DEEPSEEK_BLUE_DIM`) — the tone the maintainer confirmed from the B-vs-D round. On the lifted E1 ground the official blue was the one saturated element left on screen (S 99%, 4.2:1) and pulled the eye ahead of the body; the dim step keeps the focus cue (2.45:1 against `#0E1425` — the idle border measures 1.54) and now matches the dashboard dispatch box and the peek reply, which already focus with `selection_border`. Both DeepSeek Night skins share the value and the extensions-modal focused box follows it; the idle border `#293659`, `DEEPSEEK_BLUE`/`_BRIGHT` and the accent fields stay as they are. Decision record and previews: `docs/product/THEME_CLASSIC_READABILITY_2026-09-27.md`.
- The frame keeps one row of its own background under the bottom status row at every width, so the status text no longer sits on the frame's bottom edge. It takes only a row the rest of the frame leaves free, so it is the first row to give way on a tight pane; short terminals (16 rows or fewer) drop it with the other margins, and a theme on the terminal's own background leaves the row blank.
- Sessions' vendored cargo runs in the worktree that runs it. `scripts/vendor-cargo.sh <cargo args>` pins `CARGO_TARGET_DIR` to this worktree's `third_party/grok-build/target`, names an inherited value it ignores, seeds a cold target once from the sibling worktree whose recorded sources differ least, caps jobs (default 4, `--jobs` overrides) and refuses to start while another worktree's vendored build is in flight (`--allow-concurrent` goes through the memory gate as a second build at 2 jobs); subcommands that compile nothing and `--release`/`--profile` runs skip the seed. Measured 2026-09-27 with `crates/common/xai-message-delivery-core` in two checkouts at one commit sharing one target: the second checkout's `cargo test` compiled nothing, ran the first checkout's binary (`REPRO_MARKER=a`, `left: "a" right: "b"`, exit 101); through the wrapper it ignored the inherited target, compiled its own crate and passed (`REPRO_MARKER=b`, 19 passed). On the real vendored tree a cold worktree seeded 1397 units / 11,024 files in 3.7 s, and its first `cargo test -p xai-grok-pager --lib --no-run` at `-j 4` reused 375 units and compiled 931 — the test-only dependency closure a bin-build manifest does not cover (`criterion`, `insta`, `wiremock`, …) — in 357 s with another session's build in flight.
- The one shared vendored build target is gone. `scripts/vendor-build.sh status` reads every worktree's `third_party/grok-build/target` and reports each state, lock holder/waiter, elapsed, command and worktree plus the host facts and the memory-gate verdict (exit 1 while busy; `--target DIR` reads one target), `clone` is retired — its product was a target whose fingerprints belong to another worktree, the contamination vector this change closes — with `prune` left to clear its copies, and `run -- <cmd>` keeps the memory gate while no longer picking a target directory. `scripts/build-grok-pager.sh`, `scripts/cache-guard.sh`, `scripts/test-grok-vendor-offline.sh` and the `test-path-a-vc006/vc010/vc011/vc012/vc015` family pin the checkout's own target through `scripts/lib/vendor-target.sh` (inside the vendored subshell for the family, so its top-level `dsb-cli` build keeps the repo workspace's target), so an exported `CARGO_TARGET_DIR` cannot reintroduce the sharing. `AGENTS.md` and `skills/worktree-dispatch` carry the rule and the measured failure; the CI `vendor build queue` job runs the three hermetic tests (`test-vendor-build.sh` 57 checks, `test-vendor-cargo.sh` 34, `test-vendor-target.sh` 9).

## 6.7.0 — 2026-09-27

Published to npm as `6.1.6`; the release asset and `dsb --version` carry `6.1.6`.

- The classic DeepSeek Night theme keeps its dark-navy ground and lifts the glyph ramp so the phone screen reads: body `#C4C8DC → #D7D9E7` (APCA Lc 73 → 83), tool rows `#6E748C → #868DAC` (WCAG 3.97 → 5.60, above the AA floor), emphasis `#E8EAF6 → #F9F9FC`. The emphasis/body CIE L* gap holds at Δ11 — the rejected first candidate collapsed it to Δ5 — and the tool-row gray keeps its blue cast. Only the blue-tinted ramp moves: the neutral skin, the shared accents, `DEEPSEEK_BLUE*` and the field wiring are unchanged. Two tests pin the contrast floors on `bg_base` and the L* hierarchy, and `docs/product/THEME_CLASSIC_READABILITY_2026-09-27.md` keeps the candidate rounds, the evidence images and the edge-case checklist for the next touch-up.
- On a phone-width pane, the status bar, the task list and the scrollback span the screen width. The prompt echo's background runs from the left edge up to the scrollbar — through the column the transcript keeps for the copy chip — and to the right edge when no scrollbar is drawn, while its text stays in the same place; the scrollbar column holds only the bar, so scrolling no longer leaves stray marks there. Scrolling a prompt into its pinned position preserves the fractional pad rows across every left gutter column instead of leaving a dark notch.
- A phone pane pins the prompt echo you are scrolled into in compact mode too. `/compact-mode` (offered by the small-screen tip) and the auto-compact that the on-screen keyboard triggers used to drop the pinned echo, so it scrolled away; desktop panes keep compact mode's scrolling echo.
- On a phone-width pane a thinking or tool rail sits one column in from the screen edge again (the transcript now spans the width, and the rail had moved onto the edge), and the prompt echo keeps its thin fractional pad rows in compact mode instead of its text touching the band's top and bottom. The echo's text keeps one column of air inside each side of its band (it started three columns in), and on a phone pane the top pad sits above the turn clock's row, so the clock joins the text instead of floating over a dark strip. Every turn clock on a phone pane now closes on the transcript's last column: the copy gutter beside it is its one column of air, the same as the text's on the left.
- A phone pane keeps its phone layout when the text is pinched smaller. The phone density now reads the pane's grid as well as its width: the measured Orca iOS pane is 55×41 at 100% text and 110×82 at 50% — a pinch scales both grid axes, so the portrait shape holds — and the pane no longer flips to the desktop rhythm the moment it passes 60 columns. Desktop grids stay landscape (80×24, 120×40, 179×60), a wide portrait pane (a rotated monitor) stays desktop, and wrapping, folds and fits keep following the real column count.
- On a phone-width pane the turn clock no longer reserves a gutter on every wrapped line. A finished turn paints the short time in leftover space on the first line, or on one right-aligned row above the body when that line is full. A turn that is still streaming paints nothing, so the paragraph width stays put until it finishes. The next full line skips that row when the previous painted clock is the same minute. Tapping the clock toggles the long form for that turn only; if the long form does not fit, it uses the row above the body and the body is not rewrapped. Wider panes keep the right-edge clock and take its columns from the first line only.
- Every completed unit ends with a **debrief** before its report. `skills/session-debrief` counts what the unit produced, classifies it (incident, playbook, concept, decision, gap, or none), and records the WC-worthy parts with the global `w-conatus` skill without asking again; a unit with nothing transferable reports `no record needed` with the reason. `skills/session-unit` runs it after the merge and before the report, the report carries the WC paths and commit SHAs, and the clauses are pinned in the description windows and in the local close check (`scripts/lib/session_close.py`). This repository's personal work creates no company HQ records.
- The debrief runs before the worktree cleanup: `skills/session-unit`'s order is merge → debrief → cleanup, `skills/worktree-dispatch` §4 says why (the debrief counts with `git -C "$WT"`), and the done checklist, the anti-patterns and AGENTS.md carry the same order. The first cut cleaned up first, which would have failed the debrief once the tree was gone.
- The closeout report carries context and evidence: `skills/session-unit`'s Report asks for the result paragraph (what changed and why), separates observation from inference with the command/`file:line`/SHA behind each measured sentence and no invented before → after numbers, keeps the exact-field paste (PR, CI, merge fields, checks), names the owned worktree by exact path with `removed: true`/`false` — a tree kept in place says whether the removal condition did not stand or it was retained on purpose, with the reason — gives the not-done list a reason per item — another session's file or worktree left alone included — and names the checked range with what it showed, clean or dirty, calling nothing wider clean; runtime or deployed state appears when it matters to the outcome and was measured, with its source and time. The shape is the general half of the HQ closeout report (result narrative, sourced measurements, the scope it did not touch); the HQ record systems stay in HQ. `AGENTS.md` §One session, one unit summarizes it and `scripts/lib/session_close.py` pins the phrases.

## 6.6.0 — 2026-09-27

Published to npm as `6.1.5`; the release asset and `dsb --version` carry `6.1.5`.

- Selecting a prompt echo draws the selection bracket around the echo band itself: the corners sit on the band's own top and bottom pad rows instead of the blank rows above and below it, so the bracket hugs the echo with only its one-row pad inside. Blocks with no band of their own — tool rows, thoughts, groups — keep the box they have today.
- Padding is measured in the minimum unit everywhere, in columns. The echo's pad rows on a phone-width pane are painted as two eighths of a row (`▂` / `▆`), so the band shows the one column of air its left/right gutters have instead of the 2.15 columns a whole row would spend; wider panes keep the configured full-height pad. The composer is the top border, one text row and the bottom divider at every width (no blank row above or below the draft) — one pad row there was the "weird blank space" the report showed. Every message's clock now ends on one column inside the band's right edge (the echo's and an agent message's used to differ by one), and each block spends its own left/right chrome: a rail keeps one column of air before its body (`┃ text`), a block with neither a rail nor a band starts its text on the accent column and wraps to one column inside the frame, and the prompt echo keeps the band's own gutters. On a phone-width pane the prompt echo's band now runs edge to edge (its outer left/right margins are band, the way Claude Code's and Codex CLI's prompt areas look), its own inner gutters unchanged so the text does not move; the pinned echo paints the same fraction pads as the one in the flow, so its height no longer jumps as it reaches the top; every message's clock keeps two columns of air before it; and the frame ends with one blank row under the bottom status row.
- On a phone-width pane, tapping an opened prompt echo folds it back to two lines. The second tap counts even inside the double-click window, and it counts on any row of that echo, not only the first line. The first tap still opens the echo in place and keeps it at the top of the scrollback. Wider panes still open a folded prompt on a double-click.
- Scrolled up in a conversation, the scrollback's last row carries a centered `Jump to bottom (click) ↓` chip; clicking or tapping anywhere on it returns to the bottom and re-engages follow mode. The chip widens down to `Jump to bottom ↓` and then `▼` when the pane cannot hold the full phrase with a column of air either side, and it is absent at the bottom, under the `none` indicator setting, and while the block viewer or the scrollback search is open. It sits inside the scrollback rather than the gap row the old `▼` used, so it covers neither the prompt nor the phone bottom band; the `▲` response-top arrow is unchanged.

## 6.5.0 — 2026-09-26

Published to npm as `6.1.4`; the release asset and `dsb --version` carry `6.1.4`.

- The prompt echo takes one minimal pad on all four sides. The accent rail's column is the whole left gutter again (`block_pad_left` defaults back to 0, so every block's text starts one column inside the band and two from the pane edge; the right gutter stays at two columns), the echo's own right pad is one column where other blocks keep two, and the echo keeps one pad row above and below at phone widths instead of dropping them. Turn timestamps reserve the string's own width (8 columns, no two-column lead-in) and close flush on the entry's right edge, so the time sits at the band's end and the body wraps two columns wider. The phone bottom band's cache label reads `cache 88%` instead of `c88%`; the fuller label makes the widest money strings (`$1234.56` + `cache 100%`) drop the reasoning-effort suffix while the permission mode stays whole. One blank floor row sits under the bottom status row instead of the frame ending flush on it.
- `dsb -c` / `dsb --continue` continues the most recent full-screen session for the current workspace, forwarding the vendored pager's `--continue` (the flag Claude Code users reach for). It conflicts with `--resume` and with line-mode `--session`, and `run` / `chat` / `repl` reject it like the other TUI-only flags. The four READMEs and user-guide 03 document it.

## 6.4.0 — 2026-09-26

Published to npm as `6.1.3`; the release asset and `dsb --version` carry `6.1.3`.

- The text band's gutters are symmetric and every text block sits on the same band. Block text starts one column after the accent rail (the rail's right side gets its gutter back) and stops two columns before the pane edge — the right pad mirrors the rail's column plus the left pad — so both gutters measure three columns on a 55-column pane. The composer insets its text by one cell on all four sides (border row/column plus one inset cell above, one below before the divider), the turn status row drops the blank rows that surrounded it and takes the scrollback's left/right insets, and the queue pane's left inset follows the same formula. The default `[scrollback.layout]` pads move `block_pad_left 0 → 1` and `block_pad_right 0 → 2`, and `TurnStatusConfig::gap` defaults to false. Desktop panes move identically: the pads and insets are width-independent.
- On a phone-width pane the input box keeps a plain bottom rule, and the balance, cache hit, model and permission share the one row under it. Cost and cache sit on the left (`$10.77 c94%`); the model and permission sit on the right (`V4.1 Flash (max) · always-approve`). A leading `DeepSeek ` is dropped on that row, the cache marker is `cN%` rather than `cache N%`, and a trailing reasoning-effort parenthesis is dropped only when keeping it would cut the permission mode off. The keyboard-hint row stays gone. Panes whose prompt area is wider than 60 columns keep the model on the box border and `cache N%` on its own row.
- On a phone-width pane, a collapsed prompt echo shows up to two rows, with an ellipsis when the prompt is longer. Wider panes keep the three-line budget.
- A session on OpenRouter sticks to one upstream provider. The request body now carries the session id as OpenRouter's `session_id` sticky-routing key, so a session (subagents included) keeps the provider that holds its prefix cache instead of re-billing the whole prompt at the input rate whenever the router moves. Requests to every other endpoint are byte-identical.
- `CI grok test` no longer flakes on `commit_the_source_reaches_but_does_not_name_lets_the_snapshot_go`: git's automatic maintenance is spawned detached by default and deletes `.git/objects/maintenance.lock` as its run ends, and the vendored safety helper's copy now skips a listed file the source has already dropped instead of panicking on `NotFound`.

## 6.3.0 — 2026-09-26

Published to npm as `6.1.2`; the release asset and `dsb --version` carry `6.1.2`.

- On a phone-width pane, tap the one-line prompt echo to read the whole prompt in place. The opened echo stays at the top of the scrollback instead of jumping back to the latest line. Tap that first line to fold it again; tapping the rest of the text does not. Wider panes still open a folded prompt on a double-click.
- The frame is flush at every width. The outer margin no longer spends blank
  rows (the status bar is the first row and the bottom status row is the last),
  the horizontal margin is the one column the selection border draws into, and
  blocks carry no side pads: an entry's text starts at the accent column on a
  55-column phone pane and on a 180-column desktop pane alike. The pads stay
  configurable under `[scrollback.layout]` in `pager.toml`.
- Phone-only render gates are unchanged. At or below 60 columns the submitted
  prompt echo still folds to one row and drops its decorative `❯`, and the
  default block vpad goes away; desktop widths keep the three-line echo, the
  arrow and the block vpad.

## 6.2.0 — 2026-09-26

Published to npm as `6.1.1`; the release asset and `dsb --version` carry `6.1.1`.

- dsb no longer receives or displays xAI/Grok announcements (the shell strips them at the settings boundary; the pager never merges the remote layer).
- A worktree is created together with its owning session: `AGENTS.md` and `skills/worktree-dispatch` §1 say the create and the agent tab are one action (grok/codex/claude `--agent`, dsb sent into the launcher shell), the owning session carries the unit through PR, CI, merge and `skills/release`, and a session that keeps driving a new worktree by path from another checkout has not handed off. `scripts/check-worktree-ownership.sh` reads that state live — a worktree with dirty work and no agent tab is a defect (exit 1, `--json`) — with a hermetic fixture test in `scripts/test-check-worktree-ownership.sh`.
- A unit's final report closes with the **Session disposition** block (`skills/session-unit` Report, one line in `AGENTS.md` §One session, one unit): close now · more in this session · to hand off. Finishing and not saying so, or asking `혹시 …를 더 볼까요?`, is the measured shape the block replaces; the phrases are pinned needle by needle in `scripts/test-session-close.py`.
- On a phone-width pane (60 columns or fewer) the model and mode label (`DeepSeek V4.1 Flash (max) · always-approve`) leaves the input box's bottom border and takes its own row below the box, and the keyboard-hint row (`Enter:send · Opt+Enter:newline · Shift+Tab:mode`, and every state-specific variant of it) is gone: the label row is the row the hints used to occupy, so the bottom stack is exactly as tall as before. The DeepSeek balance and cache-hit row is unchanged, and panes above 60 columns render exactly as they did.
- The shared vendored Grok target is a queue, and it is now visible and gated: `scripts/vendor-build.sh status` reads the cargo file lock (`target/debug/.cargo-lock`) and reports holders, waiters, elapsed, command and worktree per process (exit 1 while busy) plus host free/swap/load5m with the memory-gate verdict; `clone` copies the target copy-on-write into `~/.cache/dsb-vendor-targets/<slug>` so a session builds without waiting (registry deps stay fresh; a copy that collides with a live writer is retried with backoff and then finished by a per-file clonefile walk that skips files the other build removed); `run -- <cmd>` passes a free queue through as-is and on a busy queue starts a second build only when the memory gate passes, in a clone with `CARGO_BUILD_JOBS=2` (refused otherwise: status printed, exit 1); `prune` frees idle clones. `AGENTS.md` and `skills/worktree-dispatch` say the queue and the deadlock rule — cargo called from inside a cargo build or test on a shared target is a real lock cycle, and nothing under `third_party/grok-build` does it today.
- CI caches are scoped to `main`: pull-request jobs restore without saving, `grok test` runs in its own workflow so a docs push cannot cancel it, npm tests run when npm files change, and the publish wait no longer holds a macOS runner.
- `release.sh` refuses to run in the primary checkout, so the main worktree cannot end up on a release branch; the guard has its own test.
- The first release cache is seeded on `main` and checked in the same run: a dispatch job builds the release agent on main, and the next job fails unless the restore reports a cache hit with at most 40 crates recompiled.
- A new worktree opens one tab: the launcher shell is the tab, `dsb` is sent into it, and a leftover prompt tab is closed only after the agent screen shows it is working.
- The vendored suite is aligned with this build: theme and layout goldens follow the DeepSeek picker order and title, an unstatable runtime socket stays a lexical deny path, `install_npm` names `@innocarpe/deepseek-build`, the grove identity and redirect-protocol tests sit behind the off-by-default `grove-identities` feature, and the request/log check accepts the one agent-message label it used to reject. `CI grok test` runs the full `cargo test --workspace`; the name skip list (the twelve pager goldens the suite already failed, plus four grove tests, with `--no-fail-fast`) is gone.
- The one-tab launch rule is pinned by a hermetic test, so an edit that drops those sentences fails the release verify job.
- The four user-facing READMEs describe the DeepSeek Harness (dsh) as evidence rather than a fourth layer, with the cache lifecycle and the six session behaviors; `Everyday use` now names the 60-column prompt fold, the configurable status line, and Flash image attachments.
- `skills/worktree-dispatch` and `GROK_VENDOR.md` carry the measured vendor-build fast path (a shared `CARGO_TARGET_DIR` built the pager in 2 min 56 s, and `cargo check -p <pkg> --all-targets` is the gate before a full test build), and `skills/session-unit` opens the PR as soon as the evidence exists because the longest checks run on GitHub.

## 6.1.0 — 2026-09-26

- DeepSeek-native depth: a cache miss names which assembled document moved, the session logs a cache total, a request that diverges from the log fails the turn, and a stable-body change appends instead of rewriting the cached prefix. CHANGELIST_6_1_0.md.

## 6.0.2 — 2026-09-26

- Path A names the component that moved on a cache epoch change, the session line keeps DeepSeek cache misses, and the vendored cache guard is scored on Path A request bytes. The 6.1.0 depth train was not taken.

## 6.0.1 — 2026-09-26

- A submitted prompt on a phone-width pane is a one-row band. The fold is measured at the width the text actually wraps at, so a long one-line prompt no longer stays expanded and skips the one-row budget. At or below 60 columns the echo also drops its two blank pad rows and the decorative `❯` on both the echo and the input box; `$ `, `↻  `, `? ` and `! ` stay, because those say what kind of turn or mode this is. Widths above 60 keep today's three-line budget, the padding and the arrow, and a prompt folded by a resize unfolds again when the pane widens.
- `npm install -g` on npm 12 no longer looks finished when the agent was
  never downloaded. npm 12.0.0 denies dependency install scripts unless the
  installer opts in, and still prints `added 1 package` (this machine: npm
  12.1.0). The published tarball does not contain `npm/native-bin/`. With no
  binary on disk, `dsb` exits 127 and prints the command that works:
  `npm install -g --allow-scripts=@innocarpe/deepseek-build @innocarpe/deepseek-build`.
  The form npm itself prints, with no package spec, exits `ENOENT package.json`.
  Plain `npm rebuild -g @innocarpe/deepseek-build` stays blocked. When an
  older agent is already installed, the shim still runs it and warns — it
  does not stamp `DEEPSEEK_BUILD_VERSION`, so `--version` reports that older
  binary — and the warning names the same command. The postinstall retry
  uses the rebuild form that includes `--allow-scripts`. A script that does
  run and fails still exits 1. npm 11.20.0 runs the script with or without
  the flag. The READMEs and `docs/user-guide/05-npm.md` show the working
  command.
- A published release no longer prints `[alpha]` because `version.json`
  still names an older stable pointer. `6.0.0` with `stable_version`
  `5.7.0` reported `deepseek-build 6.0.0 (…) [alpha]`. That file is an
  updater cache, not this product's channel: install does not write it.
  A release SemVer omits the suffix. A pre-release (`6.1.0-alpha.1`)
  still uses the upstream comparison, and that comparison itself is
  unchanged.
- The npm package and `deepseek-build-agent` are one install. postinstall
  of an older package replaced a newer agent (a `5.7.0` package over a
  `6.0.0` agent). The installer now leaves the newer agent in place
  unless `DEEPSEEK_BUILD_ALLOW_DOWNGRADE=1`. The wrapper no longer stamps
  its package version onto the agent, and it warns when the two versions
  differ. `dsb --version` reports the native binary.
- Pasting an image into a pane on a remote host now attaches it. A pasted image
  path is resolved by the host the pager runs on, and a terminal that works
  this way uploads the bytes to that host first — an Orca SSH pane writes the
  temp file to the remote `$TMPDIR` and pastes the remote path. An earlier
  `is_ssh` early-return skipped the classifier there, so the paste landed as
  literal path text and the attachment was impossible over SSH.
- A cache epoch change now says *what* moved, not just *that* something did.
  The stable prefix is built once per process and reused, so the only moment a
  conversation's prefix actually moves is when a later process rebuilds it —
  a resumed session. dsb stores each session's prefix shape beside its
  transcript and, on resume, prints `prefix_change=<axes>` with a detail line
  naming the entries (`tools.added=mcp__demo__pong`,
  `skills.removed=old-skill`, `environment.cwd`). A session that carries no
  stored shape — any file written before this change — logs nothing rather
  than guess, and an unchanged prefix logs `none`. The shape is observational:
  `stable_prefix_bytes` and every existing epoch are byte-identical, pinned by
  a golden test that fails loudly when a change would invalidate live caches.
- A session now reports what its cache did over the whole conversation, not
  just the last turn. `dsb run` and the REPL print
  `cache_session=hit=<n>,miss=<n>,rate=<pct>,reported=<n>,unreported=<n>` once
  per turn, summing the prompt tokens of every response that carried cache
  fields. A response that carried none moves `unreported` and nothing else —
  counting it as a miss would invent a number the provider never sent — and a
  session whose responses all lack cache fields prints no line at all rather
  than a `rate=na` that reads like a measurement. The counter is per
  conversation and never reset by a turn; it is stored with the session, so
  resuming one continues its totals instead of restarting them. It covers the
  `run` / REPL surface; the full-screen TUI status line is a separate unit.

## 6.0.0 — 2026-09-25

- Port the vendored Grok Build base from `1.0.0` to `1.0.41` — 41 upstream
  releases covering 472 changelog items, 25 sync commits and 3,587 files — and
  re-derive this product's own overlay (DeepSeek sampling mapping, the status
  line, theme skins, prompt identity, Path A helpers) on the new base with a
  three-way merge. Most of the visible change is latency and correctness work:
  session start no longer waits on remote-settings fetches or MCP connects, the
  first message in a large repository no longer waits on a full status scan,
  resuming a large session is significantly faster, git status and diff on big
  histories are bounded, subagent spawning no longer stalls or freezes the
  parent, and finished child transcripts are evicted and rebuilt on demand.
  Fixed along the way: sessions interrupted by a crash say so instead of
  dropping the turn, pasted images can no longer attach the wrong image,
  oversized images no longer brick a session, a shell command is moved to the
  background rather than cancelled when you send a message mid-run, and Esc no
  longer cancels a running turn. New: a configurable status line, a tabbed
  `/usage` `/session-info` `/context` modal, prompt-draft stashing, queued
  messages that wait for you to finish editing, workflows in the command
  palette, and hook capabilities for confirming, rewriting and adding context.
  Owner-readable summary: `docs/product/CHANGELIST_6_0_0.md`.
- A release no longer reports a failed publish when the package published
  correctly. The `v5.7.0` run read the npm registry one second after
  `npm publish` returned, got `E404` while the version was still propagating,
  and `set -euo pipefail` turned that into a failed job — the version landed
  76 s later. The post-publish read is now a bounded retry (300 s, matching the
  metadata cache lifetime) shared by `release.sh`, the emergency publish path
  and the release workflow, and the global-install smoke runs whenever a
  publish happened instead of inheriting a skip from the failed step above it.

## 5.7.0 — 2026-09-25

- The TUI fits a phone-width pane. A collapsed prompt echo now folds to one
  line plus an ellipsis at or below 60 columns (the measured iPhone pane is 55
  columns by 41 rows) instead of the fixed three rows it used to spend — about
  a third of the viewport. The bottom info line (`╰─ model · flags ─╯`) is
  drawn inside its own corners, so a long model label no longer starts on the
  divider rule and clips a character early. Desktop widths are unchanged.
- dsb reports its turn state to Orca over OSC 9999, so the sidebar dot, the
  mobile session row, and the turn-complete notification have something to
  read. Previously every dsb pane reported no agent identity and the braille
  spinner made a *working* dsb pane read as a working Claude. The frame is
  emitted only when Orca is the host, and carries `working` / `waiting` /
  `done` with control characters stripped so a crafted title cannot inject
  escapes. (Orca-side detection and launch vocabulary still need an Orca
  change; this is the status half.)
- Fix the emergency local publish path asking for a 2FA code that this account
  cannot produce. The account's second factor is a security key, so npm offers a
  browser approval instead of an emailed code; the script now runs the publish
  under a pty with `--browser=false` (both required — otherwise npm either
  refuses or blocks on "Press ENTER"), captures the approval URL npm prints, and
  approves it through the browser agent. The local publish is a fallback only:
  the default release path publishes from CI over OIDC, which needs no proof of
  presence at all.
- `npm i -g` no longer fails and deletes a working installation when the
  shell happens to carry a version stamp. `DEEPSEEK_BUILD_VERSION` (set by
  anyone who ran `scripts/build-grok-pager.sh`, and inherited by every child
  process) leaks into the agent's `--version`, so the postinstall self-check
  read a correct install as corrupt — and then deleted the binaries to "not
  leave a broken one around", taking the user's previous `deepseek-build-agent`
  with them. The check now runs without the runtime version stamps (the same
  pair the release workflows already strip), and the installer verifies every
  binary in a staging dir under the bin dir before renaming any of them into
  place, so a self-check failure leaves the previous installation untouched.
- Make syncing the vendored base a procedure with tooling rather than a
  rediscovery: a `grok-sync` skill, a runbook, a running ledger of what each
  sync decided and why, and `scripts/grok-sync-inventory.sh` — one command that
  reports the pin, the upstream commit it resolves to, upstream HEAD, the
  release/file/line gap, and a clustered inventory of every changelog bullet in
  the covered range. The old refresh procedure (`rsync --delete` plus
  re-applying `patches/grok-build/`) is documented as the small-overlay path
  only: measured against `1.0.41`, none of its 13 patches applied.
- Publish releases from CI over npm OIDC trusted publishing instead of a local
  interactive publish: the tag push now waits for the prebuilt asset, verifies
  the packaged agent reports the release version, and publishes with a
  provenance attestation. No npm token or one-time code is involved. A local
  publish remains only as an emergency fallback.
- Editing a file with CRLF line endings works, and the file keeps them. A
  multi-line `old_string` used to fail with `no_match` on any CRLF file — a
  Windows checkout, a `core.autocrlf` working copy, a `.bat` — because the
  snippet scope held `\r\n` while the model wrote `\n`. A single-line edit that
  did apply wrote LF into the file and left it with two conventions. `read` now
  returns LF and records the file's convention on the snippet, `edit` matches
  on LF and writes every line break back as the file's own, and a mixed file is
  uniformized on its next edit. Spec 45 §1.8 promised this; §1.9 now states it
  and both edit entry points — the `edit` tool and Path A `search_replace` —
  share one rule.
- Image attachments now reach the model on the official DeepSeek API again.
  DeepSeek's V4.1 Flash (`deepseek-flash`) accepts `image_url` directly, but
  the vendored sampler flattened every request bound for `api.deepseek.com`
  to text — a guard written when the flash line had no vision — so the model
  never received what the user attached. The guard is now gated on the model
  as well as the endpoint: `deepseek-v4-pro` and the V3 chat/reasoner
  families keep the text-only wire with its on-disk `<image_files>` fallback,
  while vision-capable models send the image inline.

## 5.6.0 — 2026-09-25

- First-run setup now asks for the API provider first — DeepSeek API or
  OpenRouter — then for that provider's key. The choice is saved with the key
  and the full-screen agent's model stanzas follow it (OpenRouter uses
  `deepseek/deepseek-v4-flash` / `deepseek/deepseek-v4-pro` at
  `https://openrouter.ai/api/v1`). `setup` / `auth login` accept
  `--provider deepseek|openrouter`. Session titles, web search, and image
  description are pinned to the Flash stanza instead of a vendored default
  model. A hand-written config whose default stanza reads its key from its own
  `env_key` no longer triggers the setup wizard. Re-running setup now replaces
  a stale inline key in the agent config, and `setup --help` no longer prints
  the value of `DEEPSEEK_API_KEY`. A key saved without `--provider` keeps the
  provider the home is already set up for — including after `auth logout`,
  which deletes the key but not the choice.

## 5.5.4 — 2026-08-10

- Sync the vendored Grok Build workspace through upstream commit `8a14c91`,
  including its billing nonce, skill discovery, and protobuf reflection work.
- Restore full vendor CI compatibility: repair synchronized test patterns and
  formatting, remove a dead xAI updater fallback, make `debug_redact` fixtures
  portable, and install the pinned protoc include bundle used by those tests.

## 5.5.3 — 2026-08-09

- fix compiled version injection (sccache-proof) and gate shipped tarball
- Fix release binaries shipping with the previous release's compiled version
  (5.5.2 shipped a 5.5.1-labeled agent): the product version is now baked via
  a generated file read with `include_str!`, so sccache keys on the file
  content and a version change always recompiles. The release workflow now
  also verifies the extracted tarball's agent `--version` before upload.

## 5.5.2 — 2026-08-09

- Complete the Grok Build `1.0.0` port follow-up with strict workspace
  formatting and lint compatibility, platform-safe path handling, and stable
  lifecycle, terminal-rendering, and update-check regression coverage.
- Keep pull-request Rust caches branch-scoped while allowing successful PR
  runs to save them, reducing repeated vendor validation time without sharing
  mutable cache entries across pull requests.
- Release hardening: stop the pager's version-transition cleanup from deleting
  the seeded `$GROK_HOME/CHANGELOG.md` (the welcome-screen CHANGELOG click
  silently no-oped after a version bump); make the compiled product version
  sccache-proof (a version-derived `--cfg` marker forces a cache miss, so a
  release binary can no longer be labeled with the previous version); the
  release workflow always rebuilds the agent for the release version and
  verifies `agent --version` before packaging; npm postinstall now fails the
  install (non-zero exit) instead of silently shipping a half-installed
  package when the agent self-check fails.

- Agent identity fix: sessions no longer open with "You are Grok released by
  xAI." — the vendored prompt template drops the hardcoded vendor claim, its
  encrypted copy is regenerated, and the config overlay stamps
  `system_prompt_label = "DeepSeek Build"` on every DeepSeek model stanza;
  leftover Grok fork references in existing configs are scrubbed on launch.

## 5.5.1 — 2026-08-09

- fix update banner advertising Grok Build version as available update

## 5.5.0 — 2026-08-08

- Vision-complete freeze cut merged on `main`: V1 Deep Code + V2 Reasonix + V3
  Grok throughput + V4 product finish criteria closed on public Path A evidence
- Closes V3-60-3 residual: parent `snippet_id` mint → implement-class worker
  mutates same path → parent pre-mutation edit rejected (`snippet_stale`)
- Published to npm (`5.5.0`) and GitHub Releases (`v5.5.0`) on 2026-08-08;
  dual adversarial review is external

## 5.4.0 — 2026-08-08

- L3 Path A R0A train cut (multi-tool/bg, subagent/worker-cache, worktree dogfood) + optional live L3 matrix
- **Not published:** in-repo cut merged on `main` (PR #145) only — npm and GitHub Releases skipped `5.4.0` (published `5.2.2` → `5.5.0`)

## 5.3.0 — 2026-08-08

- Spec 45 Path A Deep Code cut: public `deepseek-build`/`dsb` agent R0A multi-edit
  with session-local `snippet_id`, plus stale-id / bash invalidation fail-closed proof
  (stacked on VC003–VC005 mint/require/expire laws)
- **Not published:** in-repo cut merged on `main` (PR #138) only — npm and GitHub Releases skipped `5.3.0` (published `5.2.2` → `5.5.0`)

## 5.2.2 — 2026-08-08

- installer self-check + fresh inode (fix silent corrupt install)

## 5.2.1 — 2026-08-08

- DeepSeek Night v2 markdown hierarchy restore: h2 headings, code, and command
  lines regain hue (blue h2/code, v1-yellow commands) on top of the unchanged
  C-balanced surfaces

## 5.2.0 — 2026-08-08

- theme classic default + vision complete + theme picker restore

## 5.1.0 — 2026-08-07

### Changed
- Default theme is now **DeepSeek Night v2**, a measured C-balanced palette:
  six semantic hue families, zero hue collisions, and every text role at WCAG AA
  or better on both the base and raised surfaces.
- Hover and selection separate on different axes (lightness vs chroma).
- Grok Night, classic DeepSeek Night, and DeepSeek Night Neutral are no longer
  listed in theme pickers. Existing configs naming them keep working.

### Fixed
- Settings theme sheet now lists the shipped product theme, so users can switch
  back without using `/theme`.
- `oscura-midnight` renders as "Oscura Midnight" instead of a raw identifier.

## 5.0.1 — 2026-08-07

- widen the DeepSeek whale logo to official terminal proportions

## 5.0.0 — 2026-08-07

- Owner-bar complete product cut (`owner-bar-5x`): Path A P0 ledger green, dual adversarial reviews, tag `v5.0.0`.

## 4.0.4 — 2026-08-07

- **Image attachments** on text-only DeepSeek endpoints no longer 400: images persist to session assets with an agent-driven OCR hint (matches Reasonix/DeepCode preprocessing instead of silent drop)
- **DeepSeek status line** with account balance & cache hit rate
- **G003:** `mint file_version` on Path A `read_file` (snippet contract)
- **G004:** Standard `snippet_safe` tool_configs now always applied (dead wiring fix) + `liveness-3edits` harness scenario
- CI: dotslash install + full-build fallback fix for prebuilt tag runs

## 4.0.3 — 2026-08-07

### CLI

- **`dsb --resume [<id>]` / `-r`** resumes a full-screen TUI session (bare flag = most-recent session); `--minimal` / `--fullscreen` forwarded to the TUI
- Quit and screen-mode relaunch hints branded as **`dsb`** / **`deepseek-build`** (via `GROK_INVOCATION_NAME`) instead of upstream `grok`
- `--resume` conflicts with `--session`; TUI-only flags rejected on line-mode subcommands (`run`, `chat`, …)

## 4.0.2 — 2026-08-07

### UX

- `dsb setup` next steps: bare **`dsb`** (full-screen agent TUI), not legacy `dsb chat`
- `chat` documented as line-mode only

## 4.0.1 — 2026-08-07

### Install DX (critical)

- **`npm i -g` no longer compiles Grok from source** (ADR 0009).
- `postinstall` downloads platform prebuilts from GitHub Releases into `~/.deepseek-build/bin/`.
- npm package is thin (wrappers only) — no `third_party/grok-build` in the tarball.
- Optional source fallback: `DEEPSEEK_BUILD_ALLOW_SOURCE_BUILD=1` only.

## 4.0.0 — 2026-08-07

### L3 productization (PRD-v4 / fleet-4x)

- **Product defaults:** `[subagents] enabled = true` in auto-created product config; keep **`yolo = false`** (hearts)
- **Capability matrix** + user guides 11–14 (subagent / bg / worktree / throughput)
- **Smoke:** `./scripts/test-l3-smoke.sh`
- Tag **`v4.0.0`** (full SemVer only)

## 3.0.0 — 2026-08-07

### Heart fusion (product major)

- **2.x was shell cut; 3.0.0 is heart fusion** (PRD-v3 P0)
- L1: Path A snippet_safe edit + Spec 90 permissions matrix (not YOLO default)
- L2: Path A prefix assembly + tool-call repair + Flash-first / Pro escalate
- Honesty docs: README, KNOWN_LIMITS, cut evidence
- Tag **`v3.0.0`** (full SemVer only)

### Residual (honest)

- Spec 45 **file_version** equivalent on Grok path (full snippet_id mint polish → 3.x minor)
- Live dogfood env-gated; L3 product identity → 4.x

## 3.0.0-beta.2 — 2026-08-07

### L2 repair + Flash/Pro (Path A)

- `dsb-agent` `path_a_turn`: Spec 15 prep-before-execute + Spec 20 Flash default / Pro once under agent defaults
- H15.* / H20.* contract tests; H2 exit band

## 3.0.0-beta.1 — 2026-08-07

### L2 prefix (Path A agent context)

- `dsb-context` `assemble_path_a_context`: stable prefix + volatile tail under Spec 10 for default agent path
- H10.* epoch stability tests (identical inputs, tool/skills thrash, volatile isolation)

## 3.0.0-alpha.2 — 2026-08-07

### L1 permissions (Path A)

- Spec 90 spirit matrix for default agent: headless Ask→Deny, TTY Ask for writes, deny out-of-cwd
- Product seed/repair: explicit `yolo = false` when missing (does not clobber user `yolo = true`)
- `dsb-tools` `path_a_permissions` contract tests (H90.*)

## 3.0.0-alpha.1 — 2026-08-07

### L1 heart (Path A snippet-safe)

- Spec 45 spirit on **default Grok** `search_replace` path: `snippet_safe` + `file_version` gate; empty-old whole-file overwrite fail-closed
- Product adapter: `dsb-tools` `path_a_edit` contract tests (H45.*)
- Standard file toolset injects `snippet_safe=true` for DeepSeek agent

## 2.0.3 — 2026-08-07

### Install (product contract)

- **`npm install -g @innocarpe/deepseek-build` postinstall** now builds and installs:
  1. wrapper `dsb` / `deepseek-build`
  2. full-screen agent `deepseek-build-agent` (DeepSeek TUI)
- After install + PATH: **`dsb`** alone opens DeepSeek full-screen TUI
- Requires Rust + protoc (or dotslash). First agent build may take several minutes.
- Skip agent only: `DEEPSEEK_BUILD_SKIP_AGENT_BUILD=1` (not recommended)

## 2.0.2 — 2026-08-07

### Product entry (DeepSeek TUI only)

- Bare `dsb` / `deepseek-build` = **DeepSeek Build full-screen TUI** (product)
- CLI binary name / help Usage: **dsb** (not `grok`)
- User-facing help no longer describes the product as "Grok-class" / Grok Build UI
- `repl-legacy` hidden; line-mode remains as `chat` only for legacy/script use

## 2.0.1 — 2026-08-07

### UI / UX (DeepSeek product chrome)

- **DeepSeekNight** default TUI theme (`#4D6BFE` accents) in vendored Grok pager
- Welcome hero: DeepSeek whale braille logo + **DeepSeek Build** product strings
- Launcher splash: whale + DeepSeek Build before full-screen agent
- Force `GROK_THEME=deepseeknight` from `dsb` entry (override with `DEEPSEEK_BUILD_THEME`)
- Product config seed includes `theme = "deepseeknight"`

## 2.0.0 — 2026-08-06

### Product

- **First product release** matching REPLAN_2.0 P0: Grok Build–class agent entry with DeepSeek default
- Vendored Grok Build under `third_party/grok-build/` (ADR-0008)
- No-args TTY `dsb` / `deepseek-build` launches `deepseek-build-agent` (Grok pager)
- DeepSeek models + `api.deepseek.com` + chat_completions config seed
- Setup/auth under `~/.deepseek-build/` (credentials 0600)
- L1/L2 evidence: `docs/product/evidence/W3_L1_L2_MATRIX.md`
- W2 chat/edit dogfood evidence under `docs/product/evidence/`
- `1.x` remains legacy scaffold on npm history

### Notes

- npm publish may still require human OTP (ADR 0007 residual)
- Upstream pager chrome may still say “Grok” in places; product CLI/docs use DeepSeek Build

## 2.0.0-alpha.2 — 2026-08-06

### Added

- **No-args TTY** `dsb` / `deepseek-build` launches Grok-class full-screen agent (`deepseek-build-agent`)
- `agent` subcommand + `repl-legacy` thin REPL path
- Install builds/installs vendored `xai-grok-pager` as `deepseek-build-agent`
- Product `config.toml` seed: DeepSeek models, `api.deepseek.com`, chat_completions
- First-run setup before agent launch; credentials 0600 under `~/.deepseek-build/`
- Smoke note: `docs/product/evidence/W1_ENTRY_SMOKE.md`

## 2.0.0-alpha.1 — 2026-08-06

### Added

- **Grok Build vendor pin** under `third_party/grok-build/` (ADR-0008 strategy B)
- `SOURCE_REV` pin + Apache-2.0 / `THIRD-PARTY-NOTICES` retained in vendor tree
- Root `NOTICE` attribution for SpaceXAI Grok Build
- `scripts/build-grok-pager.sh` + [GROK_VENDOR.md](docs/architecture/GROK_VENDOR.md) (dual workspace + CI plan)
- Product SemVer band opens at **`2.0.0-alpha.1`** (not 2.0.0 cut)

### Notes

- Default `dsb` entry still product overlay until W1 entry/TUI stories land
- Full Grok `cargo check -p xai-grok-pager-bin` verified on vendor tree (local evidence)

## Prior unreleased notes (folded)

### Added

- **Welcome banner v2** — DeepSeek braille whale mark (official logo silhouette raster) + boxed product card (`banner.rs`)
- REPL prompt `❯` uses DeepSeek blue accent when color is enabled
- Theme/docs: official `#4D6BFE` tokens documented for mark, box chrome, and prompt

### Documentation

- **Product replan for 2.0.0** — [REPLAN_2.0.md](docs/product/REPLAN_2.0.md): 1.x repositioned as **scaffold**; real product DoD is **`dsb` opens a Grok Build–class coding agent** on open-source Grok Build + DeepSeek/Deep Code/Reasonix overlays
- **One-plate ultragoal** `grokbase-2x` G001–G012: [GROKBASE_2X_GOALS.md](docs/product/GROKBASE_2X_GOALS.md), [ULTRAGOAL_BRIEF_2.0.md](docs/product/ULTRAGOAL_BRIEF_2.0.md), cold-start through cut
- Fixed PR units: [WAVE_2x_PR_DAG.md](docs/product/WAVE_2x_PR_DAG.md)
- README / SSOT / versioning / MASTER_PLAN / KNOWN_LIMITS honesty banners

## 1.1.0 — 2026-08-06

### Added

- **First-run setup onboarding** — `setup` / `auth login|status|logout`
- TTY `chat`/`run` auto-prompt for API key when missing; saves `credentials.json` (0600)
- Bare `deepseek-build` with no key starts setup on TTY
- User guide `00-setup.md`

### Notes

- Still the **1.x scaffold line** (thin agent UX). Product target is **2.0.0** — see REPLAN_2.0.

## 1.0.0 — 2026-08-06

### Release

- First **1.0.0** after Waves A–D **scaffold train**: dogfood core, DeepSeek-native surface, throughput **MVP**, RC harden/docs
- Product CI (later refined into split path-gated workflows)
- Full user-guide + known limits
- Dual CLI `deepseek-build` / `dsb`; npm package `@innocarpe/deepseek-build` (registry publish remains owner-gated)

### Notes

- **Repositioned (2026-08-06):** this release is a **contract/scaffold line**, not a Grok Build–class full agent product. See [REPLAN_2.0.md](docs/product/REPLAN_2.0.md) and [KNOWN_LIMITS.md](docs/product/KNOWN_LIMITS.md).

All notable product versions use full SemVer `MAJOR.MINOR.PATCH`.

## 0.16.0 — 2026-08-06

### Documentation

- Expanded user-guide: auth, chat/run, permissions, theme, tools
- Added `docs/product/KNOWN_LIMITS.md`
- This CHANGELOG

## 0.15.0 — 2026-08-06

### Added

- Product CI: `.github/workflows/ci.yml` (fmt, clippy, test, offline smoke)
- Harden path for Wave D RC

## 0.14.0 — 2026-08-06

### Added

- Spec 60 + G5: in-process subagents, worker cache law
- Tool `subagent` (explore | implement)

## 0.13.0 — 2026-08-06

### Added

- Background bash (`background: true` → `job_id`)
- Tool `bash_collect`

## 0.12.0 — 2026-08-06

### Added

- Spec 50 + G4: parallel read-only tools (serial mutating)

## 0.11.0 — 2026-08-06

### Added

- Specs 80/110; G6c/G6d green
- MCP catalog + schema fingerprint; tool `plan`

## 0.10.0 — 2026-08-06

### Added

- Skills product expand: opt-out frontmatter, `skills list`

## 0.9.0 — 2026-08-06

### Added

- TTY permission ask once/always grants
- DeepSeek blue theme v1 + DESIGN.md

## 0.8.0 — 2026-08-06

### Added

- Spec 40 core tools surface ready-for-impl + registry pins

## 0.7.0 – 0.7.1 — 2026-08-06

### Added

- npm package `@innocarpe/deepseek-build` dual bins
- Help SemVer example tracks package version

## 0.2.0 – 0.6.0 — 2026-08-06

Wave A dogfood core: install, tools daily, dogfood proof, sessions, skills index min + effort UX.

## 0.1.0 — 2026-08-06

Initial engine + tools core preview.
