# Spec 15 — Tool-call repair

| Field | Value |
|-------|--------|
| Status | **ready-for-impl** |
| Philosophy | HARNESS §5 Reasonix; session pairing (Deep Code B) |
| Gate | Part of **G2** (M1 **must**) |
| Tests | **Automated golden + negative required** |

## 1. Behavior

Before dispatching any tool:

1. Parse model tool-call arguments as JSON.  
2. If parse fails or schema mismatch, run **repair pass** (spec-defined limits).  
3. If still invalid → **do not execute**; return structured error to model.  
4. On session load / interrupted turns: ensure every `tool_call` has a matching `tool` result or an explicit `tool_result_interrupted` placeholder before next API call.

### 1.1 Repair pass (allowed)

| Issue | Repair |
|-------|--------|
| Trailing commas in args JSON | Strip and reparse |
| Single-quoted strings | Convert to JSON strings when unambiguous |
| Unescaped control chars in strings | Escape |
| Args as JSON **string** containing object | Parse inner once |
| Missing optional fields | Fill schema defaults if schema says default |
| Unknown fields | Strip if `additionalProperties` false; else keep |

### 1.2 Never repair (fail closed)

- Changing tool **name**  
- Inventing required arguments  
- Executing with partial args when required keys missing  
- Swapping which file path was intended without snippet (M2+)  

### 1.3 Reasoning content pairing

When tools are used under thinking mode (ADR 0005): preserve `reasoning_content` on assistant messages in the transcript for all subsequent API calls until the user turn boundary rules of DeepSeek docs are satisfied.

### 1.4 Limits

- Max repair attempts per tool call: **1** auto-repair then error.  
- Log `repair_applied=true` + original snippet (truncated, redacted) at debug level only.

## 2. Non-goals

- LLM-based “guess the args” second model call in M1  
- Repairing non-tool free text  

## 3. Failure modes

| Case | Behavior |
|------|----------|
| Unrepairable JSON | Tool error result to model; turn continues |
| Missing tool result in transcript | Insert interrupted placeholder; never send unpaired call |
| 400 from API about reasoning_content | Surface; do not spin retry without transcript fix |

## 4. Test plan (automated)

| Test | Expect |
|------|--------|
| `repair_trailing_comma` | becomes valid object |
| `repair_does_not_invent_required` | error, no dispatch |
| `pairing_inserts_interrupted` | load fixture with hole → repaired transcript |
| `no_dispatch_on_invalid` | mock executor not called |

## 5. Implementation notes

- Lives in `dsb-provider-deepseek` + agent loop shared util.  
- Schema validation: use tool definitions registered for the turn.  
