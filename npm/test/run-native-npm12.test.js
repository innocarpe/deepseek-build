'use strict';

/**
 * npm 12 denies this package's postinstall unless the installer opts in, and
 * still reports success. The published tarball has no `npm/native-bin/` —
 * postinstall is what fills it — so a skipped script leaves the shims with
 * nothing to run.
 *
 * A home binary whose version disagrees with the package is warned about by
 * `mismatchWarning` (version-align) and still executed. This wrapper does not
 * stamp `DEEPSEEK_BUILD_VERSION`. That warning's npm 12 command is pinned in
 * `version-align.test.js`.
 *
 * Pinned here:
 *
 * 1. No payload and no resolved binary is the blocked install (exit 127 path).
 * 2. A native-bin that holds the release set is not that failure.
 * 3. The text the user sees names the install and rebuild commands that work
 *    on npm 12. The spec-less form npm itself prints does not.
 *
 * Nothing here touches the user's `~/.deepseek-build`.
 */

const assert = require('node:assert/strict');
const fs = require('fs');
const os = require('os');
const path = require('path');
const test = require('node:test');

const {
  detectBlockedInstall,
  blockedInstallMessage,
  missingBinaryMessage,
  NPM12_INSTALL_COMMAND,
  NPM12_REBUILD_COMMAND,
} = require('../lib/run-native');

const INSTALL = 'npm install -g --allow-scripts=@innocarpe/deepseek-build @innocarpe/deepseek-build';
const REBUILD = 'npm rebuild -g --allow-scripts=@innocarpe/deepseek-build @innocarpe/deepseek-build';

function withTempDir(t) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'dsb-npm12-'));
  t.after(() => fs.rmSync(dir, { recursive: true, force: true }));
  return dir;
}

function fillNativeBin(root) {
  const dir = path.join(root, 'npm', 'native-bin');
  fs.mkdirSync(dir, { recursive: true });
  for (const name of ['deepseek-build', 'dsb', 'deepseek-build-agent']) {
    fs.writeFileSync(path.join(dir, name), 'x');
  }
  return dir;
}

test('detectBlockedInstall is true when postinstall left no binary', (t) => {
  const root = withTempDir(t);
  assert.equal(detectBlockedInstall({ pkgRoot: root, resolvedBinary: null }), true);
});

test('a partial native-bin is still a blocked install', (t) => {
  const root = withTempDir(t);
  const dir = path.join(root, 'npm', 'native-bin');
  fs.mkdirSync(dir, { recursive: true });
  fs.writeFileSync(path.join(dir, 'dsb'), 'x');
  assert.equal(detectBlockedInstall({ pkgRoot: root }), true);
});

test('detectBlockedInstall is false once native-bin holds the release set', (t) => {
  const root = withTempDir(t);
  fillNativeBin(root);
  assert.equal(detectBlockedInstall({ pkgRoot: root, resolvedBinary: null }), false);
});

test('a resolved binary is not the hard-fail blocked install', (t) => {
  const root = withTempDir(t);
  assert.equal(
    detectBlockedInstall({ pkgRoot: root, resolvedBinary: '/tmp/deepseek-build' }),
    false
  );
});

test('blocked install message names the npm 12 commands that work', () => {
  const text = blockedInstallMessage('dsb', ['/tmp/candidate']);
  assert.match(text, /npm 12/);
  assert.match(text, /blocks dependency install scripts/);
  assert.equal(NPM12_INSTALL_COMMAND, INSTALL);
  assert.equal(NPM12_REBUILD_COMMAND, REBUILD);
  assert.match(text, new RegExp(INSTALL.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')));
  assert.match(text, new RegExp(REBUILD.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')));
  // npm's own hint omits the package spec and fails with ENOENT package.json.
  // Every opt-in install/rebuild line we print has to repeat the spec.
  // Mentioning the broken rebuild command is allowed only to say it stays blocked.
  for (const line of text.split('\n')) {
    if (!line.includes('--allow-scripts')) continue;
    if (line.includes('npm install -g') || line.includes('npm rebuild -g')) {
      assert.match(line, /@innocarpe\/deepseek-build @innocarpe\/deepseek-build/);
    }
  }
  assert.match(text, /`npm rebuild -g @innocarpe\/deepseek-build` stays blocked/);
});

test('the generic missing-binary hint also uses the working install command', () => {
  const text = missingBinaryMessage('dsb', ['/tmp/candidate']);
  assert.match(text, new RegExp(INSTALL.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')));
});
