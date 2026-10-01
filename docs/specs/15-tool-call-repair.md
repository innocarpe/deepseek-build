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
4. On session load, before the next API call, and before a live tool batch returns: every assistant `tool_call` has a matching `tool` result.
   - A missing result after the assistant message recorded the call is `tool_result_interrupted` with code `TOOL_OUTCOME_UNKNOWN`. The text says the outcome is unknown and not to retry blindly when the operation may have side effects.
   - A call the live batch never dispatched is `tool_not_started` with code `TOOL_NOT_STARTED`. The text says to retry it if it is still needed.
   - The batch writes those results before it returns, including when a worker panics or a later call fails.

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
| Missing tool result in transcript | Insert `TOOL_OUTCOME_UNKNOWN` (`tool_result_interrupted`); never send an unpaired call |
| Live batch never dispatched the call | Insert `TOOL_NOT_STARTED` before the batch returns |
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
