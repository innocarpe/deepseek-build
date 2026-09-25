'use strict';

/**
 * After `npm install -g @innocarpe/deepseek-build`:
 *
 * **Default (ADR 0009):** download prebuilt natives from GitHub Releases for
 * this package version — seconds, no Rust. Same expectation as Claude Code /
 * Codex / Grok Build: install finishes, `dsb` works.
 *
 * Optional: DEEPSEEK_BUILD_ALLOW_SOURCE_BUILD=1 falls back to cargo install
 * when prebuilt is missing (dev / unsupported platform). That fallback runs
 * only for a packed install. A git checkout is not a packed install.
 *
 * Skip entirely: DEEPSEEK_BUILD_SKIP_POSTINSTALL=1
 *
 * Source checkout: `npm install` in the git checkout (and `node` of this
 * file there) returns without downloading or compiling. A global install,
 * including `npm install -g .`, still installs — npm runs this script from
 * the checkout but sets `npm_config_global=true`. See
 * `isProjectCheckoutInstall`.
 */

const fs = require('fs');
const os = require('os');
const path = require('path');
const { spawnSync } = require('child_process');
const { installPrebuilt } = require('./install-prebuilt');

const pkgRoot = path.resolve(__dirname, '..', '..');

function isFile(p) {
  try {
    return fs.statSync(p).isFile();
  } catch {
    return false;
  }
}

/**
 * `.git` is a directory in a clone and a `gitdir:` file in a worktree.
 * Anything else (including a junk file that happens to be named `.git`) is
 * not metadata. npm never packs a `.git` entry, so this is false in every
 * registry and `npm pack` tree.
 */
function hasGitMetadata(root) {
  const gitPath = path.join(root, '.git');
  let st;
  try {
    st = fs.statSync(gitPath);
  } catch {
    return false;
  }
  if (st.isDirectory()) return true;
  if (!st.isFile()) return false;
  let head = '';
  try {
    const fd = fs.openSync(gitPath, 'r');
    try {
      const buf = Buffer.alloc(64);
      const n = fs.readSync(fd, buf, 0, buf.length, 0);
      head = buf.subarray(0, n).toString('utf8');
    } finally {
      fs.closeSync(fd);
    }
  } catch {
    return false;
  }
  return /^gitdir:\s+\S/.test(head);
}

/**
 * True only when `root` is the git checkout this package is developed from.
 *
 * All three must be present. Any one missing → not a checkout → the caller
 * installs. That is the conservative direction: a registry install that we
 * misread as a checkout would not install `deepseek-build` / `dsb`.
 *
 * The root is this script's directory (`npm/scripts/../..`), never
 * `process.cwd()`. A registry install and an `npm pack` tarball lack the
 * trio. Do not walk up to a parent `.git`: a packed copy sitting inside
 * some other repo must still install.
 *
 * `npm install -g .` is not decided here. On npm 12.1.0 it symlinks this
 * checkout into the prefix and runs the script in the checkout, so the
 * trio is present. `isProjectCheckoutInstall` keeps that global install.
 *
 * A GitHub zipball has `Cargo.toml` and `scripts/install.sh` and no `.git`.
 * It is not classified as a checkout.
 *
 * @param {string} root package root
 * @returns {boolean}
 */
function isSourceCheckout(root) {
  if (!root || typeof root !== 'string') return false;
  return (
    hasGitMetadata(root) &&
    isFile(path.join(root, 'Cargo.toml')) &&
    isFile(path.join(root, 'scripts', 'install.sh'))
  );
}

function samePath(a, b) {
  if (!a || !b) return false;
  try {
    return fs.realpathSync(a) === fs.realpathSync(b);
  } catch {
    return false;
  }
}

/**
 * True when this run is the checkout's own `npm install` (or a direct
 * `node npm/scripts/postinstall.js`), and must not install the product.
 *
 * Measured on npm 12.1.0 (2026-09-26):
 *
 * | Invocation | npm_config_global | INIT_CWD | npm_config_prefix | Skip? |
 * |---|---|---|---|---|
 * | `npm install` in the checkout | unset | the checkout | the user's prefix | yes |
 * | `npm install --prefix <checkout>` from elsewhere | unset | the other dir | the checkout | yes |
 * | `node npm/scripts/postinstall.js` | unset | unset | unset | yes |
 * | `npm install -g .` | `true` | the checkout | (ignored) | no |
 * | `npm install <checkout>` from another directory | unset | the other dir | the user's prefix | no |
 *
 * `npm install -g .` symlinks the checkout into the prefix and runs this
 * script with `argv` pointing at the checkout, so the three source files
 * are visible. The global flag is what keeps that smoke path installing.
 * A registry tarball never has the three files, so this function is false
 * there before the npm variables are consulted.
 *
 * @param {string} root package root
 * @param {NodeJS.ProcessEnv} env
 * @returns {boolean}
 */
function isProjectCheckoutInstall(root, env) {
  if (!isSourceCheckout(root)) return false;
  if (env && env.npm_config_global === 'true') return false;
  const initCwd = env && env.INIT_CWD;
  const prefix = env && env.npm_config_prefix;
  if (samePath(initCwd, root) || samePath(prefix, root)) return true;
  // Direct `node` invocation: npm did not supply either variable.
  return !initCwd && (prefix == null || prefix === '');
}

function readPackageVersion(root) {
  try {
    const p = JSON.parse(fs.readFileSync(path.join(root, 'package.json'), 'utf8'));
    return p.version;
  } catch {
    return null;
  }
}

function printPathHint(binDir) {
  const pathEnv = process.env.PATH || '';
  if (!pathEnv.split(path.delimiter).includes(binDir)) {
    console.log('');
    console.log('Add to PATH if `dsb` is not found:');
    console.log(`  export PATH="${binDir}:$PATH"`);
  }
}

function trySourceBuild(root, binDir, env) {
  const installSh = path.join(root, 'scripts', 'install.sh');
  if (!fs.existsSync(installSh)) {
    console.warn(
      'deepseek-build: source fallback unavailable (no scripts/install.sh in this package).'
    );
    return false;
  }
  if (spawnSync('cargo', ['--version'], { encoding: 'utf8' }).status !== 0) {
    console.warn('deepseek-build: source fallback needs cargo (https://rustup.rs/).');
    return false;
  }
  console.log('deepseek-build: DEEPSEEK_BUILD_ALLOW_SOURCE_BUILD=1 — running scripts/install.sh …');
  const r = spawnSync('bash', [installSh, '--prefix', path.dirname(binDir)], {
    stdio: 'inherit',
    cwd: root,
    env,
  });
  return r.status === 0;
}

function main(opts = {}) {
  const env = opts.env || process.env;
  const root = opts.root || pkgRoot;
  const install = opts.install || installPrebuilt;
  const exit = opts.exit || ((code) => process.exit(code));

  if (env.DEEPSEEK_BUILD_SKIP_POSTINSTALL === '1') {
    console.log('deepseek-build postinstall: skipped (DEEPSEEK_BUILD_SKIP_POSTINSTALL=1)');
    return { skipped: 'env' };
  }

  // Before any download, compile, or write under ~/.deepseek-build.
  // DEEPSEEK_BUILD_ALLOW_SOURCE_BUILD does not override this: a checkout
  // builds with ./scripts/install.sh, not with `npm install`.
  // A global install (`npm_config_global=true`) is not this case.
  if (isProjectCheckoutInstall(root, env)) {
    console.log('deepseek-build postinstall: skipped (source checkout).');
    console.log('Registry install: npm install -g @innocarpe/deepseek-build');
    console.log('From this checkout: ./scripts/install.sh');
    return { skipped: 'source-checkout' };
  }

  const version = readPackageVersion(root);
  if (!version) {
    console.warn('deepseek-build postinstall: cannot read package.json version');
    return { skipped: 'no-version' };
  }

  const home = env.DEEPSEEK_BUILD_HOME || path.join(os.homedir(), '.deepseek-build');
  const binDir = path.join(home, 'bin');
  const pkgNativeBin = path.join(root, 'npm', 'native-bin');

  const result = install({
    version,
    binDir,
    pkgNativeBin,
  });

  if (result.ok) {
    console.log(`deepseek-build postinstall: prebuilt OK (${result.platform}) → ${binDir}`);
    console.log('');
    console.log('DeepSeek Build ready:');
    console.log('  dsb          # full-screen DeepSeek TUI');
    console.log('  dsb setup    # API key if needed');
    printPathHint(binDir);
    return { ok: true };
  }

  console.warn('deepseek-build postinstall: prebuilt install failed:');
  console.warn(`  ${result.error || 'unknown error'}`);
  if (result.url) console.warn(`  tried: ${result.url}`);

  if (env.DEEPSEEK_BUILD_ALLOW_SOURCE_BUILD === '1') {
    if (trySourceBuild(root, binDir, env)) {
      console.log('deepseek-build postinstall: source build OK →', binDir);
      printPathHint(binDir);
      return { ok: true, source: true };
    }
  } else {
    console.warn('');
    console.warn('Default install does not compile from source (by design).');
    console.warn('  • Wait for the GitHub Release asset for your platform, or');
    console.warn('  • Dev/source: DEEPSEEK_BUILD_ALLOW_SOURCE_BUILD=1 npm i -g …');
    console.warn('  • Or use a checkout: ./scripts/install.sh');
  }

  printPathHint(binDir);

  // Fail-close: a half-installed package (e.g. the agent self-check failed
  // and the binary was removed from the bin dir) must NOT report a
  // successful npm install — otherwise `dsb` silently runs a stale agent
  // and the fix the user installed never takes effect.
  console.error('deepseek-build postinstall: install FAILED — npm install will report failure.');
  exit(1);
  return { failed: true };
}

if (require.main === module) {
  main();
}

module.exports = {
  isSourceCheckout,
  isProjectCheckoutInstall,
  main,
};
