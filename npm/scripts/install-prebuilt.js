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

function ensureDir(p) {
  fs.mkdirSync(p, { recursive: true });
}

/**
 * Run `<binary> --version` and require the package version in the output.
 *
 * Guards against a corrupted install: a truncated/partial binary (e.g. from a
 * disk-full write) can pass `file`/`codesign` checks yet be killed by the OS
 * at exec ("Killed: 9" / taskgated invalid signature). Installing over a bad
 * file also keeps its inode, which can retain a stale signature-rejection
 * cache entry. We therefore (1) remove the destination first so a fresh
 * inode is written, and (2) verify the freshly installed binary actually
 * runs before declaring success.
 */
function verifyInstalledBin(dest, name, version) {
  const r = spawnSync(dest, ['--version'], { encoding: 'utf8', timeout: 20000 });
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

function run(cmd, args, opts = {}) {
  const r = spawnSync(cmd, args, {
    encoding: 'utf8',
    stdio: opts.stdio || 'pipe',
    env: opts.env || process.env,
  });
  return r;
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

    // Extract into a staging dir, then copy required bins (tar layout: flat files at root)
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

    for (const name of REQUIRED) {
      const src = path.join(stage, name);
      if (!fs.existsSync(src)) {
        return {
          ok: false,
          error: `tarball missing required binary: ${name}`,
          platform,
          url,
        };
      }
      const dest = path.join(binDir, name);
      // Remove first so a fresh inode is created: overwriting in place can
      // keep a stale code-signature rejection cached by the OS for that
      // inode (symptom: binary instantly SIGKILLed at exec after reinstall).
      try {
        fs.rmSync(dest, { force: true });
      } catch {
        // ignore
      }
      fs.copyFileSync(src, dest);
      try {
        fs.chmodSync(dest, 0o755);
      } catch {
        // windows
      }
      if (pkgNativeBin) {
        ensureDir(pkgNativeBin);
        fs.copyFileSync(src, path.join(pkgNativeBin, name));
        try {
          fs.chmodSync(path.join(pkgNativeBin, name), 0o755);
        } catch {
          // ignore
        }
      }
    }

    // Self-check the installed binaries actually run (catches truncated
    // writes, e.g. disk-full during install) and report a clear error.
    for (const name of REQUIRED) {
      const dest = path.join(binDir, name);
      const problem = verifyInstalledBin(dest, name, version);
      if (problem) {
        // Do not leave a broken binary around for the next attempt.
        try {
          fs.rmSync(dest, { force: true });
        } catch {
          // ignore
        }
        return {
          ok: false,
          error:
            `installed binary failed self-check: ${problem}.\n` +
            `  The install may have been corrupted (e.g. low disk space).\n` +
            `  Remove ${binDir} and reinstall (npm rebuild -g @innocarpe/deepseek-build).`,
          platform,
          url,
        };
      }
    }

    return { ok: true, platform, url };
  } finally {
    try {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    } catch {
      // ignore
    }
  }
}

module.exports = { installPrebuilt, REQUIRED };
