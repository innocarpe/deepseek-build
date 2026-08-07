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
import json
import signal
import sys
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import Any
from urllib.parse import urlparse


def _now() -> int:
    return int(time.time())


def _completion_id() -> str:
    return f"chatcmpl-scripted-{_now()}"


def _sse_text(model: str, text: str) -> bytes:
    """OpenAI-style chat.completion.chunk SSE stream ending with [DONE]."""
    cid = _completion_id()
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
            "usage": {
                "prompt_tokens": 8,
                "completion_tokens": max(1, len(text.split())),
                "total_tokens": 8 + max(1, len(text.split())),
            },
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
        "usage": {
            "prompt_tokens": 8,
            "completion_tokens": max(1, len(text.split())),
            "total_tokens": 8 + max(1, len(text.split())),
        },
    }
    return json.dumps(body, separators=(",", ":")).encode()


def _sse_tool_then_text(model: str, turn: int, text: str) -> bytes:
    """Turn 0: stream a single tool_call; later turns: final text."""
    if turn == 0:
        cid = _completion_id()
        tool_name = "run_terminal_command"
        tool_args = json.dumps(
            {"command": "echo path-a-r0-tool-ok", "description": "G002 scripted probe"}
        )
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
                                        "arguments": "",
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
                        "delta": {
                            "tool_calls": [
                                {
                                    "index": 0,
                                    "function": {"arguments": tool_args},
                                }
                            ]
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
            if state.scenario == "tool-then-text":
                payload = (
                    _sse_tool_then_text(model, n, state.final_text)
                    if stream
                    else _json_text(model, state.final_text)
                )
                # non-stream tool path not needed for agent (always streams)
            else:
                payload = (
                    _sse_text(model, state.final_text)
                    if stream
                    else _json_text(model, state.final_text)
                )

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


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--host", default="127.0.0.1")
    ap.add_argument("--port", type=int, default=0, help="0 = ephemeral")
    ap.add_argument("--wire", required=True, help="JSONL path for request capture")
    ap.add_argument(
        "--scenario",
        choices=("text-pong", "tool-then-text"),
        default="text-pong",
    )
    ap.add_argument(
        "--final-text",
        default="path-a-r0-ok",
        help="Final assistant text content",
    )
    args = ap.parse_args()

    state = ScriptedState(Path(args.wire), args.scenario, args.final_text)
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
