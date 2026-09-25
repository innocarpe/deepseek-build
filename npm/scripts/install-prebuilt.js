'use strict';

/**
 * Download + extract prebuilt natives from GitHub Releases (ADR 0009).
 * Default path for `npm i -g` — no Rust compile.
 */

const { spawnSync } = require('child_process');
const fs = require('fs');
const os = require('os');
const path = require('path');
const { platformId, releaseAssetName, releaseDownloadUrl } = require('../lib/platform');

const REQUIRED = ['deepseek-build', 'dsb', 'deepseek-build-agent'];

/**
 * Version stamps the agent honours at *runtime* (`installed()` in the vendored
 * `xai-grok-version` crate: `DEEPSEEK_BUILD_VERSION`, then `GROK_TEST_VERSION`,
 * else the version baked into the binary).
 *
 * They are legitimate on a *build* shell — `scripts/build-grok-pager.sh`
 * exports `DEEPSEEK_BUILD_VERSION` so a release build bakes the product SemVer
 * — but a process that merely inherits one makes the agent print the caller's
 * value instead of its own. So the self-check below runs without them: an
 * install must not be judged against whatever version the caller's shell
 * happens to be stamping.
 *
 * The release workflows already strip them by hand
 * (`.github/workflows/release-prebuilt.yml`, `.github/workflows/publish-npm.yml`:
 * `env -u DEEPSEEK_BUILD_VERSION -u GROK_TEST_VERSION`). Keep this list in
 * step with theirs.
 *
 * `GROK_VERSION` is deliberately absent: the vendor `build.rs` reads it at
 * *compile* time only. Measured on a shipped binary, `GROK_VERSION=7.7.7`
 * still prints the built version.
 */
const VERSION_STAMP_ENV = ['DEEPSEEK_BUILD_VERSION', 'GROK_TEST_VERSION'];

function ensureDir(p) {
  fs.mkdirSync(p, { recursive: true });
}

/**
 * The environment a freshly installed binary is probed with: the caller's env
 * minus the version stamps (see `VERSION_STAMP_ENV`). PATH/HOME stay — the
 * binary needs them to exec at all.
 */
function selfCheckEnv(base = process.env) {
  const env = { ...base };
  for (const key of VERSION_STAMP_ENV) {
    delete env[key];
  }
  return env;
}

/**
 * Run `<binary> --version` and require the package version in the output.
 *
 * Guards against a corrupted install: a truncated/partial binary (e.g. from a
 * disk-full write) can pass `file`/`codesign` checks yet be killed by the OS
 * at exec ("Killed: 9" / taskgated invalid signature). So the freshly written
 * binary must actually run and report the version it was installed for.
 *
 * Runs with `selfCheckEnv()`: with an inherited version stamp the binary
 * reports the caller's version, this check calls a good install corrupt, and
 * the caller used to delete it.
 */
function verifyInstalledBin(dest, name, version) {
  const r = spawnSync(dest, ['--version'], {
    encoding: 'utf8',
    timeout: 20000,
    env: selfCheckEnv(),
  });
  if (r.status !== 0) {
    return `\`${name} --version\` exited with status ${r.status} (expected 0)`;
  }
  const out = `${r.stdout || ''}${r.stderr || ''}`;
  if (!out.includes(version)) {
    return `\`${name} --version\` printed "${out.trim().slice(0, 60)}" (expected to contain ${version})`;
  }
  return null;
}

function hasCmd(cmd) {
  const r = spawnSync(process.platform === 'win32' ? 'where' : 'which', [cmd], {
    encoding: 'utf8',
  });
  return r.status === 0;
}

/**
 * Run a download/extract helper. Keeps the caller's env on purpose: curl,
 * wget and tar need PATH (and curl its proxy vars), and none of them reads a
 * version stamp — only the binary probe does.
 */
function run(cmd, args, opts = {}) {
  const r = spawnSync(cmd, args, {
    encoding: 'utf8',
    stdio: opts.stdio || 'pipe',
    env: opts.env || process.env,
  });
  return r;
}

function selfCheckFailure(problem, binDir) {
  return (
    `installed binary failed self-check: ${problem}.\n` +
    `  The download may be corrupt (truncated download, wrong architecture, low disk space).\n` +
    `  ${binDir} was left unchanged — a previous installation, if any, still runs there.\n` +
    `  Retry: npm rebuild -g @innocarpe/deepseek-build`
  );
}

/**
 * Install the binaries in `stageDir` into `binDir`.
 *
 * Nothing under `binDir` is touched until *every* staged binary has run and
 * reported `version`. Verifying in place could only report failure by deleting
 * what had just been written — which also deleted whatever the user was
 * running before, with no backup anywhere in the package.
 *
 * Each binary is first copied into a staging dir *inside* `binDir`: that copy
 * gets a fresh inode (a stale code-signature rejection cached per inode is a
 * known reinstall failure) and its rename cannot cross a filesystem boundary.
 * A failed check leaves the staging dir to be cleaned up and `binDir` exactly
 * as it was.
 *
 * @param {object} opts
 * @param {string} opts.stageDir directory holding the extracted `REQUIRED` binaries
 * @param {string} opts.version package SemVer (no leading v)
 * @param {string} opts.binDir install destination (~/.deepseek-build/bin)
 * @param {string} [opts.pkgNativeBin] optional npm package npm/native-bin mirror
 * @param {string} [opts.platform] platform id, echoed in the result
 * @param {string} [opts.url] release URL, echoed in the result
 * @returns {{ ok: boolean, error?: string, platform?: string, url?: string }}
 */
function installFromStageDir({ stageDir, version, binDir, pkgNativeBin, platform, url }) {
  ensureDir(binDir);
  let staging;
  try {
    staging = fs.mkdtempSync(path.join(binDir, '.install-'));
  } catch (e) {
    return {
      ok: false,
      error:
        `could not create a staging dir under ${binDir}: ${e.message}\n` +
        `  Retry: npm rebuild -g @innocarpe/deepseek-build`,
      platform,
      url,
    };
  }

  try {
    for (const name of REQUIRED) {
      const src = path.join(stageDir, name);
      if (!fs.existsSync(src)) {
        return { ok: false, error: `tarball missing required binary: ${name}`, platform, url };
      }
      const staged = path.join(staging, name);
      try {
        fs.copyFileSync(src, staged);
      } catch (e) {
        return {
          ok: false,
          error:
            `could not stage ${name} under ${binDir}: ${e.message}\n` +
            `  ${binDir} was left unchanged.\n` +
            `  Retry: npm rebuild -g @innocarpe/deepseek-build`,
          platform,
          url,
        };
      }
      try {
        fs.chmodSync(staged, 0o755);
      } catch {
        // windows
      }
      const problem = verifyInstalledBin(staged, name, version);
      if (problem) {
        return { ok: false, error: selfCheckFailure(problem, binDir), platform, url };
      }
    }

    // Every binary verified — publish them. rename(2) replaces the destination
    // atomically, so there is no window where `binDir` holds neither the old
    // nor the new binary.
    for (const name of REQUIRED) {
      const dest = path.join(binDir, name);
      try {
        fs.renameSync(path.join(staging, name), dest);
      } catch (e) {
        return {
          ok: false,
          error:
            `could not install ${name}: ${e.message}\n` +
            `  ${binDir} may hold a partial install.\n` +
            `  Retry: npm rebuild -g @innocarpe/deepseek-build`,
          platform,
          url,
        };
      }
      if (pkgNativeBin) {
        ensureDir(pkgNativeBin);
        const mirror = path.join(pkgNativeBin, name);
        fs.copyFileSync(dest, mirror);
        try {
          fs.chmodSync(mirror, 0o755);
        } catch {
          // ignore
        }
      }
    }

    return { ok: true, platform, url };
  } finally {
    try {
      fs.rmSync(staging, { recursive: true, force: true });
    } catch {
      // ignore
    }
  }
}

/**
 * @param {object} opts
 * @param {string} opts.version package SemVer (no leading v)
 * @param {string} opts.binDir install destination (~/.deepseek-build/bin)
 * @param {string} [opts.pkgNativeBin] optional npm package npm/native-bin mirror
 * @returns {{ ok: boolean, error?: string, platform?: string, url?: string }}
 */
function installPrebuilt(opts) {
  const { version, binDir, pkgNativeBin } = opts;
  const platform = platformId();
  if (!platform) {
    return {
      ok: false,
      error:
        `unsupported platform ${process.platform}/${process.arch}. ` +
        `Supported: darwin-arm64 (Apple Silicon macOS only).`,
    };
  }

  const url = releaseDownloadUrl(version, platform);
  const asset = releaseAssetName(version, platform);
  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), 'dsb-prebuilt-'));
  const tarPath = path.join(tmpDir, asset);

  try {
    ensureDir(binDir);

    if (!hasCmd('curl') && !hasCmd('wget')) {
      return { ok: false, error: 'need curl or wget to download prebuilt binaries', platform, url };
    }

    console.log(`deepseek-build: downloading prebuilt (${platform})…`);
    console.log(`  ${url}`);

    let dl;
    if (hasCmd('curl')) {
      dl = run('curl', ['-fsSL', '--retry', '3', '--retry-delay', '1', '-o', tarPath, url], {
        stdio: 'inherit',
      });
    } else {
      dl = run('wget', ['-q', '-O', tarPath, url], { stdio: 'inherit' });
    }
    if (dl.status !== 0) {
      return {
        ok: false,
        error:
          `download failed for ${asset} (HTTP error or missing release asset).\n` +
          `  Tag v${version} must publish ${asset} on GitHub Releases.`,
        platform,
        url,
      };
    }

    const st = fs.statSync(tarPath);
    if (st.size < 1024) {
      return { ok: false, error: `download too small (${st.size} bytes) — not a valid tarball`, platform, url };
    }

    // Extract into a staging dir (tar layout: flat files at root), then hand
    // the extracted set to installFromStageDir for verify-then-publish.
    const stage = path.join(tmpDir, 'out');
    ensureDir(stage);
    const tar = run('tar', ['-xzf', tarPath, '-C', stage]);
    if (tar.status !== 0) {
      return {
        ok: false,
        error: `tar extract failed: ${(tar.stderr || tar.stdout || '').toString().slice(0, 200)}`,
        platform,
        url,
      };
    }

    return installFromStageDir({ stageDir: stage, version, binDir, pkgNativeBin, platform, url });
  } finally {
    try {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    } catch {
      // ignore
    }
  }
}

module.exports = {
  installPrebuilt,
  installFromStageDir,
  verifyInstalledBin,
  selfCheckEnv,
  VERSION_STAMP_ENV,
  REQUIRED,
};