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

### 1.5 Visible synthesis after a tool round

DeepSeek thinking mode can finish a call with `finish_reason=stop`, empty
`content`, and the answer only in `reasoning_content`. That stop is accepted
when no tool result is still waiting for visible text. Another call would
start another thinking round after the model already signalled completion.

A tool result with no visible assistant text after it is the exception
(Reasonix `285272440f`). The loop appends one host user message to the
volatile tail and calls the model once more:

> The previous assistant response finished without any visible answer text. Continue the same task now and provide a concise visible answer to the user. Do not send reasoning only.

A second reasoning-only stop after that message is accepted. A later tool
round earns one new retry. The message stays in the volatile tail, so the
stable prefix does not move. A reasoning-only stop before any tool call in
the turn is not retried.

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
| `synthesis_retry_once_after_tool_round` | tool result, then reasoning-only stop → one retry carrying the §1.5 sentence; the next visible answer is the turn outcome |
| `second_reasoning_only_stop_accepted` | the same shape with a second reasoning-only stop → still one retry, then the turn ends |
| `reasoning_only_stop_without_tools_not_retried` | reasoning-only stop and no tool call → one provider call |

## 5. Implementation notes

- Lives in `dsb-provider-deepseek` + agent loop shared util.  
- Schema validation: use tool definitions registered for the turn.  
