#!/usr/bin/env python3
"""Hermetic OpenAI-compatible Chat Completions fixture for Path A R0A.

Records every request body to a JSONL wire file and returns deterministic
streaming (SSE) or non-streaming responses. Used by
scripts/test-path-a-public-entry-e2e.sh (G002+).

Usage:
  python3 scripts/lib/scripted_deepseek_server.py \\
    --wire /tmp/wire.jsonl --port 0 --scenario text-pong

Prints "READY host:port" on stdout when listening. SIGTERM/SIGINT to stop.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import re
import signal
import sys
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import Any
from urllib.parse import urlparse

# ADR 0010 / Spec 45: snp_ + 26 Crockford base32 ULID chars
_SNIPPET_ID_RE = re.compile(r"snippet_id:\s*(snp_[0-9A-HJKMNP-TV-Z]{26})")


def _tool_results_after_user_query(msgs: list[Any]) -> int:
    """Count tool/function role messages after the primary user_query."""
    uq = -1
    for i, m in enumerate(msgs):
        if isinstance(m, dict) and "user_query" in str(m.get("content") or ""):
            uq = i
    return sum(
        1
        for m in msgs[uq + 1 :]
        if isinstance(m, dict) and m.get("role") in ("tool", "function")
    )


def _tool_contents_after_user_query(msgs: list[Any]) -> list[str]:
    uq = -1
    for i, m in enumerate(msgs):
        if isinstance(m, dict) and "user_query" in str(m.get("content") or ""):
            uq = i
    out: list[str] = []
    for m in msgs[uq + 1 :]:
        if isinstance(m, dict) and m.get("role") in ("tool", "function"):
            out.append(str(m.get("content") or ""))
    return out


def _extract_snippet_id(content: str) -> str | None:
    m = _SNIPPET_ID_RE.search(content or "")
    return m.group(1) if m else None


def _latest_snippet_id(msgs: list[Any]) -> str | None:
    """Most recent minted snippet_id in tool results after user_query."""
    for content in reversed(_tool_contents_after_user_query(msgs)):
        sid = _extract_snippet_id(content)
        if sid:
            return sid
    return None


def _file_sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def _now() -> int:
    return int(time.time())


def _completion_id() -> str:
    return f"chatcmpl-scripted-{_now()}"


def _usage_block(text: str, *, prompt_tokens: int = 100) -> dict[str, Any]:
    """DeepSeek-style usage with cache hit/miss fields (ADR 0005 / V2-cache).

    Hermetic fixture: most prompt tokens are reported as cache hits so Path A
    can exercise mapping → status chip / stamp without a live provider.
    """
    completion = max(1, len(text.split()))
    hit = max(0, prompt_tokens - 20)
    miss = prompt_tokens - hit
    return {
        "prompt_tokens": prompt_tokens,
        "completion_tokens": completion,
        "total_tokens": prompt_tokens + completion,
        # DeepSeek top-level fields (not prompt_tokens_details.cached_tokens).
        "prompt_cache_hit_tokens": hit,
        "prompt_cache_miss_tokens": miss,
    }


def _sse_text(model: str, text: str) -> bytes:
    """OpenAI-style chat.completion.chunk SSE stream ending with [DONE]."""
    cid = _completion_id()
    usage = _usage_block(text)
    chunks = [
        {
            "id": cid,
            "object": "chat.completion.chunk",
            "created": _now(),
            "model": model,
            "choices": [
                {
                    "index": 0,
                    "delta": {"role": "assistant", "content": ""},
                    "finish_reason": None,
                }
            ],
        },
        {
            "id": cid,
            "object": "chat.completion.chunk",
            "created": _now(),
            "model": model,
            "choices": [
                {
                    "index": 0,
                    "delta": {"content": text},
                    "finish_reason": None,
                }
            ],
        },
        {
            "id": cid,
            "object": "chat.completion.chunk",
            "created": _now(),
            "model": model,
            "choices": [{"index": 0, "delta": {}, "finish_reason": "stop"}],
            "usage": usage,
        },
    ]
    out = b""
    for ch in chunks:
        out += b"data: " + json.dumps(ch, separators=(",", ":")).encode() + b"\n\n"
    out += b"data: [DONE]\n\n"
    return out


def _json_text(model: str, text: str) -> bytes:
    body = {
        "id": _completion_id(),
        "object": "chat.completion",
        "created": _now(),
        "model": model,
        "choices": [
            {
                "index": 0,
                "message": {"role": "assistant", "content": text},
                "finish_reason": "stop",
            }
        ],
        "usage": _usage_block(text),
    }
    return json.dumps(body, separators=(",", ":")).encode()


def _sse_tool_then_text(
    model: str,
    turn: int,
    text: str,
    *,
    tool_name: str = "run_terminal_command",
    tool_args: str | None = None,
) -> bytes:
    """Turn 0: stream a single tool_call; later turns: final text."""
    if turn == 0:
        cid = _completion_id()
        if tool_args is None:
            if tool_name in ("read_file", "Read", "read"):
                # Grok schema renames path → target_file
                tool_args = json.dumps({"target_file": "mint.txt"})
            else:
                tool_args = json.dumps(
                    {
                        "command": "echo path-a-r0-tool-ok",
                        "description": "G002 scripted probe",
                    }
                )
        # Emit tool_call in one delta with full JSON arguments (avoids
        # partial-arg parse failures on some clients).
        chunks = [
            {
                "id": cid,
                "object": "chat.completion.chunk",
                "created": _now(),
                "model": model,
                "choices": [
                    {
                        "index": 0,
                        "delta": {
                            "role": "assistant",
                            "content": None,
                            "tool_calls": [
                                {
                                    "index": 0,
                                    "id": "call_scripted_1",
                                    "type": "function",
                                    "function": {
                                        "name": tool_name,
                                        "arguments": tool_args,
                                    },
                                }
                            ],
                        },
                        "finish_reason": None,
                    }
                ],
            },
            {
                "id": cid,
                "object": "chat.completion.chunk",
                "created": _now(),
                "model": model,
                "choices": [
                    {
                        "index": 0,
                        "delta": {},
                        "finish_reason": "tool_calls",
                    }
                ],
            },
        ]
        out = b""
        for ch in chunks:
            out += b"data: " + json.dumps(ch, separators=(",", ":")).encode() + b"\n\n"
        out += b"data: [DONE]\n\n"
        return out
    return _sse_text(model, text)


class ScriptedState:
    def __init__(self, wire_path: Path, scenario: str, final_text: str) -> None:
        self.wire_path = wire_path
        self.scenario = scenario
        self.final_text = final_text
        self.tool_name = "run_terminal_command"
        self.tool_args: str | None = None
        self.liveness_dir: Path | None = None
        self.liveness_step = 0
        self.lock = threading.Lock()
        self.request_count = 0
        self.wire_path.parent.mkdir(parents=True, exist_ok=True)
        # truncate on start
        self.wire_path.write_text("", encoding="utf-8")

    def record(self, method: str, path: str, headers: dict[str, str], body: Any) -> int:
        with self.lock:
            n = self.request_count
            self.request_count += 1
            rec = {
                "n": n,
                "ts": time.time(),
                "kind": "request",
                "method": method,
                "path": path,
                "headers": {
                    k: ("REDACTED" if k.lower() == "authorization" else v)
                    for k, v in headers.items()
                },
                "body": body,
            }
            with self.wire_path.open("a", encoding="utf-8") as f:
                f.write(json.dumps(rec, ensure_ascii=False) + "\n")
            return n

    def record_response_usage(
        self, n: int, path: str, model: str, usage: dict[str, Any]
    ) -> None:
        """VC009: append response-side usage so Path A e2e can assert cache fields."""
        with self.lock:
            rec = {
                "n": n,
                "ts": time.time(),
                "kind": "response_usage",
                "path": path,
                "model": model,
                "usage": usage,
            }
            with self.wire_path.open("a", encoding="utf-8") as f:
                f.write(json.dumps(rec, ensure_ascii=False) + "\n")


def make_handler(state: ScriptedState):
    class Handler(BaseHTTPRequestHandler):
        protocol_version = "HTTP/1.1"

        def log_message(self, fmt: str, *args: Any) -> None:  # quiet
            sys.stderr.write("[scripted-ds] " + (fmt % args) + "\n")

        def _read_json(self) -> Any:
            length = int(self.headers.get("Content-Length") or "0")
            raw = self.rfile.read(length) if length else b"{}"
            if not raw:
                return {}
            try:
                return json.loads(raw.decode("utf-8"))
            except json.JSONDecodeError:
                return {"_raw": raw.decode("utf-8", errors="replace")}

        def do_GET(self) -> None:  # noqa: N802
            parsed = urlparse(self.path)
            if parsed.path in ("/health", "/"):
                body = b'{"ok":true,"service":"scripted-deepseek"}\n'
                self.send_response(200)
                self.send_header("Content-Type", "application/json")
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)
                return
            self.send_error(404, "not found")

        def do_POST(self) -> None:  # noqa: N802
            parsed = urlparse(self.path)
            path = parsed.path
            body = self._read_json()
            headers = {k: v for k, v in self.headers.items()}
            n = state.record("POST", path, headers, body)

            # Accept /chat/completions and /v1/chat/completions
            if not path.endswith("/chat/completions") and path != "/chat/completions":
                err = json.dumps(
                    {
                        "error": {
                            "message": f"unsupported path {path}",
                            "type": "invalid_request_error",
                        }
                    }
                ).encode()
                self.send_response(404)
                self.send_header("Content-Type", "application/json")
                self.send_header("Content-Length", str(len(err)))
                self.end_headers()
                self.wfile.write(err)
                return

            model = "deepseek-v4-flash"
            if isinstance(body, dict) and isinstance(body.get("model"), str):
                model = body["model"]

            stream = bool(body.get("stream")) if isinstance(body, dict) else False
            # Decide response shape from message history (not raw request count):
            # session-title side-calls must not consume the tool turn.
            msgs = body.get("messages") if isinstance(body, dict) else None
            msgs = msgs if isinstance(msgs, list) else []
            has_tool_result = any(
                isinstance(m, dict) and m.get("role") in ("tool", "function") for m in msgs
            )
            tool_choice = body.get("tool_choice") if isinstance(body, dict) else None
            forced_fn = None
            if isinstance(tool_choice, dict) and tool_choice.get("type") == "function":
                fn = tool_choice.get("function") or {}
                if isinstance(fn, dict) and isinstance(fn.get("name"), str):
                    forced_fn = fn["name"]

            if forced_fn == "session_title":
                payload = (
                    _sse_tool_then_text(
                        model,
                        0,
                        state.final_text,
                        tool_name="session_title",
                        tool_args=json.dumps({"session_title": "Path A liveness"}),
                    )
                    if stream
                    else _json_text(model, "Path A liveness")
                )
            elif state.scenario == "repair-trailing-comma" and state.liveness_dir is not None:
                # Emit search_replace with trailing-comma JSON args (Spec 15 repair target).
                import hashlib

                uq = -1
                for i, m in enumerate(msgs):
                    if isinstance(m, dict) and "user_query" in str(m.get("content") or ""):
                        uq = i
                tool_results = sum(
                    1
                    for m in msgs[uq + 1 :]
                    if isinstance(m, dict) and m.get("role") in ("tool", "function")
                )
                a = state.liveness_dir / "a.txt"
                if tool_results == 0 and a.exists():
                    ver = hashlib.sha256(a.read_bytes()).hexdigest()
                    # Deliberately malformed: trailing comma after last field
                    bad_args = (
                        '{"file_path":"a.txt","old_string":"hello",'
                        f'"new_string":"hello-repaired","file_version":"{ver}",'
                        "}"
                    )
                    payload = (
                        _sse_tool_then_text(
                            model,
                            0,
                            state.final_text,
                            tool_name="search_replace",
                            tool_args=bad_args,
                        )
                        if stream
                        else _json_text(model, state.final_text)
                    )
                else:
                    payload = (
                        _sse_text(model, "repair-ok")
                        if stream
                        else _json_text(model, "repair-ok")
                    )
            elif state.scenario == "write-deny":
                # One search_replace empty-old overwrite attempt → final text.
                uq = -1
                for i, m in enumerate(msgs):
                    if isinstance(m, dict) and "user_query" in str(m.get("content") or ""):
                        uq = i
                tool_results = sum(
                    1
                    for m in msgs[uq + 1 :]
                    if isinstance(m, dict) and m.get("role") in ("tool", "function")
                )
                if tool_results == 0:
                    payload = (
                        _sse_tool_then_text(
                            model,
                            0,
                            state.final_text,
                            tool_name="search_replace",
                            tool_args=json.dumps(
                                {
                                    "file_path": "existing.txt",
                                    "old_string": "",
                                    "new_string": "OVERWRITE_ATTEMPT\n",
                                }
                            ),
                        )
                        if stream
                        else _json_text(model, state.final_text)
                    )
                else:
                    payload = (
                        _sse_text(model, "write-deny-ok")
                        if stream
                        else _json_text(model, "write-deny-ok")
                    )
            elif state.scenario == "bash-stale" and state.liveness_dir is not None:
                import hashlib

                uq = -1
                for i, m in enumerate(msgs):
                    if isinstance(m, dict) and "user_query" in str(m.get("content") or ""):
                        uq = i
                tool_results = sum(
                    1
                    for m in msgs[uq + 1 :]
                    if isinstance(m, dict) and m.get("role") in ("tool", "function")
                )
                a_path = state.liveness_dir / "a.txt"
                # Step0: bash mutates a.txt (invalidates prior version)
                # Step1: search_replace with STALE version captured before bash
                #         (server embeds stale hash from initial file bytes)
                if tool_results == 0:
                    payload = (
                        _sse_tool_then_text(
                            model,
                            0,
                            state.final_text,
                            # Product surface may expose either id; prefer run_terminal_command
                            # (also accepted as execute-kind alias on some builds).
                            tool_name="run_terminal_command",
                            tool_args=json.dumps(
                                {
                                    "command": "printf 'mutated-by-bash\\n' > a.txt",
                                    "description": "G005 bash mutation of a.txt",
                                }
                            ),
                        )
                        if stream
                        else _json_text(model, state.final_text)
                    )
                elif tool_results == 1:
                    # Stale version = hash of original content (pre-bash)
                    stale = hashlib.sha256(b"original\n").hexdigest()
                    payload = (
                        _sse_tool_then_text(
                            model,
                            0,
                            state.final_text,
                            tool_name="search_replace",
                            tool_args=json.dumps(
                                {
                                    "file_path": "a.txt",
                                    "old_string": "mutated-by-bash",
                                    "new_string": "should-fail",
                                    "file_version": stale,
                                }
                            ),
                        )
                        if stream
                        else _json_text(model, state.final_text)
                    )
                else:
                    payload = (
                        _sse_text(model, "bash-stale-ok")
                        if stream
                        else _json_text(model, "bash-stale-ok")
                    )
            elif state.scenario == "liveness-3edits" and state.liveness_dir is not None:
                # Historical G004 path: file_version-only (pre-VC004 require).
                # Prefer VC006 scenarios snippet-multiedit / snippet-stale-id for Spec 45.
                tool_results = _tool_results_after_user_query(msgs)

                def file_ver(name: str) -> str:
                    p = state.liveness_dir / name
                    return _file_sha256(p)

                steps = [
                    (
                        "search_replace",
                        {
                            "file_path": "a.txt",
                            "old_string": "hello",
                            "new_string": "hello1",
                            "file_version": file_ver("a.txt")
                            if (state.liveness_dir / "a.txt").exists()
                            else "",
                        },
                    ),
                    (
                        "search_replace",
                        {
                            "file_path": "b.txt",
                            "old_string": "world",
                            "new_string": "world1",
                            "file_version": file_ver("b.txt")
                            if (state.liveness_dir / "b.txt").exists()
                            else "",
                        },
                    ),
                    (
                        "search_replace",
                        {
                            "file_path": "a.txt",
                            "old_string": "hello1",
                            "new_string": "hello2",
                            "file_version": file_ver("a.txt")
                            if (state.liveness_dir / "a.txt").exists()
                            else "",
                        },
                    ),
                ]
                if tool_results < len(steps):
                    tname, targs = steps[tool_results]
                    if tname == "search_replace" and (
                        state.liveness_dir / targs["file_path"]
                    ).exists():
                        targs = dict(targs)
                        targs["file_version"] = file_ver(targs["file_path"])
                    payload = (
                        _sse_tool_then_text(
                            model,
                            0,
                            state.final_text,
                            tool_name=tname,
                            tool_args=json.dumps(targs),
                        )
                        if stream
                        else _json_text(model, state.final_text)
                    )
                else:
                    payload = (
                        _sse_text(model, "liveness-ok")
                        if stream
                        else _json_text(model, "liveness-ok")
                    )
            elif state.scenario == "snippet-multiedit" and state.liveness_dir is not None:
                # VC006 Path A R0A: read_file mints snippet_id; search_replace uses it.
                # Sequence (re-read after each edit because VC005 expire_path):
                # 0 read a · 1 edit a hello->hello1 · 2 read b · 3 edit b world->world1
                # 4 read a · 5 edit a hello1->hello2 · then final text
                tool_results = _tool_results_after_user_query(msgs)
                latest_sid = _latest_snippet_id(msgs)

                def emit_tool(tname: str, targs: dict[str, Any]) -> bytes:
                    return (
                        _sse_tool_then_text(
                            model,
                            0,
                            state.final_text,
                            tool_name=tname,
                            tool_args=json.dumps(targs),
                        )
                        if stream
                        else _json_text(model, state.final_text)
                    )

                if tool_results == 0:
                    payload = emit_tool(
                        "read_file", {"target_file": "a.txt"}
                    )
                elif tool_results == 1:
                    if not latest_sid:
                        payload = (
                            _sse_text(model, "snippet-multiedit-FAIL-no-snippet-id")
                            if stream
                            else _json_text(model, "snippet-multiedit-FAIL-no-snippet-id")
                        )
                    else:
                        a = state.liveness_dir / "a.txt"
                        targs: dict[str, Any] = {
                            "file_path": "a.txt",
                            "old_string": "hello",
                            "new_string": "hello1",
                            "snippet_id": latest_sid,
                        }
                        if a.exists():
                            targs["file_version"] = _file_sha256(a)
                        payload = emit_tool("search_replace", targs)
                elif tool_results == 2:
                    payload = emit_tool(
                        "read_file", {"target_file": "b.txt"}
                    )
                elif tool_results == 3:
                    if not latest_sid:
                        payload = (
                            _sse_text(model, "snippet-multiedit-FAIL-no-snippet-id")
                            if stream
                            else _json_text(model, "snippet-multiedit-FAIL-no-snippet-id")
                        )
                    else:
                        b = state.liveness_dir / "b.txt"
                        targs = {
                            "file_path": "b.txt",
                            "old_string": "world",
                            "new_string": "world1",
                            "snippet_id": latest_sid,
                        }
                        if b.exists():
                            targs["file_version"] = _file_sha256(b)
                        payload = emit_tool("search_replace", targs)
                elif tool_results == 4:
                    payload = emit_tool(
                        "read_file", {"target_file": "a.txt"}
                    )
                elif tool_results == 5:
                    if not latest_sid:
                        payload = (
                            _sse_text(model, "snippet-multiedit-FAIL-no-snippet-id")
                            if stream
                            else _json_text(model, "snippet-multiedit-FAIL-no-snippet-id")
                        )
                    else:
                        a = state.liveness_dir / "a.txt"
                        targs = {
                            "file_path": "a.txt",
                            "old_string": "hello1",
                            "new_string": "hello2",
                            "snippet_id": latest_sid,
                        }
                        if a.exists():
                            targs["file_version"] = _file_sha256(a)
                        payload = emit_tool("search_replace", targs)
                else:
                    payload = (
                        _sse_text(model, "snippet-multiedit-ok")
                        if stream
                        else _json_text(model, "snippet-multiedit-ok")
                    )
            elif state.scenario == "snippet-stale-id" and state.liveness_dir is not None:
                # VC006: read → valid edit (expires id) → reuse same id → fail closed.
                tool_results = _tool_results_after_user_query(msgs)
                tool_contents = _tool_contents_after_user_query(msgs)

                def emit_tool(tname: str, targs: dict[str, Any]) -> bytes:
                    return (
                        _sse_tool_then_text(
                            model,
                            0,
                            state.final_text,
                            tool_name=tname,
                            tool_args=json.dumps(targs),
                        )
                        if stream
                        else _json_text(model, state.final_text)
                    )

                if tool_results == 0:
                    payload = emit_tool(
                        "read_file", {"target_file": "a.txt"}
                    )
                elif tool_results == 1:
                    sid = _extract_snippet_id(tool_contents[0]) if tool_contents else None
                    if not sid:
                        payload = (
                            _sse_text(model, "snippet-stale-id-FAIL-no-snippet-id")
                            if stream
                            else _json_text(model, "snippet-stale-id-FAIL-no-snippet-id")
                        )
                    else:
                        # Stash mint id for the deliberate reuse step
                        state._stale_snippet_id = sid  # type: ignore[attr-defined]
                        a = state.liveness_dir / "a.txt"
                        targs = {
                            "file_path": "a.txt",
                            "old_string": "original",
                            "new_string": "edited-once",
                            "snippet_id": sid,
                        }
                        if a.exists():
                            targs["file_version"] = _file_sha256(a)
                        payload = emit_tool("search_replace", targs)
                elif tool_results == 2:
                    stale = getattr(state, "_stale_snippet_id", None)
                    if not stale:
                        # Fall back: first tool content's snippet_id
                        stale = (
                            _extract_snippet_id(tool_contents[0])
                            if tool_contents
                            else None
                        )
                    if not stale:
                        payload = (
                            _sse_text(model, "snippet-stale-id-FAIL-no-stale-id")
                            if stream
                            else _json_text(model, "snippet-stale-id-FAIL-no-stale-id")
                        )
                    else:
                        # Deliberately reuse expired id; attempt further mutation
                        targs = {
                            "file_path": "a.txt",
                            "old_string": "edited-once",
                            "new_string": "should-not-apply",
                            "snippet_id": stale,
                        }
                        payload = emit_tool("search_replace", targs)
                else:
                    payload = (
                        _sse_text(model, "snippet-stale-id-ok")
                        if stream
                        else _json_text(model, "snippet-stale-id-ok")
                    )
            elif state.scenario == "snippet-bash-stale" and state.liveness_dir is not None:
                # VC006 optional: read → bash mutate → edit with old snippet_id fails.
                tool_results = _tool_results_after_user_query(msgs)
                tool_contents = _tool_contents_after_user_query(msgs)

                def emit_tool(tname: str, targs: dict[str, Any]) -> bytes:
                    return (
                        _sse_tool_then_text(
                            model,
                            0,
                            state.final_text,
                            tool_name=tname,
                            tool_args=json.dumps(targs),
                        )
                        if stream
                        else _json_text(model, state.final_text)
                    )

                if tool_results == 0:
                    payload = emit_tool(
                        "read_file", {"target_file": "a.txt"}
                    )
                elif tool_results == 1:
                    sid = _extract_snippet_id(tool_contents[0]) if tool_contents else None
                    state._bash_stale_snippet_id = sid  # type: ignore[attr-defined]
                    payload = emit_tool(
                        "run_terminal_command",
                        {
                            "command": "printf 'mutated-by-bash\\n' > a.txt",
                            "description": "VC006 bash mutation of a.txt",
                        },
                    )
                elif tool_results == 2:
                    stale = getattr(state, "_bash_stale_snippet_id", None)
                    if not stale and tool_contents:
                        stale = _extract_snippet_id(tool_contents[0])
                    if not stale:
                        payload = (
                            _sse_text(model, "snippet-bash-stale-FAIL-no-id")
                            if stream
                            else _json_text(model, "snippet-bash-stale-FAIL-no-id")
                        )
                    else:
                        targs = {
                            "file_path": "a.txt",
                            "old_string": "mutated-by-bash",
                            "new_string": "should-fail",
                            "snippet_id": stale,
                        }
                        payload = emit_tool("search_replace", targs)
                else:
                    payload = (
                        _sse_text(model, "snippet-bash-stale-ok")
                        if stream
                        else _json_text(model, "snippet-bash-stale-ok")
                    )
            elif state.scenario == "tool-then-text":
                if not has_tool_result:
                    payload = (
                        _sse_tool_then_text(
                            model,
                            0,
                            state.final_text,
                            tool_name=state.tool_name,
                            tool_args=state.tool_args,
                        )
                        if stream
                        else _json_text(model, state.final_text)
                    )
                else:
                    payload = (
                        _sse_text(model, state.final_text)
                        if stream
                        else _json_text(model, state.final_text)
                    )
            else:
                payload = (
                    _sse_text(model, state.final_text)
                    if stream
                    else _json_text(model, state.final_text)
                )

            # VC009: surface DeepSeek cache usage on the wire transcript
            # (response_usage line) whenever the payload carries usage.
            usage_rec = _extract_usage_from_payload(payload, stream=stream)
            if usage_rec is not None:
                state.record_response_usage(n, path, model, usage_rec)

            self.send_response(200)
            if stream:
                self.send_header("Content-Type", "text/event-stream")
                self.send_header("Cache-Control", "no-cache")
            else:
                self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(payload)))
            self.end_headers()
            self.wfile.write(payload)

    return Handler


def _extract_usage_from_payload(payload: bytes, *, stream: bool) -> dict[str, Any] | None:
    """Best-effort parse of usage object from SSE or JSON completion body."""
    try:
        if not stream:
            body = json.loads(payload.decode("utf-8"))
            u = body.get("usage") if isinstance(body, dict) else None
            return u if isinstance(u, dict) else None
        # SSE: scan data lines for the last object that carries usage.
        last: dict[str, Any] | None = None
        for line in payload.decode("utf-8", errors="replace").splitlines():
            if not line.startswith("data: "):
                continue
            data = line[len("data: ") :].strip()
            if not data or data == "[DONE]":
                continue
            try:
                obj = json.loads(data)
            except json.JSONDecodeError:
                continue
            if isinstance(obj, dict) and isinstance(obj.get("usage"), dict):
                last = obj["usage"]
        return last
    except Exception:
        return None


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--host", default="127.0.0.1")
    ap.add_argument("--port", type=int, default=0, help="0 = ephemeral")
    ap.add_argument("--wire", required=True, help="JSONL path for request capture")
    ap.add_argument(
        "--scenario",
        choices=(
            "text-pong",
            "tool-then-text",
            "read-file-then-text",
            "liveness-3edits",
            "write-deny",
            "bash-stale",
            "repair-trailing-comma",
            # VC006 Path A Spec 45 snippet_id R0A
            "snippet-multiedit",
            "snippet-stale-id",
            "snippet-bash-stale",
        ),
        default="text-pong",
    )
    ap.add_argument(
        "--liveness-dir",
        default="",
        help=(
            "Workspace dir with a.txt/b.txt for liveness / VC006 snippet scenarios "
            "(hashes and files computed live)"
        ),
    )
    ap.add_argument(
        "--final-text",
        default="path-a-r0-ok",
        help="Final assistant text content",
    )
    ap.add_argument(
        "--tool-name",
        default="",
        help="Override tool name for tool scenarios (default depends on scenario)",
    )
    args = ap.parse_args()

    # Normalize scenario aliases
    scenario = args.scenario
    tool_name = args.tool_name
    if scenario == "read-file-then-text":
        scenario = "tool-then-text"
        if not tool_name:
            tool_name = "read_file"
    if scenario == "tool-then-text" and not tool_name:
        tool_name = "run_terminal_command"

    state = ScriptedState(Path(args.wire), scenario, args.final_text)
    state.tool_name = tool_name
    state.tool_args = None
    state.liveness_dir = Path(args.liveness_dir) if args.liveness_dir else None
    state.liveness_step = 0
    handler = make_handler(state)
    server = ThreadingHTTPServer((args.host, args.port), handler)
    host, port = server.server_address[0], server.server_address[1]

    def _stop(*_a: Any) -> None:
        threading.Thread(target=server.shutdown, daemon=True).start()

    signal.signal(signal.SIGINT, _stop)
    signal.signal(signal.SIGTERM, _stop)

    # READY line is the handshake for the e2e driver
    print(f"READY {host}:{port}", flush=True)
    print(
        f"scripted-deepseek listening on http://{host}:{port} "
        f"scenario={args.scenario} wire={args.wire}",
        file=sys.stderr,
        flush=True,
    )
    try:
        server.serve_forever(poll_interval=0.2)
    finally:
        server.server_close()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
