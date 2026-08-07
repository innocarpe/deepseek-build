'use strict';

/**
 * After `npm install -g @innocarpe/deepseek-build`:
 *
 * **Default (ADR 0009):** download prebuilt natives from GitHub Releases for
 * this package version — seconds, no Rust. Same expectation as Claude Code /
 * Codex / Grok Build: install finishes, `dsb` works.
 *
 * Optional: DEEPSEEK_BUILD_ALLOW_SOURCE_BUILD=1 falls back to cargo install
 * when prebuilt is missing (dev / unsupported platform).
 *
 * Skip entirely: DEEPSEEK_BUILD_SKIP_POSTINSTALL=1
 */

const fs = require('fs');
const os = require('os');
const path = require('path');
const { spawnSync } = require('child_process');
const { installPrebuilt } = require('./install-prebuilt');

const pkgRoot = path.resolve(__dirname, '..', '..');

function readPackageVersion() {
  try {
    const p = JSON.parse(fs.readFileSync(path.join(pkgRoot, 'package.json'), 'utf8'));
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

function trySourceBuild(binDir) {
  const installSh = path.join(pkgRoot, 'scripts', 'install.sh');
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
    cwd: pkgRoot,
    env: process.env,
  });
  return r.status === 0;
}

function main() {
  if (process.env.DEEPSEEK_BUILD_SKIP_POSTINSTALL === '1') {
    console.log('deepseek-build postinstall: skipped (DEEPSEEK_BUILD_SKIP_POSTINSTALL=1)');
    return;
  }

  const version = readPackageVersion();
  if (!version) {
    console.warn('deepseek-build postinstall: cannot read package.json version');
    return;
  }

  const home = process.env.DEEPSEEK_BUILD_HOME || path.join(os.homedir(), '.deepseek-build');
  const binDir = path.join(home, 'bin');
  const pkgNativeBin = path.join(pkgRoot, 'npm', 'native-bin');

  const result = installPrebuilt({
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
    return;
  }

  console.warn('deepseek-build postinstall: prebuilt install failed:');
  console.warn(`  ${result.error || 'unknown error'}`);
  if (result.url) console.warn(`  tried: ${result.url}`);

  if (process.env.DEEPSEEK_BUILD_ALLOW_SOURCE_BUILD === '1') {
    if (trySourceBuild(binDir)) {
      console.log('deepseek-build postinstall: source build OK →', binDir);
      printPathHint(binDir);
      return;
    }
  } else {
    console.warn('');
    console.warn('Default install does not compile from source (by design).');
    console.warn('  • Wait for the GitHub Release asset for your platform, or');
    console.warn('  • Dev/source: DEEPSEEK_BUILD_ALLOW_SOURCE_BUILD=1 npm i -g …');
    console.warn('  • Or use a checkout: ./scripts/install.sh');
  }

  printPathHint(binDir);
}

main();
