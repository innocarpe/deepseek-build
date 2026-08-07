# WAVE — vision-complete `5.x` PR DAG

**Plan:** [VISION_COMPLETE_5X_GOALS.md](./VISION_COMPLETE_5X_GOALS.md)  
**Rules:** [ULTRAGOAL_PR_PLANNING.md](./ULTRAGOAL_PR_PLANNING.md)

## Graph (high level)

```text
VC001 5.0.1 version fix ship
   │
   ├──────────────┬──────────────────┐
   ▼              ▼                  ▼
Track A        Track B            Track C
Deep Code      Reasonix           Grok L3
VC002–006      VC007–009          VC010–013
   │              │                  │
   └──────────────┴──────────────────┘
                    ▼
              VC014 docs
                    ▼
              VC015 freeze v5.Y.0
```

## PR units (template filled)

### PR unit VC001 — `chore(release): 5.0.1 product version + agent prebuilt`
- **Intent:** npm users get SemVer-aligned agent; no false v1.0.0 banner
- **Touches:** package versions, release scripts, agent build, evidence
- **Depends on:** none
- **SemVer:** **5.0.1**
- **Tests:** install dual CLI; agent `--version` shows 5.0.1; no update banner vs npm latest

### PR unit VC002 — `spec(45): Path A snippet_id ADR + store design`
- **Intent:** lock snippet_id vs file_version migration
- **Touches:** `docs/adr/`, `docs/specs/45-snippet-edit.md` notes
- **Depends on:** none (can parallel VC001)
- **SemVer:** none
- **Tests:** ADR accepted in review checklist

### PR unit VC003 — `feat(tools): mint snippet_id on Path A read_file`
- **Intent:** session table + wire/return shape
- **Touches:** dsb-tools / grok tools / agent session
- **Depends on:** VC002
- **SemVer:** none (or 5.1.0-pre)
- **Tests:** unit + hermetic Path A read shows snippet_id

### PR unit VC004 — `feat(tools): require snippet_id on Path A search_replace`
- **Intent:** Spec 45 primary path
- **Depends on:** VC003
- **SemVer:** **5.1.0** with VC005/6
- **Tests:** negative missing/stale; multi-edit liveness

### PR unit VC005 — `feat(tools): write/bash snippet invalidation laws`
- **Depends on:** VC004
- **Tests:** G005-style + write overwrite

### PR unit VC006 — `test(hearts): regression under snippet_id`
- **Depends on:** VC005
- **Tests:** `test-heart-regression.sh --with-e2e`

### PR unit VC007 — `feat(context): Spec 10 assembly on Grok Path A turns`
- **Intent:** real message builder uses product prefix contract
- **Depends on:** VC001; prefer after VC006
- **Tests:** multi-turn wire golden system/tools/skills

### PR unit VC008 — `feat(provider): reasoning_effort on DeepSeek wire`
- **Depends on:** VC007 prefer
- **SemVer:** **5.2.0** with VC009
- **Tests:** wire field present for high effort

### PR unit VC009 — `feat(ui): cache-hit visibility`
- **Depends on:** VC008 soft
- **Tests:** status row or log golden

### PR unit VC010 — `test(l3): multi-tool parallel + bg Path A R0A`
- **Depends on:** VC006
- **Tests:** hermetic multi-tool scenario

### PR unit VC011 — `test(l3): subagent + worker cache Path A R0A`
- **Depends on:** VC010
- **Tests:** spawn + prefix hash parent/worker

### PR unit VC012 — `docs+test(worktree): dogfood flow`
- **Depends on:** VC011
- **Tests:** worktree create/use/cleanup scripted

### PR unit VC013 — `test(live): extended matrix when key present`
- **Depends on:** VC012
- **Tests:** `test-l3-smoke --extended` honesty

### PR unit VC014 — `docs(product): user-guide + KNOWN_LIMITS vision pass`
- **Depends on:** VC013
- **Tests:** doc review checklist

### PR unit VC015 — `chore(release): vision-complete freeze v5.Y.0`
- **Depends on:** VC014 + dual review
- **SemVer:** **5.Y.0**
- **Tests:** vision ledger all PASS; heart regression; install

## Merge policy

- GitHub **merge commit** only  
- Labels mandatory  
- English public text  
- Stack: `--base` previous open PR when sequential  
