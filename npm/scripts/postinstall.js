'use strict';

/**
 * Best-effort native install after npm install.
 * Prefer cargo install into ~/.deepseek-build when source + cargo are available.
 * Never fails the npm install hard if cargo is missing (wrappers print guidance).
 */

const { spawnSync } = require('child_process');
const fs = require('fs');
const os = require('os');
const path = require('path');

const pkgRoot = path.resolve(__dirname, '..', '..');
const cargoToml = path.join(pkgRoot, 'Cargo.toml');
const cliToml = path.join(pkgRoot, 'crates', 'dsb-cli', 'Cargo.toml');

function hasCargo() {
  const r = spawnSync('cargo', ['--version'], { encoding: 'utf8' });
  return r.status === 0;
}

function main() {
  if (process.env.DEEPSEEK_BUILD_SKIP_POSTINSTALL === '1') {
    console.log('deepseek-build postinstall: skipped (DEEPSEEK_BUILD_SKIP_POSTINSTALL=1)');
    return;
  }
  if (!fs.existsSync(cargoToml) || !fs.existsSync(cliToml)) {
    console.log(
      'deepseek-build postinstall: source tree not present; install native bins separately (./scripts/install.sh).'
    );
    return;
  }
  if (!hasCargo()) {
    console.log(
      'deepseek-build postinstall: cargo not found.\n' +
        '  Native CLI needs Rust once (https://rustup.rs/), then either:\n' +
        '    ./scripts/install.sh\n' +
        '  or: cargo install --path crates/dsb-cli --locked --force --root ~/.deepseek-build\n' +
        '  npm package is installed; deepseek-build/dsb wrappers will print this until natives exist.'
    );
    return;
  }

  const home = process.env.DEEPSEEK_BUILD_HOME || path.join(os.homedir(), '.deepseek-build');
  console.log(
    `deepseek-build postinstall: cargo install → ${home}/bin (first build may take tens of seconds)…`
  );
  const r = spawnSync(
    'cargo',
    [
      'install',
      '--path',
      path.join(pkgRoot, 'crates', 'dsb-cli'),
      '--locked',
      '--force',
      '--root',
      home,
    ],
    { stdio: 'inherit', cwd: pkgRoot, env: process.env }
  );
  if (r.status !== 0) {
    console.warn(
      'deepseek-build postinstall: cargo install failed (npm package still installed). Run ./scripts/install.sh manually.'
    );
    // Do not fail npm install — package is still usable after manual native install.
    return;
  }
  console.log('deepseek-build postinstall: installed deepseek-build + dsb under', path.join(home, 'bin'));
}

main();
