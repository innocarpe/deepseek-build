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
