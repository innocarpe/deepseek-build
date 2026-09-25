'use strict';

const { spawnSync } = require('child_process');
const fs = require('fs');
const os = require('os');
const path = require('path');
const { readReportedVersion, mismatchWarning } = require('./version-align');

const pkgRoot = path.resolve(__dirname, '..', '..');

/**
 * Commands that actually install this package on npm 12.
 *
 * npm 12.0.0+ denies dependency install scripts unless the installer opts in,
 * and it still exits 0 (`added 1 package`). The form npm itself prints
 * (`npm install -g --allow-scripts=<name>` with no package spec) fails with
 * `ENOENT package.json`. The spec has to be repeated. `npm rebuild -g <name>`
 * is denied the same way. npm 11 and older run the script with or without
 * the flag (measured on 11.20.0).
 *
 * The package cannot set this itself. allowScripts is read only from the
 * installer — the CLI flag, user `.npmrc`, or the installing project's
 * `package.json`, which `npm i -g` skips — and the trusted identity comes
 * from the lockfile URL, not from the tarball's own manifest.
 */
const PKG_NAME = '@innocarpe/deepseek-build';
const NPM12_INSTALL_COMMAND = `npm install -g --allow-scripts=${PKG_NAME} ${PKG_NAME}`;
const NPM12_CONFIG_COMMAND = `npm config set allow-scripts=${PKG_NAME} --location=user`;
const NPM12_REBUILD_COMMAND = `npm rebuild -g --allow-scripts=${PKG_NAME} ${PKG_NAME}`;

/** Binaries postinstall mirrors into npm/native-bin/. Absent from the tarball. */
const NATIVE_BIN_NAMES = ['deepseek-build', 'dsb', 'deepseek-build-agent'];

function productHome() {
  return process.env.DEEPSEEK_BUILD_HOME || path.join(os.homedir(), '.deepseek-build');
}

function nativeBinDirectory(root) {
  return path.join(root, 'npm', 'native-bin');
}

/**
 * True when postinstall has copied the release binaries into the package.
 *
 * `npm/native-bin/` is not in the published tarball (`files` whitelist) and
 * is gitignored. Its absence is the packed state; postinstall is what fills
 * it. Missing files therefore mean the script did not run.
 */
function nativeBinPopulated(root) {
  const dir = nativeBinDirectory(root);
  return NATIVE_BIN_NAMES.every((name) => {
    try {
      return fs.statSync(path.join(dir, name)).isFile();
    } catch {
      return false;
    }
  });
}

/**
 * True when there is nothing to exec and postinstall left no payload.
 *
 * That is the npm 12 default: the script is skipped, npm still exits 0, and
 * the shims are the only thing that landed. A populated native-bin means the
 * script did run, so a missing binary is a different failure. A resolved
 * binary is still exec'd — a version mismatch is `mismatchWarning` in
 * `version-align.js`, which must not refuse to run.
 */
function detectBlockedInstall({ pkgRoot: root, resolvedBinary = null } = {}) {
  if (!root || nativeBinPopulated(root)) return false;
  return !resolvedBinary;
}

function blockedInstallMessage(binName, tried) {
  const triedLine = tried && tried.length ? `Tried: ${tried.join(', ')}\n` : '';
  return (
    `deepseek-build: native binary "${binName}" not found.\n` +
    triedLine +
    `npm 12 blocks dependency install scripts by default (npm 12.0.0+), so\n` +
    `postinstall never downloaded the agent. npm still reports a successful install.\n` +
    `Fix once:\n` +
    `  ${NPM12_INSTALL_COMMAND}\n` +
    `Or allow this package for later global installs, then install plainly:\n` +
    `  ${NPM12_CONFIG_COMMAND}\n` +
    `  npm install -g ${PKG_NAME}\n` +
    'npm rebuild needs the same opt-in. Plain `npm rebuild -g ' +
    PKG_NAME +
    '` stays blocked:\n' +
    `  ${NPM12_REBUILD_COMMAND}\n` +
    `npm 11 and older run install scripts without --allow-scripts.\n` +
    `Dev/source only: DEEPSEEK_BUILD_ALLOW_SOURCE_BUILD=1 or ./scripts/install.sh`
  );
}

function missingBinaryMessage(binName, tried) {
  return (
    `deepseek-build: native binary "${binName}" not found.\n` +
    `Tried: ${tried.join(', ')}\n` +
    `Fix (no Rust required for registry install):\n` +
    `  ${NPM12_INSTALL_COMMAND}\n` +
    `  # downloads prebuilts from GitHub Releases (ADR 0009)\n` +
    `  # npm 12: or ${NPM12_CONFIG_COMMAND}\n` +
    `  export PATH="$HOME/.deepseek-build/bin:$PATH"\n` +
    `Dev/source only: DEEPSEEK_BUILD_ALLOW_SOURCE_BUILD=1 or ./scripts/install.sh\n` +
    `Then: dsb`
  );
}

/**
 * Resolve native binary for deepseek-build / dsb.
 * Order: env → product home → cargo bin → package native-bin → package target/release
 */
function candidatePaths(binName) {
  const out = [];
  if (process.env.DEEPSEEK_BUILD_BIN) {
    out.push(process.env.DEEPSEEK_BUILD_BIN);
  }
  out.push(path.join(productHome(), 'bin', binName));

  const cargoHome = process.env.CARGO_HOME || path.join(os.homedir(), '.cargo');
  out.push(path.join(cargoHome, 'bin', binName));

  // npm package-local copies (postinstall may place agent here)
  out.push(path.join(pkgRoot, 'npm', 'native-bin', binName));
  out.push(path.join(pkgRoot, 'target', 'release', binName));
  out.push(path.join(pkgRoot, 'third_party', 'grok-build', 'target', 'release', 'xai-grok-pager'));

  return out;
}

function findBinary(binName) {
  for (const p of candidatePaths(binName)) {
    try {
      if (fs.existsSync(p) && fs.statSync(p).isFile()) {
        // For agent name, also accept xai-grok-pager path when looking for agent
        return p;
      }
    } catch {
      // continue
    }
  }
  return null;
}

function findAgentBinary() {
  const names = [
    process.env.DEEPSEEK_BUILD_AGENT_BIN,
    path.join(productHome(), 'bin', 'deepseek-build-agent'),
    path.join(pkgRoot, 'npm', 'native-bin', 'deepseek-build-agent'),
    path.join(pkgRoot, 'third_party', 'grok-build', 'target', 'release', 'xai-grok-pager'),
  ].filter(Boolean);
  for (const p of names) {
    try {
      if (fs.existsSync(p) && fs.statSync(p).isFile()) return p;
    } catch {
      // continue
    }
  }
  return null;
}

/**
 * Product contract: bare `dsb` / `deepseek-build` with no args on a TTY
 * must open the DeepSeek full-screen agent TUI.
 *
 * Prefer native wrapper (handles setup + splash + GROK_THEME). If wrapper
 * is missing but agent exists, exec agent directly so install still works.
 */
function run(binName, args) {
  // Warn and still exec. The package version is not stamped onto the child
  // (see productEnv), so a mismatched agent reports its own version.
  const agent = findAgentBinary();
  if (agent) {
    const msg = mismatchWarning(productVersion(), readReportedVersion(agent));
    if (msg) console.error(msg);
  }

  const isBare =
    args.length === 0 ||
    (args.length === 1 && (args[0] === 'agent' || args[0] === '--'));

  // Always prefer installed wrapper when present.
  let bin = findBinary(binName);
  if (!bin && binName === 'dsb') {
    bin = findBinary('deepseek-build');
  }

  if (!bin) {
    if (agent && (isBare || args[0] === 'agent')) {
      const agentArgs = args[0] === 'agent' ? args.slice(1) : args;
      return exec(agent, agentArgs, productEnv());
    }
    const tried = candidatePaths(binName);
    if (detectBlockedInstall({ pkgRoot, resolvedBinary: null })) {
      console.error(blockedInstallMessage(binName, tried));
    } else {
      console.error(missingBinaryMessage(binName, tried));
    }
    process.exit(127);
  }

  // Home and installer classification. The package version is not stamped
  // onto the child — see productEnv().
  return exec(bin, args, productEnv());
}

function productVersion() {
  try {
    const pkg = JSON.parse(fs.readFileSync(path.join(pkgRoot, 'package.json'), 'utf8'));
    if (pkg && typeof pkg.version === 'string' && pkg.version.trim()) {
      return pkg.version.trim();
    }
  } catch {
    // fall through
  }
  return process.env.DEEPSEEK_BUILD_VERSION || '';
}

function productEnv() {
  const home = process.env.DEEPSEEK_BUILD_HOME || path.join(os.homedir(), '.deepseek-build');
  const env = {
    ...process.env,
    GROK_HOME: process.env.GROK_HOME || home,
    // This wrapper IS an npm-managed install: classify as "npm" so update
    // checks consult the product npm registry instead of falling through to
    // the upstream Grok x.ai channel pointers (which advertise Grok Build
    // versions and would otherwise downgrade/overwrite the product).
    GROK_INSTALLER: process.env.GROK_INSTALLER || 'npm',
  };
  // Only force a theme via env when the user explicitly asked
  // (DEEPSEEK_BUILD_THEME / GROK_THEME). Injecting a default here overrode
  // `[ui].theme` in the product config at every launch, so an in-pager
  // `/theme` choice never survived a restart (agent_launch.rs contract:
  // "env is only for explicit user override").
  const userTheme = process.env.DEEPSEEK_BUILD_THEME || process.env.GROK_THEME;
  if (userTheme && userTheme.trim()) {
    env.GROK_THEME = userTheme;
    env.LC_GROK_THEME = userTheme;
  }
  // Do not stamp DEEPSEEK_BUILD_VERSION from package.json. The agent honours
  // that variable over the version baked into the binary (`installed()`).
  // A package older than the agent would then make the TUI and
  // `deepseek-build-agent --version` report the package. Measured: package
  // 5.7.0 plus a baked 6.0.0 binary printed `deepseek-build 5.7.0 (…) [alpha]`.
  // A value the caller already exported is left untouched via `process.env`.
  return env;
}

function exec(bin, args, env) {
  const result = spawnSync(bin, args, {
    stdio: 'inherit',
    env,
  });
  if (result.error) {
    console.error(`deepseek-build: failed to spawn ${bin}: ${result.error.message}`);
    process.exit(1);
  }
  process.exit(result.status === null ? 1 : result.status);
}

module.exports = {
  run,
  findBinary,
  findAgentBinary,
  candidatePaths,
  productEnv,
  productVersion,
  detectBlockedInstall,
  blockedInstallMessage,
  missingBinaryMessage,
  nativeBinPopulated,
  NPM12_INSTALL_COMMAND,
  NPM12_CONFIG_COMMAND,
  NPM12_REBUILD_COMMAND,
};
