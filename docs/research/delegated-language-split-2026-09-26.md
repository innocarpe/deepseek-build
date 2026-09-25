# Delegated-run language split — 2026-09-26

**Nature:** Non-binding evidence ([SOURCES.md](../product/SOURCES.md) precedence).
Not a product commitment. This note does not change prompts, tools, or runtime code.

**Question.** On 2026-09-25, `session_kind=subagent` runs in this repo's session
logs answered in Korean at anywhere from 0% to 100%, under English task briefs,
with a Korean output rule loaded. What makes that split, or which candidates
do the logs rule out?

**Answer, in one sentence.** The logs do not identify the splitter. They do
rule out rule absence, model id, effort, parent language, and "the run switched
because it had just read Korean." What remains is a generation-time choice
under task texts that share an opener and the same rule bytes but are not
token-identical, and that choice was not replayed on one frozen prompt.

Tree used for the code citations below: `13affaa` (merge of PR #223). The
sessions themselves were recorded on 2026-09-25 and were not re-run.

---

## 1. Observation

A prior census of these logs, using the criterion in §2, reported the rates in
the **Prior** column. Re-running that criterion on the files on disk produced
**This read**. Rates are Korean prose rows / non-empty assistant prose rows.

| Session prefix | n | Korean | Rate | Run length | Prior |
|---|---:|---:|---:|---|---|
| `01a0d813-fac2` | 160 | 0 | 0.000 | `E160` | 0% |
| `01a0d813-e7d4` | 43 | 42 | 0.977 | `E1K42` | 98% |
| `01a0d814-1c62` | 135 | 1 | 0.007 | `E68K1E66` | 1% |
| `01a0d814-30cf` | 76 | 0 | 0.000 | `E76` | 0% |
| `01a0d7cd-60ad` | 25 | 0 | 0.000 | `E25` | 0% |
| `01a0d7cd-7c24` | 28 | 28 | 1.000 | `K28` | 100% |
| `01a0d7ff-c62e` | 7 | 5 | 0.714 | `E1K5E1` | 71% |
| `01a0d895-ffea` | 120 | 14 | 0.117 | `E1K2E105K12` | ~2% |
| `01a0d795-5f71` | 40 | 0 | 0.000 | `E40` | 0% |
| `01a0d8b0-66e3` | 5 | 4 | 0.800 | `K4E1` | 80% |
| `01a0d8b0-66e4` …`26dec9` | 19 | 18 | 0.947 | `E1K18` | 95% |
| `01a0d8b0-66e4` …`179bbb` | 13 | 13 | 1.000 | `K13` | 100% |
| `01a0d8b0-66e5` …`241d5f` | 5 | 1 | 0.200 | `E4K1` | 20% |
| `01a0d8b0-66e5` …`b7e8c6` | 9 | 9 | 1.000 | `K9` | 100% |
| `01a0d8b0-66ee` | 18 | 17 | 0.944 | `E1K17` | 94% |

Run length is the prose rows in order: `E` = no Hangul syllable, `K` = at least
one, with the count of that stretch. `E1K42` means the first prose row has no
Hangul and the next 42 do.

The prior percentages match this read, with three readings that need the run
length and not just the rate:

- `01a0d814-1c62` is 1/135. That single `K` is one row between two English
  stretches, not a switch that held.
- `01a0d8b0-66e5` …`241d5f` is `E4K1`. The one Korean row is the last one, the
  final report. Four progress rows are English. 20% here is not "a fifth of the
  sentences."
- `01a0d895-ffea` was still being appended while this note was written. An
  earlier pass in the same sitting saw 109 non-empty prose rows and 3 Korean
  (`E1K2E105K1`, rate 0.028), which is the prior "~2%" (that census was unsure
  between 42 and 88 rows). The table above is the later snapshot, 120 rows.
  Of those, 109 were `deepseek/deepseek-v4.1-flash` (106 `E`, 3 `K`, rate
  0.028) and 11 were `deepseek-flash` (11 `K`). The blended 0.117 is the flash
  tail added onto an English v4 stretch. Every other session in the table
  stayed on `deepseek/deepseek-v4.1-flash` for every prose row, and their files
  did not change between the two passes.

All fifteen are `session_kind=subagent`. The two sharp pairs from the prior
census are real and survive the re-count:

- `01a0d813-fac2` (`E160`) and `01a0d813-e7d4` (`E1K42`), spawned in the same
  minute by the same parent.
- `01a0d7cd-60ad` (`E25`) and `01a0d7cd-7c24` (`K28`), same parent, seven
  seconds apart.

A third batch, six `explore` runs spawned together (`01a0d8b0-66e3` through
`01a0d8b0-66ee`), ranges from `E4K1` to `K13` under one parent.

The prior note that both the 0% runs and the 98% runs open in English is true
of the conflict quartet (`fac2`, `e7d4`, `1c62`, `30cf`): each first prose row
is an ASCII sentence beginning "I'll start by…". It is false of the set.
`7c24`, `66e3`, `66e4` …`179bbb`, and `66e5` …`b7e8c6` are Korean on row 1,
before any tool result.

---

## 2. Measurement

Logs live at `~/.deepseek-build/sessions/<percent-encoded cwd>/<session id>/`.
This read used `summary.json`, `chat_history.jsonl`, `prompt_context.json`,
`system_prompt.txt`, `events.jsonl`, and the parent's `updates.jsonl`.

**Prose criterion**, same as the prior census:

- Count rows with `type == "assistant"` whose body, after `strip()`, is
  non-empty. A row that only carries tool calls is not counted.
- A counted row is Korean if the body matches `[\uac00-\ud7a3]` at least once.
- Rate = Korean rows / counted rows.

One Hangul syllable marks the row, including an English sentence that quotes a
Korean word. That is why §1 also gives the run length. A stricter "more Hangul
letters than Latin letters" cut was computed and then not used as the headline:
Korean sentences that cite English identifiers fail it (the final report of
`7c24` has 3817 Hangul syllables and 12456 Latin letters, and it is a Korean
report). The any-syllable rule is the one that matches the prior table.

**Tool-description criterion**, not part of the prior census. For each
assistant tool call, if `arguments` is a JSON object with a string
`description`, that string is Korean when it matches the same syllable class.
`explore` runs recorded no `description` argument at all (`read_file`, `grep`,
`list_dir` keys only), so this channel does not exist there. Absence is not an
English description.

**Parent linkage.** `events.jsonl` `turn_started.session_relationship` is
`primary` on every one of these subagent sessions, and each session has exactly
one `turn_started`. On this tree the field is hardcoded at
`third_party/grok-build/crates/codegen/xai-grok-shell/src/session/acp_session_impl/turn.rs:854`:

```rust
session_relationship: crate::session::events::SessionRelationship::Primary,
```

Parents were taken from `subagent_spawned` records in the parent's
`updates.jsonl` (`parent_session_id`, `child_session_id`, `subagent_type`,
`model`, `description`). The child task text does not contain that description:
every child task in §1 has zero Hangul syllables, including the children whose
spawn description is Korean.

The script that produces the prose table, the run length, the description
counts, and the prompt hashes is below. It prints no message bodies. It was
run from `/tmp` against the local session store; it is not part of the product.

```python
#!/usr/bin/env python3
"""Recount delegated-run Korean rates. Stdlib only. Prints no message bodies.

Prose: type==assistant, body strip non-empty. Korean: [\\uac00-\\ud7a3] once.
Description: tool-call arguments["description"], same syllable test.
"""

from __future__ import annotations

import hashlib
import json
import os
import re

HANGUL = re.compile(r"[\uac00-\ud7a3]")
PREFIXES = (
    "01a0d813-fac2", "01a0d813-e7d4", "01a0d814-1c62", "01a0d814-30cf",
    "01a0d7cd-60ad", "01a0d7cd-7c24", "01a0d7ff-c62e", "01a0d895-ffea",
    "01a0d795-5f71", "01a0d8b0-66e3", "01a0d8b0-66e4", "01a0d8b0-66e5",
    "01a0d8b0-66ee",
)

def sha(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8", errors="replace")).hexdigest()[:12]

def text_of(content) -> str:
    if isinstance(content, str):
        return content
    if isinstance(content, list):
        parts = []
        for block in content:
            if isinstance(block, str):
                parts.append(block)
            elif isinstance(block, dict) and isinstance(block.get("text"), str):
                parts.append(block["text"])
        return "\n".join(parts)
    return ""

def rle(bits: list[str]) -> str:
    if not bits:
        return ""
    out, cur, n = [], bits[0], 1
    for bit in bits[1:]:
        if bit == cur:
            n += 1
        else:
            out.append(f"{cur}{n}")
            cur, n = bit, 1
    out.append(f"{cur}{n}")
    return "".join(out)

def load_jsonl(path: str) -> list:
    rows = []
    with open(path, encoding="utf-8") as handle:
        for line in handle:
            line = line.strip()
            if line:
                rows.append(json.loads(line))
    return rows

def find_sessions(root: str) -> list[str]:
    found = []
    for dirpath, dirnames, filenames in os.walk(root):
        base = os.path.basename(dirpath)
        if base.startswith(PREFIXES) and "chat_history.jsonl" in filenames:
            found.append(dirpath)
            dirnames.clear()
    return sorted(found)

def main() -> None:
    root = os.path.expanduser("~/.deepseek-build/sessions")
    for path in find_sessions(root):
        rows = load_jsonl(os.path.join(path, "chat_history.jsonl"))
        prompt = json.load(open(os.path.join(path, "prompt_context.json"), encoding="utf-8"))
        system = open(os.path.join(path, "system_prompt.txt"), encoding="utf-8").read()
        bits, models = [], set()
        desc_n = desc_k = 0
        instr = task = ""
        for row in rows:
            kind = row.get("type")
            if kind == "assistant":
                body = text_of(row.get("content")).strip()
                if row.get("model_id"):
                    models.add(row["model_id"])
                if body:
                    bits.append("K" if HANGUL.search(body) else "E")
                for call in row.get("tool_calls") or []:
                    raw = call.get("arguments")
                    try:
                        args = json.loads(raw) if isinstance(raw, str) else raw
                    except json.JSONDecodeError:
                        args = None
                    if isinstance(args, dict) and isinstance(args.get("description"), str):
                        desc_n += 1
                        if HANGUL.search(args["description"]):
                            desc_k += 1
            elif kind == "user" and row.get("synthetic_reason") == "project_instructions":
                instr = text_of(row.get("content"))
            elif kind == "user" and row.get("prompt_index") == 0 and not row.get("synthetic_reason"):
                task = text_of(row.get("content"))
        rule = next(
            (item for item in prompt.get("agents_md_files") or []
             if "99-hq-language" in (item.get("file_name") or "")),
            None,
        )
        n, k = len(bits), bits.count("K")
        sid = os.path.basename(path)
        print(
            sid[:13], sid[-6:],
            f"n={n}", f"ko={k}", f"rate={0 if n == 0 else round(k / n, 3)}",
            rle(bits),
            f"desc={desc_k}/{desc_n}",
            f"sys_h={len(HANGUL.findall(system))}",
            f"sys={sha(system)}",
            f"instr={sha(instr)}", f"instr_h={len(HANGUL.findall(instr))}",
            f"task={sha(task)}", f"task_h={len(HANGUL.findall(task))}", f"task_len={len(task)}",
            f"rule={sha(rule.get('content') or '') if rule else 'NONE'}",
            f"models={','.join(sorted(models))}",
        )

if __name__ == "__main__":
    main()
```

---

## 3. What is the same on both sides of a split

### Conflict quartet — parent `01a0d7f7-acf7`

Spawned 2026-09-25 10:19 UTC, `general-purpose`, `reasoning_effort=max`,
wire model `deepseek/deepseek-v4.1-flash` on every prose row.
`01a0d813-e7d4` at 10:19:38, `fac2` at 10:19:43, `1c62` at 10:19:51,
`30cf` at 10:19:57.

| Input | Shared? |
|---|---|
| `system_prompt.txt` sha `cad868b50f91`, Hangul syllables | yes, 0 |
| project-instruction user message sha `0370f1354134`, 16455 bytes | yes |
| `99-hq-language.md` sha `50cd7ba7ad61`, 630 Hangul syllables | yes, loaded |
| repo `Agents.md` sha `d359a30cfe72`, Hangul syllables | yes, 0 |
| `turn_started`: model, `yolo_mode=true`, `conversation_message_count=2`, `session_relationship=primary` | yes |
| task Hangul syllables, and the words "English" / "Korean" / "language" | 0 / absent |
| task bytes | no. Line-similarity 0.264–0.462. Shared opener is "You are resolving git merge conflicts in a scratch merge repository." The differing lines are file lists and reworded merge rules. None of those lines is a reply-language instruction. |
| first tool-result Hangul | 0 in all four. Lengths differ (`e7d4` 413 bytes; the English-prose siblings 800–2400). |

Parent prose, same criterion: 722/723 Korean (`K722E1`). Spawn descriptions,
which are parent-side labels and are not in the child prompt, are Korean for
all four ("충돌 해소 그룹 A" through "그룹 D"). The parent was Korean. The
children still split `E160` / `E1K42` / `E68K1E66` / `E76`.

`e7d4` switches on prose row 2, after a tool result that contains no Hangul.
The other three never switch, except `1c62`'s single mixed row at position 69.

### Gap pair — parent `01a0d7cc-d826`

`explore`, same effort, same wire model, system-prompt sha `1ea3c01c3110`
(the hash differs from the quartet because the template includes the workspace
path, and because the explore tail differs from `general-purpose`).
Project-instruction message is 16460 bytes on both. `99-hq-language.md` is the
same sha `50cd7ba7ad61`. Created 09:02:36 and 09:02:43 UTC.

Task opener, shared for the first 80 characters: "You are doing a READ-ONLY gap
analysis…". Full-text line similarity 0.286. The only lines matching
"language" are "Report the language(s) used and rough LOC per area" and
"Language/stack." Those name the programming language of the tree, not the
reply language. Task Hangul count is 0 on both. Spawn descriptions are English
("Explore Reasonix upstream", "Explore Deep Code upstream").

`60ad` is `E25`. `7c24` is `K28`. `7c24`'s first prose row is Korean and has
no tool result before it. `60ad`'s first prose row is English and has no tool
result before it. The split is already there on the first generated sentence.

Parent prose: 46/69 Korean. One parent, both child outcomes.

### dsh batch — parent `01a0d8ac-50cf`

Six `explore` runs, one parent, created in the same second
(13:10:34 UTC, spanning 35 ms). Same wire model, same effort, same system-prompt sha
`52c0cbd19bb1`, same `99-hq-language.md` sha, project-instruction sha
`0370f1354134` (the primary-checkout blob). Task opener shared: "You are
surveying the DeepSeek Harness (`dsh`) source tree…". Pairwise line similarity
0.158–0.353. No reply-language word. Task Hangul count 0. Spawn descriptions
are Korean subsystem labels. Outcomes: `K4E1`, `E1K18`, `K13`, `E4K1`, `K9`,
`E1K17`.

Parent prose was still being appended during this reading. One pass counted
320/320 Korean; a later pass counted 332/332 Korean. Stored reasoning
`summary` text on the later pass is not Korean prose: 426 Hangul syllables
against 70875 Latin letters across 50 reasoning rows, and none of those rows
has more Hangul than Latin. A later re-count of this parent can move; the
subagent files in the table did not, except `01a0d895-ffea`. The subagent sessions
in the quartet and the gap pair have zero `reasoning` rows, so their reasoning
language is not in the log. `01a0d895-ffea` has 14 reasoning rows, also
Latin-dominated (469 Hangul syllables, 17631 Latin letters).

---

## 4. Candidate verdicts

| Candidate | What was checked | Verdict | Evidence |
|---|---|---|---|
| Rule not loaded | `prompt_context.json` `agents_md_files[]` and the `## From:` list in the project-instruction user message | Excluded as the splitter | 14 of 15 sessions load `~/.claude/rules/99-hq-language.md`, sha `50cd7ba7ad61`, including both sides of every pair in §3. The title line "세션이 내는 모든 글은 한국어다" sits at about 20% of that user message, and the message then continues with an 11k-byte English `Agents.md`. |
| Rule only on one side of a pair | sha of that file, and of the whole project-instruction message | Excluded | Identical within each pair in §3. |
| Rule in the system prompt | `system_prompt.txt` and the `system` chat row, syllable count | Excluded as a position that varies | 0 Hangul syllables in every session's system prompt. The chat `system` row matches `system_prompt.txt`. The rule arrives only as the user message with `synthetic_reason=project_instructions`. That position is the same on both sides. |
| Model id | `summary.json` `current_model_id`, every assistant `model_id`, spawn `model` | Excluded for the pairs in §3 | All of those prose rows are `deepseek/deepseek-v4.1-flash`. Spawn `model` is `deepseek-v4-flash`. `01a0d895-ffea` is the exception, and it is not one of those pairs: see §1. |
| Reasoning effort | `summary.json` `reasoning_effort` | Excluded | `max` on every subagent in the table. |
| Starting context size | `turn_started.conversation_message_count`, instruction length, task length | Excluded as the splitter | `conversation_message_count` is 2 on every subagent's single turn event. Instruction length is 16455 or 16460, the same within each pair. Gap-pair tasks are 2813 vs 2780 bytes. |
| Brief shape ("Do NOT", "Report back", "Rules:") | marker counts on the task message | Excluded as sufficient | The conflict tasks each contain one "Do NOT" and one "Report back". The dsh tasks each contain "Rules:". The markers occur on both outcomes. |
| Reply-language line in the task | case-insensitive scan for English / Korean / language / 한국어 | Explains one session, not the split | The only task that tells the model the reply language is `01a0d795-5f71`: a heading "Report back (concise, English)". That run is `E40`, and it is also the only run that did not load `99-hq-language.md` (created 08:01 UTC; every later session in the table loaded it). `01a0d895-ffea`'s task says the work-order file "is in Korean"; the v4 prose still stayed English for 106 of 109 rows. The gap pair's "language" lines are about the upstream tree's programming language. |
| Switched after reading Korean | Hangul count of tool results preceding the first `K` prose row | Excluded | `7c24` and three of the dsh runs (`66e3`, `66e4` …`179bbb`, `66e5` …`b7e8c6`) are `K` on row 1, with no prior tool result. `e7d4` switches on row 2 after a tool result with 0 Hangul. |
| Parent language decides the child | parent prose rate, spawn description, whether that description is in the child task | Excluded as the splitter | The conflict parent is 722/723 Korean and spawned both `E160` and `E1K42`. The gap parent spawned both `E25` and `K28`. Spawn descriptions are not in the child task (child task Hangul count is 0). |
| `session_relationship` | every `turn_started` event | Excluded | The value is `primary` on all fifteen, so it does not vary and it does not point at the parent. See the hardcoded write cited in §2. |
| Tool-result language after the first row | Hangul count and byte length of the first tool results | Not the first-row split. Not shown to cause the later switch. | First-row language is chosen before any tool result. `e7d4` still switches after a 0-Hangul result. The English-staying siblings also got 0-Hangul results, of different lengths. Length was not isolated from the task text. |
| Temperature or seed | `events.jsonl` keys, assistant-row keys | Inconclusive | Neither file records temperature or a seed. `SamplerConfig.temperature` defaults to `None` (`xai-grok-sampler/src/config.rs`) and the client copies that field onto the request. A `Some(0.7)` in `client.rs` is a unit-test fixture, not this path. What the API did with an omitted temperature is not in these logs. |
| Same prompt, resampled | a frozen brief sent more than once | Not run | No deepseek-build subagent was launched for this note. The spawn tool available in the writing session does not record `session_kind=subagent` under `~/.deepseek-build`. The briefs in §3 are not token-identical, so the natural pairs are not that experiment. |

Two measurements are real and are not the sibling splitter:

**The description field can obey the rule while the prose does not.** On the
conflict quartet, which is where `description` exists:

| Session | Prose | Descriptions with Hangul |
|---|---|---|
| `fac2` | `E160` | 251/251 |
| `e7d4` | `E1K42` | 86/86 |
| `1c62` | `E68K1E66` | 4/204 |
| `30cf` | `E76` | 0/218 |

`fac2`'s first prose row is an English "I'll start by…" sentence, and that
same turn's tool descriptions are Korean (the first description is 18 Hangul
syllables and 0 Latin letters). A prose-only census records this run as 0%.
The description field does not explain the prose split: `fac2` and `e7d4` both
filled it with Hangul, and their prose still diverged. `30cf` filled it in
English. Among siblings, the description channel splits too.

**The system prompt tells the model that a direct user instruction outranks a
project file.** The stored `system_prompt.txt` of these sessions contains the
sentence, and the current template has it at
`third_party/grok-build/crates/codegen/xai-grok-agent/templates/subagent_prompt.md:64`:

> Direct user instructions in the chat always take precedence over any project instruction file content.

The Korean rule is loaded as a project instruction. The task is the later user
message, and it does not state a reply language except in `01a0d795-5f71`.
The sentence is byte-identical on both sides of each pair, so it cannot by
itself be why one sibling switched and the other did not. It is the conflict
sitting in every one of these prompts. This note does not claim the model
applied the sentence; the logs do not record an interpretation.

**The repo contract these runs loaded did not contain the Korean section.**
Every loaded `Agents.md` has 0 Hangul syllables. Commit `d83cfe3`
(2026-09-25 23:33 +0900, "docs(agents): make Korean session output a standing
rule in this repo") is not an ancestor of the conflict session's
`head_commit` `dfa47441`. The rule text that was actually in the prompt is the
home file `~/.claude/rules/99-hq-language.md`. Its sha in the logs matches the
file's sha at the time of this reading (`50cd7ba7ad61`). The inode's birth time
on that path is 2026-09-26, so the birth time does not prove the file was
missing at 08:01 UTC when `01a0d795-5f71` failed to load it. The load list is
the evidence for that one session.

---

## 5. Conclusion

The splitter was not identified.

Excluded, for the pairs that actually diverge: the rule being absent, the rule
living only in the system prompt on one side, model id, reasoning effort,
starting `conversation_message_count`, parent prose language, the language of
the spawn description, `session_relationship`, and a Korean tool result
immediately before the switch.

Established as a separate fact, not as the cause of the pair split: a
`general-purpose` run can put Korean in the tool `description` and English in
the prose for the whole session (`fac2`), or Korean in both (`e7d4` after row
1), or English in both (`30cf`). `explore` runs have no description argument,
so the whole choice is in the prose, and that choice still splits (`60ad` vs
`7c24`) on the first row.

Left open:

- The task bodies are not the same string. Line similarity on the sharp pairs
  is about 0.28. No reply-language cue was found in the diff. Something else
  in the wording or the subject could still matter, and this read did not
  isolate it.
- Sampling under one frozen prompt was not run. Temperature is not in the
  session log. A first-row split with no prior tool result (`60ad` English,
  `7c24` Korean), under the same rule bytes, model, effort, system prompt, and
  parent, is the observation a sampling account has to cover. It is not, by
  itself, a measurement that sampling is the cause.

That is the stopping point this note was allowed to reach. Treating the
remaining guess as a finding would be the failure mode.

---

## 6. What to change next

Proposals only. Nothing in this list was implemented here.

1. **Repeat one frozen brief.** Three or more `deepseek-build` subagents, same
   model, same tree, one English task with no reply-language line, scored with
   §2 on both prose and `description`. One run does not separate a sampling
   split from a wording split. This note did not spend those calls.
2. **Stop using a prose-only rate as the compliance number.** `fac2` is 0%
   prose and 251/251 Korean descriptions. A rate that ignores `description`
   will keep calling a partial obey a failure, and the reverse.
3. **Put the language line where the delegated brief cannot outrank it by
   accident.** The precedence sentence in `subagent_prompt.md` makes a project
   file the weaker copy. The brief the child actually sees is English. The
   candidates to test, after the replay in (1), are a line in that brief and a
   line in the subagent system prompt. Which of those moves the rate is not
   known from these logs.
4. **Do not use `session_relationship` to find the parent.** It is written as
   `primary` for these subagent turns. The parent id is on `subagent_spawned`
   in the parent's `updates.jsonl`.
