use super::conversions::hashes_line_up;
use super::*;
use crate::test_support::{add_worktree, publish, seed_source};
use std::path::PathBuf;
use xai_test_utils::git::{run_git, run_git_with_env};

#[path = "safety_tests/conversions.rs"]
mod conversions;
#[path = "safety_tests/gate.rs"]
mod gate;
#[path = "safety_tests/git_dir.rs"]
mod git_dir;
#[path = "safety_tests/reachability.rs"]
mod reachability;
fn reclaim(worktree: &Path) -> Safety {
    reclaim_beside(worktree, None)
}

fn reclaim_beside(worktree: &Path, surviving: Option<&Path>) -> Safety {
    let safety = safe_to_delete_worktree(worktree, surviving);
    if safety == Safety::Delete {
        crate::remove_worktree(worktree).expect("removal");
    }
    safety
}

fn reclaim_after_snapshot(worktree: &Path, source: &Path, ref_name: &str) -> Safety {
    snapshot_into(worktree, source, ref_name);
    let safety = safe_to_delete_worktree_after_snapshot(worktree, Some(source), ref_name);
    if safety == Safety::Delete {
        crate::remove_worktree(worktree).expect("removal");
    }
    safety
}

#[path = "safety_tests/working_tree.rs"]
mod working_tree;

struct Fixture {
    _root: tempfile::TempDir,
    source: PathBuf,
    remote: PathBuf,
}

impl Fixture {
    fn new(ignore_lines: &str) -> Self {
        let root = tempfile::tempdir().unwrap();
        let source = root.path().join("source");
        seed_source(&source, ignore_lines);
        Self {
            remote: root.path().join("remote.git"),
            source,
            _root: root,
        }
    }

    fn linked_worktree(&self, name: &str) -> PathBuf {
        add_worktree(&self.source, &self.source.with_file_name(name))
    }

    fn snapshot_worktree(&self, name: &str) -> PathBuf {
        let at = self.source.with_file_name(name);
        copy_tree(&self.source, &at);
        at
    }

    fn add_source_clutter(&self) {
        std::fs::write(self.source.join("tracked.txt"), "tagged\n").unwrap();
        run_git(
            &self.source,
            &["commit", "-am", "a commit only a tag holds"],
        );
        let tagged = run_git(&self.source, &["rev-parse", "HEAD"]);
        run_git(&self.source, &["tag", "backup-2026-08-07"]);
        run_git(&self.source, &["reset", "--hard", "HEAD~1"]);
        for name in ["refs/prefetch/origin/main", "refs/backup/nightly"] {
            run_git(&self.source, &["update-ref", name, &tagged]);
        }
        run_git(&self.source, &["update-ref", "refs/stash", &tagged]);
        seed_module_store(
            self.source.parent().unwrap(),
            &self.source.join(".git/modules/vendor/example-lib"),
        );
    }

    fn standalone_worktree(&self, name: &str) -> PathBuf {
        let at = self.source.with_file_name(name);
        run_git(
            self.source.parent().unwrap(),
            &[
                "clone",
                "--branch",
                "main",
                self.remote.to_str().unwrap(),
                at.to_str().unwrap(),
            ],
        );
        at
    }
}

fn seed_module_store(root: &Path, at: &Path) {
    let scratch = root.join("module-scratch");
    std::fs::create_dir_all(&scratch).unwrap();
    xai_test_utils::git::git_init_seed(&scratch);
    std::fs::write(scratch.join("file.txt"), "sub\n").unwrap();
    run_git(&scratch, &["add", "."]);
    run_git(&scratch, &["commit", "-m", "submodule seed"]);
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();
    copy_tree(&scratch.join(".git"), at);
    std::fs::remove_dir_all(&scratch).unwrap();
}

fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let (from, to) = (entry.path(), to.join(entry.file_name()));
        // An entry can be gone by the time the walk reaches it: the source is a
        // live repository, and git spawns automatic maintenance detached by
        // default (`maintenance.autoDetach`), so `commit`, `push` and `fetch`
        // leave behind a process that deletes `.git/objects/maintenance.lock`
        // as its run ends. A `file_type` that fails for a gone entry falls
        // through to `copy_listed_file`, which skips it the same way.
        if entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false) {
            copy_tree(&from, &to);
        } else {
            copy_listed_file(&from, &to);
        }
    }
}

/// Copy one file a walk has already listed.
///
/// The source is a live repository (see `copy_tree`), so the walk can list a
/// file the maintenance process removes before the copy reaches it. The copy
/// then fails with `NotFound` — `ci-grok-test` run 36225954017 failed exactly
/// so, on `.git/objects/maintenance.lock`. A file that is gone by then is
/// skipped; every other error still fails.
fn copy_listed_file(from: &Path, to: &Path) {
    if let Err(error) = std::fs::copy(from, to)
        && error.kind() != std::io::ErrorKind::NotFound
    {
        panic!("copy {} to {}: {error}", from.display(), to.display());
    }
}

fn attributes(worktree: &Path, rules: impl AsRef<[u8]>, branch: &str) {
    std::fs::write(worktree.join(".gitattributes"), rules).unwrap();
    run_git(worktree, &["add", ".gitattributes"]);
    run_git(worktree, &["commit", "-m", "route the paths"]);
    publish(worktree, branch);
}

fn snapshot_into(worktree: &Path, source: &Path, ref_name: &str) {
    crate::snapshot_worktree_to_ref(worktree, ref_name, "snapshot").unwrap();
    crate::transfer_snapshot_to_repo(worktree, source, ref_name).unwrap();
}

/// A copy that walks a live repository lists a file the source can drop before
/// the copy reaches it — the maintenance process behind
/// `.git/objects/maintenance.lock` is exactly that. Reaching here is the
/// assertion: a panic would fail whichever test was running.
#[test]
fn a_file_the_source_dropped_after_the_listing_is_skipped() {
    let root = tempfile::tempdir().unwrap();
    let listed = root.path().join("maintenance.lock");
    std::fs::write(&listed, "").unwrap();
    let to = root.path().join("snapshot").join("maintenance.lock");
    std::fs::create_dir_all(to.parent().unwrap()).unwrap();

    std::fs::remove_file(&listed).unwrap(); // the maintenance run ended

    copy_listed_file(&listed, &to);

    assert!(
        !to.exists(),
        "a file the source dropped leaves nothing behind"
    );
}

/// The skip is for entries the walk listed, not for a missing tree: an empty
/// snapshot would leave every assertion about it meaningless.
#[test]
#[should_panic]
fn a_copy_of_a_tree_that_is_not_there_still_fails() {
    let root = tempfile::tempdir().unwrap();

    copy_tree(&root.path().join("gone"), &root.path().join("snapshot"));
}
