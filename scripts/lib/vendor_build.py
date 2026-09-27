#!/usr/bin/env python3
"""status / run / prune for the vendored Grok build queues.

The queues this reads
---------------------
Every worktree of this repo builds the vendored tree in its **own** cargo
target, `<worktree>/third_party/grok-build/target`. They used to share the
primary checkout's target; that cannot come back, because cargo names a path
package's artifacts by its path *relative to the workspace root*, so two
worktrees write the same file names into one target, and cargo's mtime-only
freshness then hands the worktree with the older sources the other worktree's
code (measured 2026-09-27).

A cargo target directory serializes builds with a file lock on
`<target>/<profile-dir>/.cargo-lock`: dev, test and check share `debug/`,
release and bench share `release/`. A cargo that waits on that lock prints
nothing, so a queue looks like a hung command in a TUI. `status` therefore
reads *every* worktree's target: one vendored build at a time is a whole-host
rule (memory is the scarce resource), and with per-worktree targets no single
lock file answers it any more.

  status  read the queues. Read-only. Exit 0 free, 1 busy, 2 error.
          Default: every worktree's vendored target. `--target DIR` narrows
          it to one directory (the single-target contract).
  run     run a command now. No vendored build in flight → it runs as-is
          (exec). One in flight → it runs only when the memory gate passes,
          with CARGO_BUILD_JOBS=2; otherwise nothing starts, status is
          printed, and the exit code is 1. The command keeps its own exit
          code when it runs. The target directory is the running command's
          own business: `scripts/vendor-cargo.sh` pins it to the worktree.
  prune   delete personal target copies idle for >= N days (default 3).
          Nothing creates them any more (the queue is per worktree now);
          this clears the ones older sessions left in
          `~/.cache/dsb-vendor-targets`. The base target is never touched.
          Exit 0 done, 1 a delete failed, 2 usage.

The memory gate
---------------------
A second vendored build beside a busy one adds jobs, not replaces them, so
`run` may only start one when the host has room. The gate is free >= 25%
AND load5m <= 18 AND swap <= 4 GiB — one step inside every freeze signal
in `mac-slowdown` §1 (free < 15%, swap > 5 GB, load > 20), and strictly
inside the HQ memory guard's recovery thresholds (free <= 30% and swap
>= 12 GiB, or free <= 10% and swap >= 8 GiB), which fire after damage.
Measured incidents on this 24 GB / 18-core Mac: 2026-09-25 (ld x17 at
~1.2 GB each → 20 GB, swap 8.8 → 15.1 GB, load 8 → 45) and 2026-09-26
12:24 KST (HQ guard severe: swap 17.1 GB, load 46.8). Facts can be
overridden with DSB_HOST_FREE_PERCENT / DSB_HOST_SWAP_GIB /
DSB_HOST_LOAD5M (the hermetic tests use this).

Labels are inferred
-------------------
file-lock ownership is not readable from the process table. Cargo starts
compiling only while it holds the lock, so a lock-file process with live
descendants is the holder (strongest when a descendant is a compiler) and
one with none is waiting. Both are marked "(inferred)" in the output.

No deadlock exists today
------------------------
Nothing under `third_party/grok-build` invokes cargo from inside a cargo
build or test; adding such a call would create a real lock cycle on that
target. status flags a descendant cargo as `nested_cargo` so that shape is
visible if it ever appears.
"""
import argparse
import fcntl
import json
import os
import re
import shutil
import subprocess
import sys
import time
from pathlib import Path

LOCK_NAME = ".cargo-lock"
DEFAULT_PRUNE_DAYS = 3
DEFAULT_CLONE_ROOT = "~/.cache/dsb-vendor-targets"
STARTING_GRACE_SECONDS = 3
GATE_FREE_PERCENT_MIN = 25.0
GATE_LOAD5M_MAX = 18.0
GATE_SWAP_GIB_MAX = 4.0
SECOND_BUILD_JOBS = 2
COMPILER_RE = re.compile(
    r"^(rustc|rustc_driver|clippy-driver|rustdoc|cc1|cc1plus|cc|clang|clang\+\+|gcc|g\+\+|ld|ld64|dsymutil|swiftc)(-\d+(\.\d+)*)?$"
)


def fail(message, code=2):
    print(f"vendor-build: error: {message}", file=sys.stderr)
    sys.exit(code)


def script_repo_root():
    return Path(__file__).resolve().parent.parent.parent


def primary_checkout(repo_root):
    for extra in (["--path-format=absolute", "--git-common-dir"], ["--git-common-dir"]):
        try:
            r = subprocess.run(
                ["git", "-C", str(repo_root), "rev-parse", *extra],
                capture_output=True,
                text=True,
            )
        except OSError:
            break
        if r.returncode == 0 and r.stdout.strip():
            p = Path(r.stdout.strip())
            if not p.is_absolute():
                p = repo_root / p
            return p.parent.resolve()
    return repo_root.resolve()


def default_target(primary):
    return primary / "third_party" / "grok-build" / "target"


def default_clone_root():
    return Path(os.path.expanduser(DEFAULT_CLONE_ROOT))


def resolve_target(args):
    if args.target:
        return Path(os.path.expanduser(args.target)).resolve()
    return default_target(primary_checkout(script_repo_root())).resolve()


def resolve_root(args):
    if getattr(args, "root", None):
        return Path(os.path.expanduser(args.root)).resolve()
    return default_clone_root().resolve()


def resolve_repo(args):
    if getattr(args, "repo", None):
        return Path(os.path.expanduser(args.repo)).resolve()
    return primary_checkout(script_repo_root())


# ---------------------------------------------------------------------------
# Host facts and the memory gate
# ---------------------------------------------------------------------------


def _read_text(path):
    try:
        with open(path, encoding="utf-8") as fh:
            return fh.read()
    except OSError:
        return None


def _run_text(argv, timeout=5):
    try:
        r = subprocess.run(argv, capture_output=True, text=True, timeout=timeout)
    except (OSError, subprocess.TimeoutExpired):
        return None
    return r.stdout if r.returncode == 0 else None


def _meminfo_kb(text):
    if not text:
        return None
    try:
        return int(text.split()[0])
    except (ValueError, IndexError):
        return None


def host_facts():
    """Free %, swap used (GiB) and load5m, measured like the HQ memory guard.

    macOS: `/usr/bin/memory_pressure -Q` and `sysctl vm.swapusage`; Linux:
    /proc/meminfo and /proc/loadavg. DSB_HOST_FREE_PERCENT,
    DSB_HOST_SWAP_GIB and DSB_HOST_LOAD5M override the measurement — the
    hermetic tests use them, and a container can feed its host's numbers.
    """
    override = {
        "free_percent": os.environ.get("DSB_HOST_FREE_PERCENT"),
        "swap_used_gib": os.environ.get("DSB_HOST_SWAP_GIB"),
        "load5m": os.environ.get("DSB_HOST_LOAD5M"),
    }
    if any(raw is not None for raw in override.values()):
        facts = {"source": "override"}
        for key, raw in override.items():
            try:
                facts[key] = float(raw) if raw is not None else None
            except ValueError:
                facts[key] = None
        return facts

    facts = {"free_percent": None, "swap_used_gib": None, "load5m": None, "source": "measured"}
    if sys.platform == "darwin":
        text = _run_text(["/usr/bin/memory_pressure", "-Q"])
        m = re.search(r"free percentage:\s+(\d+)%", text or "")
        if m:
            facts["free_percent"] = float(m.group(1))
        text = _run_text(["/usr/sbin/sysctl", "-n", "vm.swapusage"])
        m = re.search(r"used = ([\d.]+)M", text or "")
        if m:
            facts["swap_used_gib"] = round(float(m.group(1)) / 1024.0, 2)
    elif sys.platform.startswith("linux"):
        text = _read_text("/proc/meminfo") or ""
        fields = {}
        for line in text.splitlines():
            parts = line.split(":")
            if len(parts) == 2:
                fields[parts[0].strip()] = parts[1].strip()
        total = _meminfo_kb(fields.get("MemTotal"))
        available = _meminfo_kb(fields.get("MemAvailable"))
        if total and available is not None:
            facts["free_percent"] = round(available * 100.0 / total, 1)
        swap_total = _meminfo_kb(fields.get("SwapTotal"))
        swap_free = _meminfo_kb(fields.get("SwapFree"))
        if swap_total is not None and swap_free is not None:
            facts["swap_used_gib"] = round((swap_total - swap_free) / 1048576.0, 2)
    try:
        facts["load5m"] = round(os.getloadavg()[1], 2)
    except (OSError, AttributeError):
        pass
    return facts


def evaluate_gate(facts):
    reasons = []
    free_percent = facts.get("free_percent")
    load5m = facts.get("load5m")
    swap_used_gib = facts.get("swap_used_gib")
    if free_percent is None or load5m is None or swap_used_gib is None:
        reasons.append("host facts unavailable; refusing to add a second build blind")
    else:
        if free_percent < GATE_FREE_PERCENT_MIN:
            reasons.append(f"free {free_percent:g}% < {GATE_FREE_PERCENT_MIN:g}%")
        if load5m > GATE_LOAD5M_MAX:
            reasons.append(f"load5m {load5m:g} > {GATE_LOAD5M_MAX:g}")
        if swap_used_gib > GATE_SWAP_GIB_MAX:
            reasons.append(f"swap {swap_used_gib:g} GiB > {GATE_SWAP_GIB_MAX:g} GiB")
    return {
        "allowed": not reasons,
        "reasons": reasons,
        "thresholds": {
            "free_percent_min": GATE_FREE_PERCENT_MIN,
            "load5m_max": GATE_LOAD5M_MAX,
            "swap_used_gib_max": GATE_SWAP_GIB_MAX,
            "second_build_jobs": SECOND_BUILD_JOBS,
        },
    }


def host_line(facts, gate):
    free_percent = facts.get("free_percent")
    swap_used_gib = facts.get("swap_used_gib")
    load5m = facts.get("load5m")
    free_s = f"{free_percent:g}%" if free_percent is not None else "?"
    swap_s = f"{swap_used_gib:g} GiB" if swap_used_gib is not None else "?"
    load_s = f"{load5m:g}" if load5m is not None else "?"
    source = " (override)" if facts.get("source") == "override" else ""
    if gate["allowed"]:
        verdict = "ALLOWED"
    else:
        verdict = "DENIED (" + "; ".join(gate["reasons"]) + ")"
    return f"host: free {free_s} · swap {swap_s} · load5m {load_s}{source} → 2nd build: {verdict}"


def busy_hint(busy):
    if busy:
        return (
            "vendor-build: BUSY → exit 1 — a vendored build is in flight and one runs at a "
            "time; wait for it, or run yours through `./scripts/vendor-cargo.sh "
            "--allow-concurrent` when the memory gate above allows a second build"
        )
    return "vendor-build: FREE → exit 0"


# ---------------------------------------------------------------------------
# Process and lock facts
# ---------------------------------------------------------------------------


def ps_snapshot():
    r = subprocess.run(
        ["ps", "-axww", "-o", "pid=,ppid=,etime=,command="],
        capture_output=True,
        text=True,
    )
    if r.returncode != 0:
        fail(f"ps failed: {r.stderr.strip()}")
    procs = {}
    for line in r.stdout.splitlines():
        parts = line.strip().split(None, 3)
        if len(parts) < 4:
            continue
        try:
            pid, ppid = int(parts[0]), int(parts[1])
        except ValueError:
            continue
        procs[pid] = {"pid": pid, "ppid": ppid, "etime": parts[2], "command": parts[3]}
    return procs


def comm_of(command):
    first = command.split()[0] if command.split() else ""
    return os.path.basename(first)


def descendants(procs, pid):
    kids = {}
    for p in procs.values():
        kids.setdefault(p["ppid"], []).append(p["pid"])
    out, stack = [], list(kids.get(pid, []))
    while stack:
        c = stack.pop()
        out.append(c)
        stack.extend(kids.get(c, []))
    return out


def parse_etime(text):
    parts = text.split("-")
    days = 0
    if len(parts) == 2:
        try:
            days = int(parts[0])
        except ValueError:
            return None
        t = parts[1]
    else:
        t = parts[0]
    nums = []
    for chunk in t.split(":"):
        try:
            nums.append(int(chunk))
        except ValueError:
            return None
    if len(nums) == 2:
        hh, mm, ss = 0, nums[0], nums[1]
    elif len(nums) == 3:
        hh, mm, ss = nums
    else:
        return None
    return days * 86400 + hh * 3600 + mm * 60 + ss


def classify(procs, desc_ids, elapsed):
    desc = [{"pid": pid, "comm": comm_of(procs[pid]["command"])} for pid in desc_ids if pid in procs]
    compilers = [d for d in desc if COMPILER_RE.match(d["comm"])]
    nested = [d for d in desc if d["comm"] == "cargo"]
    if compilers:
        d = compilers[0]
        label, basis = "holder", f'compiler descendant pid {d["pid"]} ({d["comm"]})'
    elif desc:
        d = desc[0]
        label, basis = "holder", f'live descendant pid {d["pid"]} ({d["comm"]}); no compiler process'
    elif elapsed is not None and elapsed < STARTING_GRACE_SECONDS:
        label, basis = "unknown", f"started {elapsed}s ago; no descendants yet"
    else:
        label, basis = "waiter", "no descendants"
    return label, basis, desc, nested


def discover_locks(target):
    """Every `<target>/<profile-dir>/.cargo-lock` that exists."""
    locks = []
    if not target.is_dir():
        return locks
    try:
        children = sorted(target.iterdir())
    except OSError:
        return locks
    for child in children:
        candidate = child / LOCK_NAME
        try:
            if candidate.is_file():
                locks.append((child.name, candidate))
        except OSError:
            continue
    return locks


def lock_pids(lock_path):
    if shutil.which("lsof") is None:
        fail("lsof is not on PATH; status reads lock holders with it")
    r = subprocess.run(["lsof", "-t", str(lock_path)], capture_output=True, text=True)
    pids = []
    for chunk in r.stdout.split():
        try:
            pids.append(int(chunk))
        except ValueError:
            continue
    return pids


def lock_holders_cwd(pids):
    if not pids:
        return {}
    r = subprocess.run(
        ["lsof", "-a", "-p", ",".join(str(p) for p in pids), "-d", "cwd", "-Fn"],
        capture_output=True,
        text=True,
    )
    cur, out = None, {}
    for line in r.stdout.splitlines():
        if line.startswith("p"):
            try:
                cur = int(line[1:])
            except ValueError:
                cur = None
        elif line.startswith("n") and cur is not None:
            out[cur] = line[1:]
    return out


def probe_lock(path):
    """Try a shared, non-blocking file lock: 'locked' when someone holds it."""
    try:
        fh = open(path, "rb")
    except OSError:
        return "missing"
    try:
        try:
            fcntl.flock(fh, fcntl.LOCK_SH | fcntl.LOCK_NB)
        except OSError:
            return "locked"
        fcntl.flock(fh, fcntl.LOCK_UN)
        return "free"
    finally:
        fh.close()


def worktree_roots(primary):
    try:
        r = subprocess.run(
            ["git", "-C", str(primary), "worktree", "list", "--porcelain"],
            capture_output=True,
            text=True,
        )
    except OSError:
        return []
    if r.returncode != 0:
        return []
    roots = []
    for line in r.stdout.splitlines():
        if line.startswith("worktree "):
            roots.append(line[len("worktree "):].rstrip("/"))
    return roots


def map_worktree(cwd, roots):
    if not cwd:
        return None
    for root in roots:
        if cwd == root or cwd.startswith(root + "/"):
            return root
    return None


# ---------------------------------------------------------------------------
# status
# ---------------------------------------------------------------------------


def collect_locks(target, wt_roots):
    procs = ps_snapshot()
    reports = []
    busy = False
    for profile, path in discover_locks(target):
        seen = [p for p in lock_pids(path) if p != os.getpid()]
        probe = probe_lock(path)
        cwd_map = lock_holders_cwd(seen)
        entries = []
        for pid in sorted(seen):
            proc = procs.get(pid)
            cwd = cwd_map.get(pid)
            wt = map_worktree(cwd, wt_roots)
            if proc is None:
                entries.append({
                    "pid": pid, "ppid": None, "etime": None, "elapsed_seconds": None,
                    "command": "(exited while reading)", "cwd": cwd, "worktree": wt,
                    "label": "unknown", "basis": "process exited between lsof and ps",
                    "descendants": [], "nested_cargo": [],
                })
                continue
            elapsed = parse_etime(proc["etime"])
            label, basis, desc, nested = classify(procs, descendants(procs, pid), elapsed)
            entries.append({
                "pid": pid, "ppid": proc["ppid"], "etime": proc["etime"],
                "elapsed_seconds": elapsed, "command": proc["command"],
                "cwd": cwd, "worktree": wt,
                "label": label, "basis": basis,
                "descendants": desc, "nested_cargo": nested,
            })
        lock_busy = bool(seen) or probe == "locked"
        busy = busy or lock_busy
        reports.append({
            "profile": profile,
            "path": str(path),
            "probe": probe,
            "busy": lock_busy,
            "processes": entries,
        })
    return reports, busy


def worktree_targets(primary):
    """(worktree, vendored target) for every worktree of this repo, git's order."""
    return [
        (wt, Path(wt) / "third_party" / "grok-build" / "target")
        for wt in worktree_roots(primary)
    ]


def collect_machine(primary, wt_roots):
    """Per-worktree reports for every vendored target. Returns (entries, busy)."""
    entries = []
    busy = False
    for wt, target in worktree_targets(primary):
        if not target.is_dir():
            entries.append({"worktree": wt, "target": str(target), "state": "absent", "locks": []})
            continue
        reports, wt_busy = collect_locks(target, wt_roots)
        busy = busy or wt_busy
        entries.append({
            "worktree": wt,
            "target": str(target),
            "state": "busy" if wt_busy else "free",
            "locks": reports,
        })
    return entries, busy


def cmd_status(args):
    primary = resolve_repo(args)
    wt_roots = worktree_roots(primary)
    facts = host_facts()
    gate = evaluate_gate(facts)
    note = "holder/waiter labels are inferred from live descendants, not read from the file lock"

    if args.target:
        target = Path(os.path.expanduser(args.target)).resolve()
        reports, busy = collect_locks(target, wt_roots)
        payload = {
            "target": str(target),
            "status": "busy" if busy else "free",
            "exit_code": 1 if busy else 0,
            "host": facts,
            "second_build": gate,
            "locks": reports,
            "note": note,
        }
        if args.json:
            print(json.dumps(payload, indent=2))
        else:
            print(render_status(target, reports, busy, facts, gate))
        return 1 if busy else 0

    entries, busy = collect_machine(primary, wt_roots)
    payload = {
        "repo": str(primary),
        "worktrees": len(entries),
        "targets": entries,
        "status": "busy" if busy else "free",
        "exit_code": 1 if busy else 0,
        "host": facts,
        "second_build": gate,
        "note": note,
    }
    if args.json:
        print(json.dumps(payload, indent=2))
    else:
        print(render_machine_status(primary, entries, busy, facts, gate))
    return 1 if busy else 0


def render_locks(reports, indent="  "):
    lines = []
    for r in reports:
        if r["probe"] == "locked" or r["processes"]:
            state = "BUSY (file lock held)" if r["probe"] == "locked" else "BUSY (process on the lock file)"
        else:
            state = "free"
        lines.append(f"{indent}lock {r['profile']}/{LOCK_NAME}: {state}")
        for p in r["processes"]:
            etime = p["etime"] or "?"
            lines.append(f"{indent}  pid {p['pid']}  etime {etime}  {p['label']} (inferred) — {p['basis']}")
            lines.append(f"{indent}        cmd: {p['command']}")
            if p["cwd"]:
                lines.append(f"{indent}        cwd: {p['cwd']}")
            if p["worktree"]:
                lines.append(f"{indent}        worktree: {p['worktree']}")
            for n in p.get("nested_cargo", []):
                lines.append(
                    f"{indent}        WARNING nested cargo pid {n['pid']} — "
                    "cargo inside cargo on one target is a lock cycle"
                )
    return lines


def render_status(target, reports, busy, facts, gate):
    lines = [f"target: {target}", host_line(facts, gate)]
    if not reports:
        if target.is_dir():
            lines.append(f"locks: none — no {LOCK_NAME} under any profile directory (nothing has built here)")
        else:
            lines.append(f"locks: none — {target} does not exist (nothing is building there)")
    lines.extend(render_locks(reports))
    lines.append(busy_hint(busy))
    return "\n".join(lines)


def render_machine_status(primary, entries, busy, facts, gate):
    lines = [f"repo: {primary} ({len(entries)} worktree(s), each with its own vendored target)", host_line(facts, gate)]
    for e in entries:
        lines.append(f"worktree {e['worktree']}")
        if e["state"] == "absent":
            lines.append(f"  target: {e['target']} — not built yet (nothing is building there)")
            continue
        state = "BUSY" if e["state"] == "busy" else "free"
        lines.append(f"  target: {e['target']} — {state}")
        lines.extend(render_locks(e["locks"], indent="  "))
    lines.append(busy_hint(busy))
    return "\n".join(lines)


# ---------------------------------------------------------------------------
# run
# ---------------------------------------------------------------------------


def cmd_run(args):
    primary = resolve_repo(args)
    wt_roots = worktree_roots(primary)
    cmd = list(args.cmd)
    while cmd and cmd[0] == "--":
        cmd.pop(0)
    if not cmd:
        fail("run needs a command after `--`, e.g. `vendor-build.sh run -- cargo test -p <pkg>`")

    entries, busy = collect_machine(primary, wt_roots)
    facts = host_facts()
    gate = evaluate_gate(facts)

    if not busy:
        print("vendor-build: no vendored build in flight — running the command as-is", file=sys.stderr)
        sys.stdout.flush()
        sys.stderr.flush()
        os.execvp(cmd[0], cmd)  # never returns

    if not gate["allowed"]:
        print(render_machine_status(primary, entries, busy, facts, gate))
        print(
            "vendor-build: 2nd build DENIED by the memory gate — nothing started; "
            "wait for the host to recover",
            file=sys.stderr,
        )
        return 1

    env = os.environ.copy()
    env["CARGO_BUILD_JOBS"] = str(SECOND_BUILD_JOBS)
    print(
        f"vendor-build: a vendored build is in flight; memory gate passed — running with "
        f"CARGO_BUILD_JOBS={SECOND_BUILD_JOBS}",
        file=sys.stderr,
    )
    sys.stdout.flush()
    sys.stderr.flush()
    os.execvpe(cmd[0], cmd, env)  # never returns


# ---------------------------------------------------------------------------
# prune
# ---------------------------------------------------------------------------


def overlapping(a, b):
    """True when either path is the other or inside the other."""
    sa, sb = str(a), str(b)
    return sa == sb or sa.startswith(sb + os.sep) or sb.startswith(sa + os.sep)


def scan_mtime(path, cutoff):
    """Newest mtime under path, stopping early once cutoff is reached.

    Returns (newest, stale): stale means nothing was modified at or after the
    cutoff. An unreadable entry is treated as fresh — never delete what we
    cannot read.
    """
    newest = 0.0
    try:
        newest = os.lstat(path).st_mtime
    except OSError:
        return time.time(), False
    if newest >= cutoff:
        return newest, False
    for dirpath, dirnames, filenames in os.walk(path):
        for name in filenames + dirnames:
            p = os.path.join(dirpath, name)
            try:
                m = os.lstat(p).st_mtime
            except OSError:
                return time.time(), False
            if m > newest:
                newest = m
                if newest >= cutoff:
                    return newest, False
    return newest, newest < cutoff


def dir_size_kb(path):
    r = subprocess.run(["du", "-sk", str(path)], capture_output=True, text=True)
    if r.returncode == 0:
        try:
            return int(r.stdout.split()[0])
        except (IndexError, ValueError):
            pass
    return None


def human_kb(kb):
    if kb is None:
        return "unknown size"
    if kb >= 1024 * 1024:
        return f"{kb / 1048576:.1f} GB"
    if kb >= 1024:
        return f"{kb / 1024:.1f} MB"
    return f"{kb} KB"


def human_ago(ts):
    if not ts:
        return "unknown"
    delta = time.time() - ts
    if delta >= 86400:
        days = int(delta // 86400)
        return f"{time.strftime('%Y-%m-%d', time.localtime(ts))} ({days} day(s) ago)"
    if delta >= 3600:
        return f"{int(delta // 3600)} hour(s) ago"
    return f"{int(delta // 60)} minute(s) ago"


def cmd_prune(args):
    root = resolve_root(args)
    target = resolve_target(args)
    if args.days < 0:
        fail("--days must be >= 0")
    if not root.is_dir():
        print(f"vendor-build: nothing to prune — {root} does not exist")
        return 0

    cutoff = time.time() - args.days * 86400
    base_resolved = target.resolve()
    pruned, kept, skipped, failed = [], [], [], []
    for entry in sorted(root.iterdir()):
        if entry.is_symlink() or not entry.is_dir():
            continue
        entry_resolved = entry.resolve()
        if overlapping(entry_resolved, base_resolved):
            skipped.append(entry)
            continue
        newest, stale = scan_mtime(entry, cutoff)
        if not stale:
            kept.append((entry, newest))
            continue
        size_kb = dir_size_kb(entry)
        # macOS can report ENOTEMPTY for a directory rmdir immediately after
        # its children were unlinked; one short retry covers that, while a
        # genuinely undeletable file still fails twice and is reported.
        last_error = None
        for attempt in (1, 2):
            try:
                shutil.rmtree(entry)
                last_error = None
                break
            except OSError as e:
                last_error = e
                if attempt == 1:
                    time.sleep(0.5)
        if last_error is not None:
            print(f"vendor-build: could not remove {entry}: {last_error}", file=sys.stderr)
            failed.append(entry)
            continue
        pruned.append((entry, newest, size_kb))

    for entry, newest, size_kb in pruned:
        print(f"  pruned {entry.name} — {human_kb(size_kb)}, last touched {human_ago(newest)}")
    for entry, newest in kept:
        print(f"  kept   {entry.name} — last touched {human_ago(newest)}")
    for entry in failed:
        print(f"  failed {entry.name} — removal stopped partway; clear the cause and re-run prune")
    for entry in skipped:
        print(f"  skipped base target inside root: {entry}")
    freed = sum(s for _, _, s in pruned if s is not None)
    total = len(pruned) + len(kept) + len(failed)
    print(
        f"vendor-build: pruned {len(pruned)} of {total} clone(s); "
        f"freed ~{human_kb(freed)} (du estimate); the base target was not touched"
    )
    return 1 if failed else 0


# ---------------------------------------------------------------------------
# entry point
# ---------------------------------------------------------------------------


def build_parser():
    parser = argparse.ArgumentParser(
        prog="vendor-build.sh",
        description="Read the vendored Grok build queues, one per worktree.",
    )
    sub = parser.add_subparsers(dest="command", required=True)

    def add_common(p):
        p.add_argument(
            "--target", default=None,
            help=(
                "one target directory to read instead of every worktree's vendored target "
                "(default for `status`: all worktrees; for `prune`: the primary checkout's target)"
            ),
        )
        p.add_argument(
            "--repo", default=None,
            help="repository whose worktrees are read (default: this repo's primary checkout)",
        )

    st = sub.add_parser("status", help="read the vendored build queues (exit 0 free, 1 busy)")
    st.add_argument("--json", action="store_true", help="machine-readable output")
    add_common(st)

    rn = sub.add_parser(
        "run",
        help="run a command now; with a build in flight only when the memory gate passes",
        description=(
            "Run a command now. No vendored build in flight: it runs as-is. One in "
            "flight: it runs with CARGO_BUILD_JOBS=2, but only when the memory gate "
            "passes; otherwise nothing starts, status is printed, and the exit code "
            "is 1. The command's target directory is its own business — "
            "scripts/vendor-cargo.sh pins it to the worktree."
        ),
    )
    rn.add_argument("cmd", nargs=argparse.REMAINDER, help="the command, after `--`")
    add_common(rn)

    pr = sub.add_parser("prune", help="delete personal target copies idle for >= N days")
    pr.add_argument("--days", type=int, default=DEFAULT_PRUNE_DAYS)
    pr.add_argument("--root", default=None, help="copy namespace (default: ~/.cache/dsb-vendor-targets)")
    add_common(pr)
    return parser


def main(argv):
    args = build_parser().parse_args(argv)
    if args.command == "status":
        return cmd_status(args)
    if args.command == "run":
        return cmd_run(args)
    if args.command == "prune":
        return cmd_prune(args)
    fail(f"unknown command {args.command!r}")


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
