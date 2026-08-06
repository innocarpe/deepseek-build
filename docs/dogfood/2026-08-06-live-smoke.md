# Dogfood live smoke — 2026-08-06

## Commands used
- deepseek-build --version
- deepseek-build run "Reply with exactly: pong"
- deepseek-build --dogfood run (this turn) to write this note

## Auth
- Source: ~/.deepseek-build/credentials.json (mode 0600)
- Env DEEPSEEK_API_KEY not required when file is set

## Result
- Live Flash chat returned pong successfully
- Agent can write under --dogfood (workspace only)

## Limits observed
- Without --dogfood, writes need --allow-workspace-write; bash is dry-run without --bash-execute/--dogfood
- No interactive ask UX yet (headless ask→deny)
- grep is literal substring, not full regex

## Next
- Sessions (0.5.0), surface (0.6.0), npm (0.7.0)
