'use strict';

/**
 * Regression tests for the prebuilt installer (ADR 0009).
 *
 * Two defects are pinned here, both measured on `5.6.0`:
 *
 * 1. **Self-check env.** `DEEPSEEK_BUILD_VERSION` set in the caller's shell
 *    (e.g. a shell that ran `scripts/build-grok-pager.sh`) leaks into the
 *    agent's `--version`, which prints the variable's value instead of the
 *    version baked into the binary. The check then calls a *good* install
 *    corrupt and `npm i -g` fails.
 * 2. **Destructive failure.** After that verdict the installer deleted the
 *    binaries — including a previous, working installation — leaving the user
 *    with no `deepseek-build-agent` and no backup in the package.
 *
 * The tests use stub binaries (`sleep` + `echo`) instead of the real agent: a
 * stub is what makes both properties observable, and a real 167 MB binary
 * cannot be faked into reporting a wrong version. Nothing here touches the
 * user's `~/.deepseek-build` — every path comes from a temp dir.
 */

const assert = require('node:assert/strict');
const fs = require('fs');
const os = require('os');
const path = require('path');
const test = require('node:test');

const {
  installFromStageDir,
  verifyInstalledBin,
  selfCheckEnv,
  VERSION_STAMP_ENV,
  REQUIRED,
} = require('../scripts/install-prebuilt');
const { isSourceCheckout, main: runPostinstall } = require('../scripts/postinstall');

const VERSION = '9.9.9';
/** Deliberately different from VERSION: if it leaks through, the output differs. */
const POLLUTED = '1.2.3';

/**
 * A stub `--version` binary.
 *
 * `stampAware`: prints the env stamp when set — the vendored `installed()`
 * contract (`DEEPSEEK_BUILD_VERSION` → `GROK_TEST_VERSION` → built value) that
 * makes the bug reachable.
 * `stampAware: false`: always prints its built version, like `/bin/echo` — used
 * to prove the check still fails a genuinely wrong binary.
 */
function writeStub(dir, name, version, { stampAware = true, exitCode = 0 } = {}) {
  const file = path.join(dir, name);
  const body = stampAware
    ? [
        '#!/bin/sh',
        'v="${DEEPSEEK_BUILD_VERSION:-${GROK_TEST_VERSION:-' + version + '}}"',
        `printf 'deepseek-build %s (stub000) [stable]\\n' "$v"`,
        `exit ${exitCode}`,
        '',
      ].join('\n')
    : ['#!/bin/sh', `printf 'deepseek-build ${version} (stub000) [stable]\\n'`, `exit ${exitCode}`, ''].join('\n');
  fs.writeFileSync(file, body);
  fs.chmodSync(file, 0o755);
  return file;
}

/** A complete staging dir (all `REQUIRED` binaries report `VERSION`). */
function makeStage(dir, version, opts) {
  fs.mkdirSync(dir, { recursive: true });
  for (const name of REQUIRED) {
    writeStub(dir, name, version, opts);
  }
  return dir;
}

/** An existing installation in `binDir`, with a distinguishing marker. */
function writeExistingInstall(binDir, version) {
  fs.mkdirSync(binDir, { recursive: true });
  for (const name of REQUIRED) {
    writeStub(binDir, name, version);
    // Marker appended after the shebang; a stub prints nothing extra, so the
    // marker survives only if the installer never replaced the file.
    fs.appendFileSync(path.join(binDir, name), `# existing-install ${version}\n`);
  }
}

function sha(file) {
  return require('crypto').createHash('sha256').update(fs.readFileSync(file)).digest('hex');
}

function withTempDir(t) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'dsb-install-test-'));
  t.after(() => fs.rmSync(dir, { recursive: true, force: true }));
  return dir;
}

/** Run `fn` with env vars set, restoring the previous values afterwards. */
function withEnv(vars, fn) {
  const saved = {};
  for (const [k, v] of Object.entries(vars)) {
    saved[k] = Object.prototype.hasOwnProperty.call(process.env, k) ? process.env[k] : undefined;
    if (v === undefined) delete process.env[k];
    else process.env[k] = v;
  }
  try {
    return fn();
  } finally {
    for (const [k, v] of Object.entries(saved)) {
      if (v === undefined) delete process.env[k];
      else process.env[k] = v;
    }
  }
}

// ---------------------------------------------------------------------------
// 1. Self-check env
// ---------------------------------------------------------------------------

test('selfCheckEnv drops the runtime version stamps and keeps everything else', () => {
  const env = selfCheckEnv({
    PATH: '/usr/bin',
    HOME: '/home/example',
    DEEPSEEK_BUILD_VERSION: POLLUTED,
    GROK_TEST_VERSION: POLLUTED,
  });
  for (const key of VERSION_STAMP_ENV) {
    assert.equal(Object.prototype.hasOwnProperty.call(env, key), false, `${key} must be absent`);
  }
  assert.equal(env.PATH, '/usr/bin', 'PATH is what lets the binary exec at all');
  assert.equal(env.HOME, '/home/example');
});

test('the stamp list covers both variables the agent honours at runtime', () => {
  // `installed()` in xai-grok-version reads exactly these two before falling
  // back to the baked version; the release workflows strip the same pair.
  assert.deepEqual(VERSION_STAMP_ENV, ['DEEPSEEK_BUILD_VERSION', 'GROK_TEST_VERSION']);
});

test('verifyInstalledBin tolerates a polluted env (the install that used to fail)', (t) => {
  const dir = withTempDir(t);
  const bin = writeStub(dir, 'deepseek-build-agent', VERSION);
  withEnv({ DEEPSEEK_BUILD_VERSION: POLLUTED, GROK_TEST_VERSION: undefined }, () => {
    const problem = verifyInstalledBin(bin, 'deepseek-build-agent', VERSION);
    assert.equal(problem, null, `expected a pass, got: ${problem}`);
  });
});

test('verifyInstalledBin still fails a binary that reports the wrong version', (t) => {
  const dir = withTempDir(t);
  // Stamp-ignoring stub, like the release artifact: it cannot pass by accident.
  const bin = writeStub(dir, 'deepseek-build-agent', '0.0.1', { stampAware: false });
  const problem = verifyInstalledBin(bin, 'deepseek-build-agent', VERSION);
  assert.ok(problem, 'a wrong version must still be reported');
  assert.match(problem, /0\.0\.1/);
});

test('verifyInstalledBin still fails a binary that cannot exec (exit != 0)', (t) => {
  const dir = withTempDir(t);
  const bin = writeStub(dir, 'deepseek-build-agent', VERSION, { exitCode: 3 });
  const problem = verifyInstalledBin(bin, 'deepseek-build-agent', VERSION);
  assert.ok(problem, 'a non-zero exit must still be reported');
  assert.match(problem, /status 3/);
});

// ---------------------------------------------------------------------------
// 2. Destructive failure
// ---------------------------------------------------------------------------

test('polluted env: install succeeds and leaves a working binDir', (t) => {
  const root = withTempDir(t);
  const stage = makeStage(path.join(root, 'stage'), VERSION);
  const binDir = path.join(root, 'home', 'bin');

  withEnv({ DEEPSEEK_BUILD_VERSION: POLLUTED, GROK_TEST_VERSION: undefined }, () => {
    const result = installFromStageDir({ stageDir: stage, version: VERSION, binDir });
    assert.equal(result.ok, true, `expected ok, got: ${result.error}`);
  });

  for (const name of REQUIRED) {
    const dest = path.join(binDir, name);
    assert.ok(fs.existsSync(dest), `${name} must be installed`);
    assert.equal(verifyInstalledBin(dest, name, VERSION), null, `${name} must report ${VERSION}`);
  }
  assert.deepEqual(
    fs.readdirSync(binDir).filter((n) => n.startsWith('.install-')),
    [],
    'no staging dir may be left behind'
  );
});

test('a failed self-check does not remove an existing installation', (t) => {
  const root = withTempDir(t);
  const binDir = path.join(root, 'home', 'bin');
  writeExistingInstall(binDir, '5.5.4');
  const before = Object.fromEntries(REQUIRED.map((n) => [n, sha(path.join(binDir, n))]));

  // Staged build reports the wrong version (a truncated download, say).
  const stage = makeStage(path.join(root, 'stage'), '0.0.1', { stampAware: false });

  const result = installFromStageDir({ stageDir: stage, version: VERSION, binDir });
  assert.equal(result.ok, false, 'the install must fail closed');
  assert.match(result.error, /failed self-check/);
  // npm 12 still blocks `npm rebuild -g <pkg>` (measured). The retry has to
  // carry --allow-scripts and repeat the package spec.
  assert.match(
    result.error,
    /Retry: npm rebuild -g --allow-scripts=@innocarpe\/deepseek-build @innocarpe\/deepseek-build/
  );

  for (const name of REQUIRED) {
    const dest = path.join(binDir, name);
    assert.ok(fs.existsSync(dest), `${name} must survive a failed install (was deleted)`);
    assert.equal(sha(dest), before[name], `${name} must be byte-identical to the previous install`);
  }
  assert.deepEqual(
    fs.readdirSync(binDir).filter((n) => n.startsWith('.install-')),
    [],
    'no staging dir may be left behind'
  );
});

test('a failed self-check does not leave a fresh install behind either', (t) => {
  const root = withTempDir(t);
  const binDir = path.join(root, 'home', 'bin');

  const stage = makeStage(path.join(root, 'stage'), VERSION);
  // Second binary is corrupt: the first must not be published.
  fs.rmSync(path.join(stage, 'dsb'));
  writeStub(stage, 'dsb', '0.0.1', { stampAware: false });

  const result = installFromStageDir({ stageDir: stage, version: VERSION, binDir });
  assert.equal(result.ok, false);
  assert.equal(fs.existsSync(path.join(binDir, 'deepseek-build')), false,
    'a verified sibling must not be published while another fails the check');
  assert.equal(fs.existsSync(path.join(binDir, 'deepseek-build-agent')), false);
});

test('a good install replaces a previous one and reports the tarball version', (t) => {
  const root = withTempDir(t);
  const binDir = path.join(root, 'home', 'bin');
  writeExistingInstall(binDir, '5.5.4');

  const stage = makeStage(path.join(root, 'stage'), VERSION);
  const result = installFromStageDir({ stageDir: stage, version: VERSION, binDir });
  assert.equal(result.ok, true, `expected ok, got: ${result.error}`);

  for (const name of REQUIRED) {
    const dest = path.join(binDir, name);
    assert.equal(verifyInstalledBin(dest, name, VERSION), null, `${name} must report ${VERSION}`);
    assert.doesNotMatch(fs.readFileSync(dest, 'utf8'), /existing-install/,
      `${name} must be the new file, not the old one`);
  }
});

test('a failed install leaves a previous installation runnable, not just present', (t) => {
  const root = withTempDir(t);
  const binDir = path.join(root, 'home', 'bin');
  writeExistingInstall(binDir, '5.5.4');

  const stage = makeStage(path.join(root, 'stage'), '0.0.1', { stampAware: false });
  const result = installFromStageDir({ stageDir: stage, version: VERSION, binDir });
  assert.equal(result.ok, false);

  // The user's next `dsb` must still work. `verifyInstalledBin` is exactly the
  // exec a user-vs-install mismatch shows up in.
  const problem = verifyInstalledBin(path.join(binDir, 'deepseek-build-agent'), 'deepseek-build-agent', '5.5.4');
  assert.equal(problem, null, `the previous install must still run: ${problem}`);
});

test('a newer agent is not replaced by an older package', (t) => {
  const root = withTempDir(t);
  const binDir = path.join(root, 'home', 'bin');
  writeExistingInstall(binDir, '6.0.0');
  const before = Object.fromEntries(REQUIRED.map((n) => [n, sha(path.join(binDir, n))]));

  const stage = makeStage(path.join(root, 'stage'), '5.7.0');
  const result = withEnv({ DEEPSEEK_BUILD_ALLOW_DOWNGRADE: undefined }, () =>
    installFromStageDir({ stageDir: stage, version: '5.7.0', binDir })
  );
  assert.equal(result.ok, false, 'an older package must not downgrade the agent');
  assert.match(result.error, /refusing to replace deepseek-build-agent 6\.0\.0 with package 5\.7\.0/);

  for (const name of REQUIRED) {
    const dest = path.join(binDir, name);
    assert.equal(sha(dest), before[name], `${name} must stay the newer install`);
  }
  assert.equal(
    verifyInstalledBin(path.join(binDir, 'deepseek-build-agent'), 'deepseek-build-agent', '6.0.0'),
    null
  );
});

test('DEEPSEEK_BUILD_ALLOW_DOWNGRADE=1 installs the older package', (t) => {
  const root = withTempDir(t);
  const binDir = path.join(root, 'home', 'bin');
  writeExistingInstall(binDir, '6.0.0');
  const stage = makeStage(path.join(root, 'stage'), '5.7.0');

  const result = withEnv({ DEEPSEEK_BUILD_ALLOW_DOWNGRADE: '1' }, () =>
    installFromStageDir({ stageDir: stage, version: '5.7.0', binDir })
  );
  assert.equal(result.ok, true, `expected ok, got: ${result.error}`);
  assert.equal(
    verifyInstalledBin(path.join(binDir, 'deepseek-build-agent'), 'deepseek-build-agent', '5.7.0'),
    null
  );
});

test('an agent that will not run does not block a reinstall', (t) => {
  const root = withTempDir(t);
  const binDir = path.join(root, 'home', 'bin');
  writeExistingInstall(binDir, '6.0.0');
  writeStub(binDir, 'deepseek-build-agent', '6.0.0', { exitCode: 3 });

  const stage = makeStage(path.join(root, 'stage'), '5.7.0');
  const result = installFromStageDir({ stageDir: stage, version: '5.7.0', binDir });
  assert.equal(result.ok, true, `a dead agent is not a version to protect: ${result.error}`);
  assert.equal(
    verifyInstalledBin(path.join(binDir, 'deepseek-build-agent'), 'deepseek-build-agent', '5.7.0'),
    null
  );
});

test('the mirror copy (npm/native-bin) is only written on success', (t) => {
  const root = withTempDir(t);
  const binDir = path.join(root, 'home', 'bin');
  const mirror = path.join(root, 'pkg', 'native-bin');
  writeExistingInstall(binDir, '5.5.4');

  const badStage = makeStage(path.join(root, 'bad'), '0.0.1', { stampAware: false });
  const failed = installFromStageDir({
    stageDir: badStage, version: VERSION, binDir, pkgNativeBin: mirror,
  });
  assert.equal(failed.ok, false);
  assert.equal(fs.existsSync(mirror), false, 'a failed install must not populate the mirror');

  const goodStage = makeStage(path.join(root, 'good'), VERSION);
  const ok = installFromStageDir({
    stageDir: goodStage, version: VERSION, binDir, pkgNativeBin: mirror,
  });
  assert.equal(ok.ok, true, `expected ok, got: ${ok.error}`);
  for (const name of REQUIRED) {
    assert.ok(fs.existsSync(path.join(mirror, name)), `${name} must be mirrored on success`);
  }
});

// ---------------------------------------------------------------------------
// 3. Source checkout must not install; a packed tree still must
// ---------------------------------------------------------------------------

/** Cargo workspace + source installer. `git` selects the metadata shape. */
function writeCheckoutMarkers(root, git) {
  fs.writeFileSync(path.join(root, 'Cargo.toml'), '[workspace]\n');
  fs.mkdirSync(path.join(root, 'scripts'), { recursive: true });
  fs.writeFileSync(path.join(root, 'scripts', 'install.sh'), '#!/bin/sh\nexit 99\n');
  if (git === 'dir') fs.mkdirSync(path.join(root, '.git'));
  if (git === 'file') {
    fs.writeFileSync(path.join(root, '.git'), 'gitdir: /tmp/fake.git/worktrees/x\n');
  }
  if (git === 'junk') fs.writeFileSync(path.join(root, '.git'), 'not-a-gitdir\n');
}

function assertInstallerNotCalled(result, calls) {
  assert.equal(calls, 0, 'postinstall must not download or compile');
  assert.equal(result.skipped, 'source-checkout');
}

test('this repository is classified as a source checkout', () => {
  // Worktree shape: `.git` is a file, not a directory. The predicate has to
  // accept that or `npm install` in an Orca worktree still installs.
  const root = path.resolve(__dirname, '..', '..');
  assert.equal(isSourceCheckout(root), true);
});

test('postinstall on this checkout does not call the installer', () => {
  let calls = 0;
  const result = runPostinstall({
    env: {},
    install: () => {
      calls += 1;
      return { ok: true, platform: 'test' };
    },
    exit: (code) => {
      throw new Error(`exit ${code}`);
    },
  });
  assertInstallerNotCalled(result, calls);
});

test('DEEPSEEK_BUILD_ALLOW_SOURCE_BUILD does not install from a checkout', (t) => {
  const root = withTempDir(t);
  writeCheckoutMarkers(root, 'file');
  let calls = 0;
  const result = runPostinstall({
    root,
    env: { DEEPSEEK_BUILD_ALLOW_SOURCE_BUILD: '1', DEEPSEEK_BUILD_HOME: path.join(root, 'home') },
    install: () => {
      calls += 1;
      return { ok: true, platform: 'test' };
    },
    exit: (code) => {
      throw new Error(`exit ${code}`);
    },
  });
  assertInstallerNotCalled(result, calls);
  assert.equal(fs.existsSync(path.join(root, 'home')), false, 'skip must not create the product home');
});

test('a clone (.git directory) and a worktree (gitdir file) are checkouts', (t) => {
  const clone = withTempDir(t);
  writeCheckoutMarkers(clone, 'dir');
  assert.equal(isSourceCheckout(clone), true);

  const worktree = withTempDir(t);
  writeCheckoutMarkers(worktree, 'file');
  assert.equal(isSourceCheckout(worktree), true);
});

test('a .git symlink to a directory counts as metadata', (t) => {
  const root = withTempDir(t);
  writeCheckoutMarkers(root, null);
  const gitdir = path.join(root, 'real-git');
  fs.mkdirSync(gitdir);
  fs.symlinkSync(gitdir, path.join(root, '.git'));
  assert.equal(isSourceCheckout(root), true);
});

test('a junk file named .git is not a checkout', (t) => {
  const root = withTempDir(t);
  writeCheckoutMarkers(root, 'junk');
  assert.equal(isSourceCheckout(root), false);
});

test('missing any one of the three markers is not a checkout', (t) => {
  const noGit = withTempDir(t);
  writeCheckoutMarkers(noGit, null);
  assert.equal(isSourceCheckout(noGit), false, 'zipball: Cargo.toml + install.sh, no .git');

  const noCargo = withTempDir(t);
  writeCheckoutMarkers(noCargo, 'dir');
  fs.rmSync(path.join(noCargo, 'Cargo.toml'));
  assert.equal(isSourceCheckout(noCargo), false);

  const noInstall = withTempDir(t);
  writeCheckoutMarkers(noInstall, 'dir');
  fs.rmSync(path.join(noInstall, 'scripts', 'install.sh'));
  assert.equal(isSourceCheckout(noInstall), false);

  const cargoDir = withTempDir(t);
  writeCheckoutMarkers(cargoDir, 'dir');
  fs.rmSync(path.join(cargoDir, 'Cargo.toml'));
  fs.mkdirSync(path.join(cargoDir, 'Cargo.toml'));
  assert.equal(isSourceCheckout(cargoDir), false, 'Cargo.toml must be a file');
});

test('a packed package nested inside a checkout is not itself a checkout', (t) => {
  const checkout = withTempDir(t);
  writeCheckoutMarkers(checkout, 'dir');
  const packed = path.join(checkout, 'lib', 'node_modules', '@innocarpe', 'deepseek-build');
  fs.mkdirSync(packed, { recursive: true });
  fs.writeFileSync(path.join(packed, 'package.json'), '{"version":"9.9.9"}\n');
  assert.equal(isSourceCheckout(checkout), true);
  assert.equal(isSourceCheckout(packed), false);
});

test('a packed tree still calls the installer and does not exit on success', (t) => {
  const root = withTempDir(t);
  fs.writeFileSync(path.join(root, 'package.json'), '{"version":"9.9.9"}\n');
  let got;
  const result = runPostinstall({
    root,
    env: { DEEPSEEK_BUILD_HOME: path.join(root, 'home') },
    install: (opts) => {
      got = opts;
      return { ok: true, platform: 'darwin-arm64' };
    },
    exit: (code) => {
      throw new Error(`exit ${code}`);
    },
  });
  assert.equal(result.ok, true);
  assert.equal(got.version, '9.9.9');
  assert.equal(got.binDir, path.join(root, 'home', 'bin'));
  assert.equal(got.pkgNativeBin, path.join(root, 'npm', 'native-bin'));
});

test('npm install in the checkout (INIT_CWD is the checkout) skips', (t) => {
  const root = withTempDir(t);
  writeCheckoutMarkers(root, 'dir');
  let calls = 0;
  const result = runPostinstall({
    root,
    env: {
      INIT_CWD: root,
      npm_config_prefix: path.join(root, 'user-prefix'),
      npm_config_global: 'false',
    },
    install: () => {
      calls += 1;
      return { ok: true };
    },
    exit: () => {
      throw new Error('exit');
    },
  });
  assertInstallerNotCalled(result, calls);
});

test('npm install --prefix the checkout from elsewhere skips', (t) => {
  const root = withTempDir(t);
  writeCheckoutMarkers(root, 'dir');
  const elsewhere = withTempDir(t);
  let calls = 0;
  const result = runPostinstall({
    root,
    env: { INIT_CWD: elsewhere, npm_config_prefix: root },
    install: () => {
      calls += 1;
      return { ok: true };
    },
    exit: () => {
      throw new Error('exit');
    },
  });
  assertInstallerNotCalled(result, calls);
});

test('INIT_CWD through a symlink to the checkout still skips', (t) => {
  const root = withTempDir(t);
  writeCheckoutMarkers(root, 'dir');
  const linkParent = withTempDir(t);
  const link = path.join(linkParent, 'checkout');
  fs.symlinkSync(root, link);
  let calls = 0;
  const result = runPostinstall({
    root,
    env: { INIT_CWD: link, npm_config_prefix: path.join(linkParent, 'prefix') },
    install: () => {
      calls += 1;
      return { ok: true };
    },
    exit: () => {
      throw new Error('exit');
    },
  });
  assertInstallerNotCalled(result, calls);
});

test('npm install -g . still calls the installer (npm_config_global=true)', (t) => {
  const root = withTempDir(t);
  writeCheckoutMarkers(root, 'file');
  fs.writeFileSync(path.join(root, 'package.json'), '{"version":"9.9.9"}\n');
  let calls = 0;
  const result = runPostinstall({
    root,
    env: {
      npm_config_global: 'true',
      INIT_CWD: root,
      DEEPSEEK_BUILD_HOME: path.join(root, 'home'),
    },
    install: (opts) => {
      calls += 1;
      assert.equal(opts.version, '9.9.9');
      return { ok: true, platform: 'darwin-arm64' };
    },
    exit: (code) => {
      throw new Error(`exit ${code}`);
    },
  });
  assert.equal(calls, 1);
  assert.equal(result.ok, true);
});

test('npm install of the checkout from another directory still installs', (t) => {
  const root = withTempDir(t);
  writeCheckoutMarkers(root, 'dir');
  fs.writeFileSync(path.join(root, 'package.json'), '{"version":"9.9.9"}\n');
  const consumer = withTempDir(t);
  const prefix = withTempDir(t);
  let calls = 0;
  const result = runPostinstall({
    root,
    env: {
      INIT_CWD: consumer,
      npm_config_prefix: prefix,
      DEEPSEEK_BUILD_HOME: path.join(root, 'home'),
    },
    install: () => {
      calls += 1;
      return { ok: true, platform: 'darwin-arm64' };
    },
    exit: (code) => {
      throw new Error(`exit ${code}`);
    },
  });
  assert.equal(calls, 1);
  assert.equal(result.ok, true);
});

test('a packed tree that fails install still exits 1', (t) => {
  const root = withTempDir(t);
  fs.writeFileSync(path.join(root, 'package.json'), '{"version":"9.9.9"}\n');
  let code = null;
  const result = runPostinstall({
    root,
    env: { DEEPSEEK_BUILD_HOME: path.join(root, 'home') },
    install: () => ({ ok: false, error: 'missing asset', url: 'https://example.invalid/x' }),
    exit: (c) => {
      code = c;
    },
  });
  assert.equal(code, 1);
  assert.equal(result.failed, true);
});
