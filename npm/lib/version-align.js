'use strict';

/**
 * Keep the npm package version and the installed agent version from silently
 * diverging.
 *
 * ADR 0009: `npm i -g @innocarpe/deepseek-build@X` downloads the GitHub
 * Release for X and installs `deepseek-build`, `dsb`, and
 * `deepseek-build-agent` together. They are one install, not two products
 * that drift. The in-app updater and a source install can still replace the
 * agent without replacing the global package. An older package's postinstall
 * must not then roll the agent backwards, and `dsb` must say when the two
 * versions disagree.
 *
 * `VERSION_STAMP_ENV` is the pair `installed()` honours at runtime
 * (`xai-grok-version`). A probe that judges the binary has to drop them, or
 * it reports the caller's stamp instead of the bake. `install-prebuilt.js`
 * re-exports this list; keep it in step with the release workflows.
 */

const { spawnSync } = require('child_process');

const VERSION_STAMP_ENV = ['DEEPSEEK_BUILD_VERSION', 'GROK_TEST_VERSION'];

function envWithoutVersionStamps(base = process.env) {
  const env = { ...base };
  for (const key of VERSION_STAMP_ENV) delete env[key];
  return env;
}

function parseSemver(input) {
  if (typeof input !== 'string') return null;
  const m = input.trim().match(/^(\d+)\.(\d+)\.(\d+)(?:-([0-9A-Za-z.]+))?$/);
  if (!m) return null;
  return {
    core: [Number(m[1]), Number(m[2]), Number(m[3])],
    pre: m[4] ? m[4].split('.') : [],
  };
}

function comparePre(a, b) {
  if (a.length === 0 && b.length === 0) return 0;
  if (a.length === 0) return 1;
  if (b.length === 0) return -1;
  const n = Math.max(a.length, b.length);
  for (let i = 0; i < n; i++) {
    if (a[i] === undefined) return -1;
    if (b[i] === undefined) return 1;
    const an = /^[0-9]+$/.test(a[i]);
    const bn = /^[0-9]+$/.test(b[i]);
    if (an && bn) {
      const d = Number(a[i]) - Number(b[i]);
      if (d !== 0) return d > 0 ? 1 : -1;
      continue;
    }
    if (an !== bn) return an ? -1 : 1;
    if (a[i] !== b[i]) return a[i] > b[i] ? 1 : -1;
  }
  return 0;
}

/** `1` when `a > b`, `-1` when `a < b`, `0` when equal, `null` if either is not SemVer. */
function compareSemver(a, b) {
  const pa = parseSemver(a);
  const pb = parseSemver(b);
  if (!pa || !pb) return null;
  for (let i = 0; i < 3; i++) {
    if (pa.core[i] !== pb.core[i]) return pa.core[i] > pb.core[i] ? 1 : -1;
  }
  return comparePre(pa.pre, pb.pre);
}

/** First SemVer in a `--version` line (`deepseek-build 6.0.0 (hash) [alpha]`). */
function reportedVersion(text) {
  const m = String(text).match(/(\d+\.\d+\.\d+(?:-[0-9A-Za-z.]+)?)/);
  return m ? m[1] : null;
}

/**
 * Run `<bin> --version` without the runtime stamps and return the SemVer it
 * prints. `null` when the file is missing, exits non-zero, or prints nothing
 * parseable — the caller then does not treat it as a version to protect.
 */
function readReportedVersion(bin, baseEnv = process.env) {
  let r;
  try {
    r = spawnSync(bin, ['--version'], {
      encoding: 'utf8',
      timeout: 20000,
      env: envWithoutVersionStamps(baseEnv),
    });
  } catch {
    return null;
  }
  if (!r || r.status !== 0) return null;
  return reportedVersion(`${r.stdout || ''}${r.stderr || ''}`);
}

/**
 * Error text when `existingVersion` is strictly newer than `packageVersion`.
 * `null` when the install may replace the agent (upgrade, reinstall, probe
 * failure, or `DEEPSEEK_BUILD_ALLOW_DOWNGRADE=1`).
 */
function downgradeRefusal(existingVersion, packageVersion, env = process.env) {
  if (env.DEEPSEEK_BUILD_ALLOW_DOWNGRADE === '1') return null;
  const cmp = compareSemver(existingVersion, packageVersion);
  if (cmp === null || cmp <= 0) return null;
  return (
    `refusing to replace deepseek-build-agent ${existingVersion} with package ${packageVersion}.\n` +
    `  The installed agent is newer. It was left unchanged.\n` +
    `  To align the npm package with the agent:\n` +
    `    npm install -g @innocarpe/deepseek-build@${existingVersion}\n` +
    `  To install this older package on purpose:\n` +
    `    DEEPSEEK_BUILD_ALLOW_DOWNGRADE=1 npm install -g @innocarpe/deepseek-build@${packageVersion}`
  );
}

/**
 * Stderr warning when the npm package and the agent binary disagree.
 * `null` when they match or either side is missing.
 *
 * `dsb --version` reports the native binary the shim executes, not
 * `package.json`. This warning is how a stale package stays visible.
 */
function mismatchWarning(packageVersion, agentVersion) {
  if (!packageVersion || !agentVersion) return null;
  const cmp = compareSemver(agentVersion, packageVersion);
  if (cmp === null || cmp === 0) return null;
  const lines = [
    `deepseek-build: npm package ${packageVersion} and deepseek-build-agent ${agentVersion} differ.`,
    '  `dsb` and `deepseek-build` run the installed binaries. `--version` reports that binary, not the npm package.',
  ];
  if (cmp > 0) {
    lines.push(
      '  The agent is newer. This package will not replace it unless DEEPSEEK_BUILD_ALLOW_DOWNGRADE=1.',
      `  Align the package: npm install -g @innocarpe/deepseek-build@${agentVersion}`
    );
  } else {
    lines.push(
      `  The agent is older than this package. Run the install again so postinstall can fetch ${packageVersion}.`
    );
  }
  return lines.join('\n');
}

module.exports = {
  VERSION_STAMP_ENV,
  envWithoutVersionStamps,
  compareSemver,
  reportedVersion,
  readReportedVersion,
  downgradeRefusal,
  mismatchWarning,
};
