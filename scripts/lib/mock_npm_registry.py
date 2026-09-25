#!/usr/bin/env python3
"""A tiny npm packument server for the verify-retry test.

It exists to reproduce, hermetically, the race that broke the 5.7.0 release:
`npm publish` returns but a read against the registry does not see the new
version yet. The real registry is read through a CDN with its own cache, so a
test cannot ask it to be slow on purpose; this server can.

It answers every request with a packument for one package name, and a version
is only included after `--present-after` seconds have passed since start. npm
resolves `name@version` against that document client-side, so a version the
document does not contain produces npm's own `E404`, exactly as the release
run saw it. Nothing here reaches the network.

Usage:
  mock_npm_registry.py --port-file PATH --name NAME --version V
                       [--present-after SECONDS | --absent] [--log PATH]

It prints nothing to stdout and writes the listening port to --port-file once
the socket is bound, which is the signal callers wait for.
"""

import argparse
import http.server
import json
import os
import socketserver
import sys
import threading
import time


class Handler(http.server.BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def do_GET(self):  # noqa: N802 (BaseHTTPRequestHandler naming)
        state = self.server.state
        state.record(self.path)

        with state.lock:
            present = state.absent is False and (
                state.present_after <= 0
                or (time.time() - state.started) >= state.present_after
            )

        versions = {}
        if present:
            versions[state.version] = {"name": state.name, "version": state.version}
        else:
            # Keep an older release so the document is never empty: the point
            # is a package that exists but does not yet carry the new version.
            older = state.older_version
            versions[older] = {"name": state.name, "version": older}

        latest = state.version if present else state.older_version
        body = json.dumps(
            {
                "name": state.name,
                "dist-tags": {"latest": latest},
                "versions": versions,
            }
        ).encode()

        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, *args):
        pass  # keep the test output clean; --log carries what matters


class State:
    def __init__(self, name, version, older_version, present_after, absent, log_path):
        self.name = name
        self.version = version
        self.older_version = older_version
        self.present_after = present_after
        self.absent = absent
        self.log_path = log_path
        self.started = time.time()
        self.lock = threading.Lock()

    def record(self, path):
        if not self.log_path:
            return
        with self.lock:
            with open(self.log_path, "a", encoding="utf-8") as handle:
                handle.write("%.3f %s\n" % (time.time() - self.started, path))


class Server(socketserver.ThreadingTCPServer):
    allow_reuse_address = True
    daemon_threads = True


def main():
    parser = argparse.ArgumentParser(add_help=True)
    parser.add_argument("--port-file", required=True)
    parser.add_argument("--name", required=True)
    parser.add_argument("--version", required=True)
    parser.add_argument("--older-version", default="0.0.1")
    parser.add_argument("--present-after", type=float, default=0.0)
    parser.add_argument("--absent", action="store_true")
    parser.add_argument("--log", default="")
    args = parser.parse_args()

    server = Server(("127.0.0.1", 0), Handler)
    server.state = State(
        name=args.name,
        version=args.version,
        older_version=args.older_version,
        present_after=args.present_after,
        absent=args.absent,
        log_path=args.log,
    )

    with open(args.port_file, "w", encoding="utf-8") as handle:
        handle.write(str(server.server_address[1]))
        handle.flush()
        os.fsync(handle.fileno())

    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        server.server_close()


if __name__ == "__main__":
    sys.exit(main())
