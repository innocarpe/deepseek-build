#!/usr/bin/env python3
"""Run a cargo command against this checkout's own vendored Grok target.

One worktree, one target
------------------------
cargo names a path package's artifacts with a hash of its path *relative to
the workspace root*, so two worktrees of this repo that share one
CARGO_TARGET_DIR write the same file names, and cargo's mtime-only freshness
then hands the worktree with the older sources the other worktree's code
(measured 2026-09-27 on crates/common/xai-message-delivery-core: worktree B's
`cargo test` finished in 0.35s and ran worktree A's binary). This wrapper
exists so that cannot happen:

* it exports CARGO_TARGET_DIR=<worktree>/third_party/grok-build/target,
  overriding whatever the caller's environment carried, and says so when the
  inherited value differed;
* a cold target is seeded once from the sibling worktree whose recorded
  sources differ least (`scripts/lib/dsbdev.py seed`), so the first command in
  a new worktree compiles the crates whose sources differ, not the workspace;
* the job count is capped (4 by default — memory is the scarce resource);
* it refuses to start while another worktree's vendored build is in flight;
  `--allow-concurrent` lets it through the memory gate as a second build,
  capped to 2 jobs, exactly like `vendor-build.sh run`.

Everything after the wrapper's own flags goes to cargo unchanged:

  ./scripts/vendor-cargo.sh test -p xai-grok-pager --lib
  ./scripts/vendor-cargo.sh check -p xai-grok-shell --all-targets
  ./scripts/vendor-cargo.sh test -p xai-grok-pager --lib -- --nocapture
  ./scripts/vendor-cargo.sh --jobs 6 test -p xai-grok-pager --lib

Subcommands that compile nothing (`fmt`, `metadata`, `tree`, …) run without a
seed and without the queue check. `--release` and `--profile` skip the seed:
its units come from a debug-profile build and do not transfer.

The command runs from the vendored tree, in this worktree's own target. The
top-level workspace (crates/) needs none of this: cargo already defaults its
target to this worktree.
"""
import argparse
import os
import shutil
import subprocess
import sys
from pathlib import Path

import vendor_build  # same directory: the queue reader and the memory gate

DEFAULT_JOBS = 4
SECOND_JOBS = vendor_build.SECOND_BUILD_JOBS
NO_BUILD = {
    "fmt", "metadata", "tree", "pkgid", "locate-project", "verify-project",
    "search", "add", "remove", "generate-lockfile", "update", "vendor", "help",
    "version", "login", "logout", "owner", "yank", "publish", "package",
    "info", "fetch", "init", "new",
}
GLOBAL_FLAGS = {
    "-v", "--verbose", "-q", "--quiet", "--version", "-V", "--help", "-h",
    "--locked", "--offline", "--frozen", "--future-incompat-report", "--list",
}


def log(message):
    print(f"vendor-cargo: {message}", file=sys.stderr)


def fail(message, code=2):
    print(f"vendor-cargo: error: {message}", file=sys.stderr)
    sys.exit(code)


def usage():
    print(__doc__.strip())
    print()
    print("Flags (leading; everything after the first non-flag token goes to cargo):")
    print("  --jobs N             job count for cargo (default: CARGO_BUILD_JOBS, else 4)")
    print("  --no-seed            never seed a cold target")
    print("  --allow-concurrent   start even while another worktree's vendored build is")
    print("                       in flight, when the memory gate passes (then 2 jobs)")
    print("  --vendor DIR         vendored tree (default: <worktree>/third_party/grok-build)")
    print("  --target-dir DIR     cargo target directory (default: <vendor>/target)")
    print("  --repo DIR           repository whose worktrees the queue check reads")
    print("  -h, --help           this text")


def parse(argv):
    """Split wrapper flags from the cargo argv.

    Wrapper flags are leading: parsing stops at `--` or at the first token
    that is not one of them, so `--target <triple>` and `-j` stay cargo's.
    """
    parser = argparse.ArgumentParser(prog="vendor-cargo.sh", add_help=False)
    parser.add_argument("--jobs", type=int, default=None)
    parser.add_argument("--no-seed", action="store_true", dest="no_seed")
    parser.add_argument("--allow-concurrent", action="store_true", dest="allow_concurrent")
    parser.add_argument("--vendor", default=None)
    parser.add_argument("--target-dir", default=None, dest="target_dir")
    parser.add_argument("--repo", default=None)
    parser.add_argument("-h", "--help", action="store_true", dest="help")
    known, rest = [], []
    i = 0
    while i < len(argv):
        a = argv[i]
        if a == "--":
            i += 1
            break
        if a.startswith("--") and "=" in a:
            head = a.split("=", 1)[0]
        else:
            head = a
        if head not in ("--jobs", "--no-seed", "--allow-concurrent", "--vendor",
                        "--target-dir", "--repo", "-h", "--help"):
            break
        known.append(a)
        if head in ("--jobs", "--vendor", "--target-dir", "--repo"):
            if "=" not in a:
                i += 1
                if i >= len(argv):
                    fail(f"{head} needs a value")
                known.append(argv[i])
        i += 1
    args = parser.parse_args(known)
    return args, argv[i:]


def git(root, *argv):
    r = subprocess.run(
        ["git", "-C", str(root), "rev-parse", *argv], capture_output=True, text=True
    )
    return r.stdout.strip() if r.returncode == 0 and r.stdout.strip() else None


def resolve_worktree(args):
    if args.repo:
        return Path(os.path.expanduser(args.repo)).resolve()
    top = git(os.getcwd(), "--show-toplevel")
    if top is None:
        fail("not inside a git worktree; run this from the checkout, or pass --vendor")
    return Path(top).resolve()


def resolve_vendor(args, worktree):
    if args.vendor:
        return Path(os.path.expanduser(args.vendor)).resolve()
    vendor = worktree / "third_party" / "grok-build"
    if not (vendor / "SOURCE_REV").is_file():
        fail(f"no vendored tree at {vendor}; run this from a checkout of this repo, or pass --vendor")
    return vendor


def cargo_subcommand(argv):
    for token in argv:
        if token.startswith("+"):  # +toolchain
            continue
        if token in GLOBAL_FLAGS:
            continue
        if token.startswith("-"):
            continue
        return token
    return None


def profile_override(argv):
    """True when cargo runs a non-dev profile (its units are not seedable)."""
    if "--release" in argv or "-r" in argv:
        return True
    return "--profile" in argv or any(a.startswith("--profile=") for a in argv)


def target_is_cold(target):
    return not (target / "debug" / ".fingerprint").is_dir()


def seed(vendor, target, worktree):
    helper = Path(__file__).resolve().parent / "dsbdev.py"
    log("cold target — seeding it once from the best sibling donor")
    r = subprocess.run(
        [sys.executable, str(helper), "seed", "--vendor", str(vendor), "--target", str(target)],
        cwd=str(worktree),
    )
    if r.returncode != 0:
        log("seeding failed; building without it")


def queue(primary):
    wt_roots = vendor_build.worktree_roots(primary)
    entries, busy = vendor_build.collect_machine(primary, wt_roots)
    facts = vendor_build.host_facts()
    return entries, busy, facts, vendor_build.evaluate_gate(facts)


def main(argv):
    args, cargo_argv = parse(argv)
    if args.help:
        usage()
        return 0
    if not cargo_argv:
        fail("nothing to run; pass a cargo command, e.g. `vendor-cargo.sh test -p <pkg> --lib`", code=2)

    worktree = resolve_worktree(args)
    vendor = resolve_vendor(args, worktree)
    target = (
        Path(os.path.expanduser(args.target_dir)).resolve()
        if args.target_dir
        else (vendor / "target")
    )
    primary = vendor_build.primary_checkout(worktree)

    subcommand = cargo_subcommand(cargo_argv)
    builds = subcommand not in NO_BUILD

    second = False
    if builds:
        entries, busy, facts, gate = queue(primary)
        if busy and not args.allow_concurrent:
            print(vendor_build.render_machine_status(primary, entries, busy, facts, gate))
            log(
                "a vendored build is in flight and one runs at a time — wait for it, or pass "
                "--allow-concurrent to go through the memory gate as a second build"
            )
            return 1
        if busy:
            if not gate["allowed"]:
                print(vendor_build.render_machine_status(primary, entries, busy, facts, gate))
                log("--allow-concurrent refused: the memory gate denies a second build")
                return 1
            second = True
            log(f"a build is in flight; memory gate passed — running as a second build at -j {SECOND_JOBS}")

    if builds and not args.no_seed and not profile_override(cargo_argv) and target_is_cold(target):
        seed(vendor, target, worktree)

    env = os.environ.copy()
    inherited = env.get("CARGO_TARGET_DIR")
    if inherited:
        try:
            same = Path(inherited).resolve() == target
        except OSError:
            same = False
        if not same:
            log(f"ignoring inherited CARGO_TARGET_DIR={inherited} — this worktree builds in {target}")
    env["CARGO_TARGET_DIR"] = str(target)

    jobs = args.jobs
    if jobs is None:
        try:
            jobs = int(env.get("CARGO_BUILD_JOBS") or DEFAULT_JOBS)
        except ValueError:
            jobs = DEFAULT_JOBS
    if second:
        jobs = min(jobs, SECOND_JOBS)
    env["CARGO_BUILD_JOBS"] = str(jobs)

    if not shutil.which("cargo", path=env.get("PATH")):
        # The dsb tool shell can lack cargo (measured 2026-09-26); a PATH that
        # has one is left alone, so a stub on PATH keeps working.
        env["PATH"] = os.pathsep.join(
            [str(Path.home() / ".cargo" / "bin"), "/opt/homebrew/bin", "/usr/local/bin",
             env.get("PATH", "")]
        )

    log(f"{subcommand or 'cargo'} → target {target} · -j {jobs}")
    sys.stdout.flush()
    sys.stderr.flush()
    os.chdir(vendor)
    os.execvpe("cargo", ["cargo", *cargo_argv], env)  # never returns


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
