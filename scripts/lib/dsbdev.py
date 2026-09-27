#!/usr/bin/env python3
"""Helpers behind scripts/dsbdev.sh: per-worktree target seeding.

  dsbdev.py snapshot  --vendor DIR --out FILE
  dsbdev.py seed      --vendor DIR --target DIR
  dsbdev.py summarize --out FILE            (cargo JSON messages on stdin)
  dsbdev.py record    --vendor DIR --target DIR --snapshot FILE --started EPOCH

Why a target per worktree (measured 2026-09-27): cargo names a path
package's artifacts with a hash of its path *relative to the workspace root*,
so every worktree of this repo writes the same file names into a shared
CARGO_TARGET_DIR. A build in one worktree overwrites the other's, and cargo
decides freshness by mtime alone, so a worktree whose edits are older than
another worktree's last build gets that other worktree's code with a no-op
build.

A cold build of the pager is ~1,400 units and ~15 min, so a new worktree's
target is seeded instead of built from scratch:

  record    after a debug build that compiled something, walk the fingerprint
            graph from the pager binary and write
            <target>/debug/dsbdev/manifest.json: the units it used, their
            fingerprint hash-file signatures, the codegen objects the binary
            links, and the git blob of every repository file at build start.
  seed      in a worktree with no manifest, pick the sibling worktree target
            whose recorded sources differ least, clone (APFS clonefile) just
            those units, stamp them all with one mtime, then bump the mtime of
            every file whose content differs from what the donor built. Cargo
            then rebuilds exactly the crates whose sources differ, plus their
            dependents. Not cloned: a unit rebuilt in the donor since its
            manifest, and a unit whose build script watches an absolute path
            in a checkout (the pager's git HEAD). Without any manifest, only
            registry units are cloned from the shared target; its workspace
            units have unknown sources.
"""

import argparse
import ctypes
import errno
import json
import os
import re
import shutil
import stat
import subprocess
import sys
import time

PACKAGE = os.environ.get("DSBDEV_PACKAGE", "xai-grok-pager-bin")
BIN = os.environ.get("DSBDEV_BIN", "xai-grok-pager")
MANIFEST_VERSION = 1
HASH_RE = re.compile(r"-([0-9a-f]{16})(?:\.|$)")
UNIT_RE = re.compile(r"^(.+)-([0-9a-f]{16})$")


def log(message):
    print(f"dsbdev: {message}", file=sys.stderr)


def git(repo, *args, stdin=None):
    return subprocess.run(
        ["git", "-C", repo, *args],
        input=stdin,
        capture_output=True,
        check=True,
    ).stdout


def repo_root(vendor):
    return git(vendor, "rev-parse", "--show-toplevel").decode().strip()


# --- source snapshot ------------------------------------------------------


def source_map(repo):
    """Map repo-relative path -> git blob id of the file as it is on disk."""
    files = {}
    for entry in git(repo, "ls-files", "-s", "-z").split(b"\0"):
        if not entry:
            continue
        meta, path = entry.split(b"\t", 1)
        mode, blob, _stage = meta.split()
        if mode == b"160000":  # submodule
            continue
        files[path.decode()] = blob.decode()
    changed = [
        p for p in git(repo, "diff", "--name-only", "-z").decode().split("\0") if p
    ]
    untracked = [
        p
        for p in git(repo, "ls-files", "-o", "--exclude-standard", "-z").decode().split("\0")
        if p
    ]
    present = [p for p in changed + untracked if os.path.isfile(os.path.join(repo, p))]
    for p in changed:
        if p not in present:
            files.pop(p, None)  # deleted in the worktree
    if present:
        blobs = git(
            repo, "hash-object", "--no-filters", "--stdin-paths", stdin="\n".join(present).encode()
        ).decode().split()
        files.update(zip(present, blobs))
    return files


def cmd_snapshot(args):
    repo = repo_root(args.vendor)
    snap = {"repo": repo, "taken": time.time(), "files": source_map(repo)}
    os.makedirs(os.path.dirname(args.out), exist_ok=True)
    with open(args.out, "w") as fh:
        json.dump(snap, fh)
    return 0


# --- fingerprint graph ----------------------------------------------------


def hash_files(unit_dir):
    """The fingerprint hash files of one unit dir: {kind-name: hex}."""
    out = {}
    try:
        names = os.listdir(unit_dir)
    except OSError:
        return out
    for name in names:
        if name.endswith(".json") or name.startswith(("dep-", "output-")) or name == "invoked.timestamp":
            continue
        try:
            with open(os.path.join(unit_dir, name)) as fh:
                value = fh.read().strip()
        except (OSError, UnicodeDecodeError):
            continue
        if len(value) == 16:
            out[name] = value
    return out


def unit_sig(unit_dir):
    """{kind-name: "hex@mtime_ns"} of a unit's hash files.

    The hash alone does not change when a unit is recompiled from edited
    sources (cargo fingerprints path units by dep-info and mtime, not by
    content), but cargo rewrites the hash file on every compile, so its mtime
    tells a rebuilt unit apart.
    """
    sig = {}
    for kind, value in hash_files(unit_dir).items():
        try:
            sig[kind] = f"{value}@{os.stat(os.path.join(unit_dir, kind)).st_mtime_ns}"
        except OSError:
            pass
    return sig


def to_u64(hex_le):
    return int.from_bytes(bytes.fromhex(hex_le), "little")


def find_bin_unit(debug):
    """The fingerprint unit that produced debug/<BIN> (macOS uplifts by copy)."""
    try:
        uplifted = os.stat(os.path.join(debug, BIN))
    except OSError:
        return None
    want = (uplifted.st_size, int(uplifted.st_mtime))
    stem = BIN.replace("-", "_")
    best = None
    fp = os.path.join(debug, ".fingerprint")
    for unit in os.listdir(fp):
        m = UNIT_RE.match(unit)
        if not m or m.group(1) != PACKAGE:
            continue
        if f"bin-{BIN}" not in hash_files(os.path.join(fp, unit)):
            continue
        try:
            st = os.stat(os.path.join(debug, "deps", f"{stem}-{m.group(2)}"))
        except OSError:
            continue
        if (st.st_size, int(st.st_mtime)) == want and (best is None or st.st_mtime > best[0]):
            best = (st.st_mtime, unit)
    return best[1] if best else None


def walk_units(debug, bin_unit):
    """All units reachable from the bin unit: {unit: unit_sig}, missing count."""
    fp = os.path.join(debug, ".fingerprint")
    by_hash = {}
    for unit in os.listdir(fp):
        for kind, value in hash_files(os.path.join(fp, unit)).items():
            by_hash.setdefault(to_u64(value), (unit, kind))
    units = {}
    missing = 0
    stack = [(bin_unit, f"bin-{BIN}")]
    seen = set()
    while stack:
        unit, kind = stack.pop()
        if (unit, kind) in seen:
            continue
        seen.add((unit, kind))
        path = os.path.join(fp, unit)
        units.setdefault(unit, {})[kind] = unit_sig(path).get(kind)
        try:
            with open(os.path.join(path, kind + ".json")) as fh:
                deps = json.load(fh).get("deps", [])
        except (OSError, ValueError):
            continue
        for dep in deps:
            hit = by_hash.get(dep[-1])
            if hit is None:
                missing += 1
            else:
                stack.append(hit)
    return units, missing


def linked_objects(debug):
    """Basenames of the deps/*.o files the binary's debug map points at.

    With split-debuginfo=unpacked every rebuild of a crate leaves its previous
    codegen-unit objects behind under new names (measured: +256 files per
    pager rebuild, never removed), so the objects a seed needs are the ones
    the linked binary references, not every file with the unit's hash.
    None when nm is unavailable.
    """
    try:
        out = subprocess.run(
            ["nm", "-ap", os.path.join(debug, BIN)], capture_output=True, text=True, check=True
        ).stdout
    except (OSError, subprocess.CalledProcessError):
        return None
    deps = os.path.join(os.path.realpath(debug), "deps") + os.sep
    objects = []
    for line in out.splitlines():
        if " OSO " not in line:
            continue
        path = os.path.realpath(line.split(" OSO ", 1)[1])
        if path.startswith(deps) and path.endswith(".o"):
            objects.append(os.path.basename(path))
    return objects


def cmd_record(args):
    debug = os.path.join(args.target, "debug")
    out = os.path.join(debug, "dsbdev", "manifest.json")
    try:
        with open(args.last_build) as fh:
            compiled = json.load(fh)["compiled"]
    except (OSError, ValueError, KeyError):
        compiled = None
    if compiled == [] and load_manifest(debug):
        return 0  # nothing rebuilt: the recorded units and sources still hold
    bin_unit = find_bin_unit(debug)
    if bin_unit is None:
        log(f"record skipped: no fingerprint matches {debug}/{BIN}")
        return 0
    units, missing = walk_units(debug, bin_unit)
    with open(args.snapshot) as fh:
        snap = json.load(fh)
    repo = snap["repo"]
    files = snap["files"]
    # A file edited while cargo ran may or may not be what rustc read.
    for path in list(files):
        try:
            if os.stat(os.path.join(repo, path)).st_mtime >= float(args.started):
                files[path] = None
        except OSError:
            files[path] = None
    manifest = {
        "version": MANIFEST_VERSION,
        "package": PACKAGE,
        "bin": BIN,
        "bin_unit": bin_unit,
        "units": units,
        "missing_deps": missing,
        "worktree": repo,
        "built_at": time.time(),
        "files": files,
        "objects": linked_objects(debug),
    }
    os.makedirs(os.path.dirname(out), exist_ok=True)
    tmp = out + ".tmp"
    with open(tmp, "w") as fh:
        json.dump(manifest, fh)
    os.replace(tmp, out)
    return 0


# --- seeding --------------------------------------------------------------


def worktrees(repo):
    out = []
    for line in git(repo, "worktree", "list", "--porcelain").decode().splitlines():
        if line.startswith("worktree "):
            out.append(line[len("worktree "):])
    return out


def load_manifest(debug):
    try:
        with open(os.path.join(debug, "dsbdev", "manifest.json")) as fh:
            manifest = json.load(fh)
    except (OSError, ValueError):
        return None
    if manifest.get("version") != MANIFEST_VERSION or manifest.get("package") != PACKAGE:
        return None
    return manifest


def checkout_roots(repo):
    """Every worktree root of the repository and its git common dir."""
    common = git(repo, "rev-parse", "--path-format=absolute", "--git-common-dir").decode().strip()
    return [os.path.realpath(p) for p in worktrees(repo) + [common]]


def watches_checkout(unit_dir, roots):
    """Whether a unit's build script watches an absolute path in a checkout.

    Such a path belongs to the donor's checkout (the pager's build script
    watches the git HEAD and ref behind its version string), so a clone would
    keep the donor's value and never rerun for this worktree.
    """
    try:
        names = os.listdir(unit_dir)
    except OSError:
        return False
    for name in names:
        if not name.endswith(".json"):
            continue
        try:
            with open(os.path.join(unit_dir, name)) as fh:
                local = json.load(fh).get("local", [])
        except (OSError, ValueError):
            continue
        for entry in local:
            for path in (entry.get("RerunIfChanged") or {}).get("paths", []):
                if os.path.isabs(path):
                    real = os.path.realpath(path)
                    if any(real == r or real.startswith(r + os.sep) for r in roots):
                        return True
    return False


def differing(mine, theirs):
    # A donor entry of None is a file edited mid-build: treat it as different.
    return [p for p, blob in mine.items() if theirs.get(p) != blob]


def registry_packages(vendor):
    """Names of registry (non-path) packages the pager depends on."""
    host = ""
    for line in subprocess.run(["rustc", "-vV"], cwd=vendor, capture_output=True, text=True).stdout.splitlines():
        if line.startswith("host: "):
            host = line[len("host: "):]
    argv = ["cargo", "metadata", "--format-version", "1"]
    if host:
        # Without it cargo wants every platform's packages (alsa, windows, …).
        argv += ["--filter-platform", host]
    try:
        out = subprocess.run(argv + ["--offline"], cwd=vendor, capture_output=True, check=True).stdout
    except subprocess.CalledProcessError:
        out = subprocess.run(argv, cwd=vendor, capture_output=True, check=True).stdout
    meta = json.loads(out)
    pkgs = {p["id"]: p for p in meta["packages"]}
    nodes = {n["id"]: n for n in meta["resolve"]["nodes"]}
    roots = [pid for pid, p in pkgs.items() if p["name"] == PACKAGE and p["source"] is None]
    seen = set()
    stack = list(roots)
    while stack:
        pid = stack.pop()
        if pid in seen:
            continue
        seen.add(pid)
        for dep in nodes[pid]["deps"]:
            kinds = {k.get("kind") for k in dep.get("dep_kinds", [])}
            if kinds <= {"dev"} and kinds:
                continue
            stack.append(dep["pkg"])
    return {pkgs[pid]["name"] for pid in seen if pkgs[pid]["source"] is not None}


def _clonefile():
    libc = ctypes.CDLL(None, use_errno=True)
    fn = libc.clonefile
    fn.restype = ctypes.c_int
    fn.argtypes = [ctypes.c_char_p, ctypes.c_char_p, ctypes.c_uint32]
    return fn


def clone_path(clonefile, src, dst, stamp, counts):
    """Clone a file or tree; never overwrite; stamp every file with one mtime."""
    try:
        st = os.lstat(src)
    except OSError:
        counts["vanished"] += 1
        return
    if stat.S_ISDIR(st.st_mode):
        os.makedirs(dst, exist_ok=True)
        try:
            names = os.listdir(src)
        except OSError:
            return
        for name in names:
            clone_path(clonefile, os.path.join(src, name), os.path.join(dst, name), stamp, counts)
        return
    if os.path.lexists(dst):
        counts["kept"] += 1
        return
    if stat.S_ISLNK(st.st_mode):
        os.symlink(os.readlink(src), dst)
    elif clonefile(os.fsencode(src), os.fsencode(dst), 0) != 0:
        err = ctypes.get_errno()
        if err == errno.ENOENT:
            counts["vanished"] += 1
            return
        if err == errno.EEXIST:
            counts["kept"] += 1
            return
        raise OSError(err, os.strerror(err), src)
    os.utime(dst, ns=(stamp, stamp), follow_symlinks=False)
    counts["cloned"] += 1


def clone_units(src_debug, dst_debug, units, with_bin, stamp, objects=None):
    clonefile = _clonefile()
    objects = set(objects) if objects is not None else None
    counts = {"cloned": 0, "kept": 0, "vanished": 0}
    wanted = {UNIT_RE.match(u).group(2): u for u in units if UNIT_RE.match(u)}
    for sub in (".fingerprint", "build", "deps"):
        os.makedirs(os.path.join(dst_debug, sub), exist_ok=True)
    for unit in units:
        for sub in (".fingerprint", "build"):
            src = os.path.join(src_debug, sub, unit)
            if os.path.isdir(src):
                clone_path(clonefile, src, os.path.join(dst_debug, sub, unit), stamp, counts)
    with os.scandir(os.path.join(src_debug, "deps")) as it:
        for entry in it:
            m = HASH_RE.search(entry.name)
            if not (m and m.group(1) in wanted):
                continue
            if objects is not None and entry.name.endswith(".o") and entry.name not in objects:
                continue  # an object an earlier build left behind
            clone_path(clonefile, entry.path, os.path.join(dst_debug, "deps", entry.name), stamp, counts)
    if with_bin:
        clone_path(clonefile, os.path.join(src_debug, BIN), os.path.join(dst_debug, BIN), stamp, counts)
    return counts


def cmd_seed(args):
    if sys.platform != "darwin":
        log("seed skipped: needs APFS clonefile (macOS)")
        return 0
    target = os.path.realpath(args.target)
    debug = os.path.join(target, "debug")
    if load_manifest(debug):
        return 0
    started = time.time()
    repo = repo_root(args.vendor)
    vendor_rel = os.path.relpath(os.path.realpath(args.vendor), os.path.realpath(repo))
    mine = source_map(repo)
    roots = checkout_roots(repo)

    best = None
    for wt in worktrees(repo):
        cand = os.path.realpath(os.path.join(wt, vendor_rel, "target"))
        if cand == target:
            continue
        manifest = load_manifest(os.path.join(cand, "debug"))
        if manifest is None:
            continue
        diff = differing(mine, manifest["files"])
        key = (len(diff), -manifest["built_at"])
        if best is None or key < best[0]:
            best = (key, cand, manifest, diff)

    # One mtime for every cloned file: cargo compares a unit's outputs with its
    # dependencies' outputs and its sources by mtime (strictly newer = stale),
    # so a single stamp keeps the clones mutually fresh and newer than every
    # source already on disk, and a bumped source (stamp + 10ms) is newer than
    # all of them.
    stamp = time.time_ns()
    bump = (stamp + 10_000_000) / 1e9
    if best is not None:
        _, donor, manifest, diff = best
        src_debug = os.path.join(donor, "debug")
        # Unit names become path components below; take only cargo's shape.
        recorded = {
            u: kinds
            for u, kinds in manifest["units"].items()
            if UNIT_RE.match(u) and os.sep not in u and not u.startswith(".")
        }

        def current(unit):
            live = unit_sig(os.path.join(src_debug, ".fingerprint", unit))
            return all(v is not None and live.get(k) == v for k, v in recorded[unit].items())

        # A unit rebuilt in the donor since its manifest was built from sources
        # the manifest does not describe: leave it for cargo to compile.
        units = [u for u in recorded if current(u)]
        stale = len(recorded) - len(units)
        bound = [u for u in units if watches_checkout(os.path.join(src_debug, ".fingerprint", u), roots)]
        units = [u for u in units if u not in bound]
        counts = clone_units(
            src_debug, debug, units, stale == 0 and not bound, stamp, manifest.get("objects")
        )
        # The donor may be building while we copy; drop what changed under us.
        for unit in units:
            if not current(unit):
                shutil.rmtree(os.path.join(debug, ".fingerprint", unit), ignore_errors=True)
                stale += 1
        for path in diff:
            full = os.path.join(repo, path)
            if os.path.isfile(full):
                os.utime(full, (bump, bump))
        # cargo marks a unit stale when a source is newer than the moment it
        # started compiling it, so do not start the build before the bump.
        time.sleep(max(0.0, bump - time.time() + 0.01))
        log(
            f"seeded {len(units)} units from {manifest['worktree']} "
            f"({counts['cloned']} files cloned, {stale} stale and {len(bound)} checkout-bound units skipped, "
            f"{len(diff)} files differ) in {time.time() - started:.1f}s"
        )
        return 0

    primary = worktrees(repo)[0]
    shared = os.path.join(primary, vendor_rel, "target", "debug")
    if not os.path.isdir(os.path.join(shared, ".fingerprint")) or os.path.realpath(shared) == debug:
        log("no seed source; first build compiles everything")
        return 0
    try:
        names = registry_packages(args.vendor)
    except (subprocess.CalledProcessError, OSError, ValueError, KeyError) as e:
        log(f"seed skipped: cargo metadata failed ({e})")
        return 0
    units = []
    for unit in os.listdir(os.path.join(shared, ".fingerprint")):
        m = UNIT_RE.match(unit)
        if m and m.group(1) in names and not watches_checkout(
            os.path.join(shared, ".fingerprint", unit), roots
        ):
            units.append(unit)
    counts = clone_units(shared, debug, units, False, stamp)
    log(
        f"seeded {len(units)} registry units from the shared target "
        f"({counts['cloned']} files cloned) in {time.time() - started:.1f}s; "
        "workspace crates build once"
    )
    return 0


# --- build summary --------------------------------------------------------


def package_name(package_id):
    # "registry+https://…#serde@1.0.0", "path+file:///…/xai-grok-version#0.1.0"
    frag = package_id.rsplit("#", 1)[-1]
    if "@" in frag:
        return frag.split("@", 1)[0]
    return package_id.split("#", 1)[0].rstrip("/").rsplit("/", 1)[-1]


def cmd_summarize(args):
    compiled = []
    fresh = 0
    ok = False
    for line in sys.stdin:
        try:
            msg = json.loads(line)
        except ValueError:
            sys.stderr.write(line)
            continue
        reason = msg.get("reason")
        if reason == "compiler-artifact":
            if msg.get("fresh"):
                fresh += 1
            else:
                name = package_name(msg.get("package_id", ""))
                if "custom-build" in msg.get("target", {}).get("kind", []):
                    name += " (build script)"
                compiled.append(name)
        elif reason == "build-finished":
            ok = bool(msg.get("success"))
    result = {
        "compiled": compiled,
        "fresh": fresh,
        "success": ok,
        "seconds": round(time.time() - float(args.started), 1),
    }
    os.makedirs(os.path.dirname(args.out), exist_ok=True)
    with open(args.out, "w") as fh:
        json.dump(result, fh)
    shown = ", ".join(compiled[:6]) + (" …" if len(compiled) > 6 else "")
    log(
        f"{len(compiled)} compiled, {fresh} fresh in {result['seconds']}s"
        + (f" — {shown}" if compiled else "")
    )
    return 0


def main(argv):
    parser = argparse.ArgumentParser(prog="dsbdev.py")
    sub = parser.add_subparsers(dest="cmd", required=True)
    p = sub.add_parser("snapshot")
    p.add_argument("--vendor", required=True)
    p.add_argument("--out", required=True)
    p = sub.add_parser("seed")
    p.add_argument("--vendor", required=True)
    p.add_argument("--target", required=True)
    p = sub.add_parser("summarize")
    p.add_argument("--out", required=True)
    p.add_argument("--started", required=True)
    p = sub.add_parser("record")
    p.add_argument("--vendor", required=True)
    p.add_argument("--target", required=True)
    p.add_argument("--snapshot", required=True)
    p.add_argument("--started", required=True)
    p.add_argument("--last-build", required=True)
    args = parser.parse_args(argv)
    return {
        "snapshot": cmd_snapshot,
        "seed": cmd_seed,
        "summarize": cmd_summarize,
        "record": cmd_record,
    }[args.cmd](args)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
