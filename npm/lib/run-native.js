'use strict';

const { spawnSync } = require('child_process');
const fs = require('fs');
const os = require('os');
const path = require('path');

/**
 * Resolve native binary for deepseek-build / dsb.
 * Order: env override → product home bin → cargo bin → package-local release build.
 */
function candidatePaths(binName) {
  const out = [];
  const envHome = process.env.DEEPSEEK_BUILD_HOME;
  const home = envHome || path.join(os.homedir(), '.deepseek-build');
  out.push(path.join(home, 'bin', binName));

  const cargoHome = process.env.CARGO_HOME || path.join(os.homedir(), '.cargo');
  out.push(path.join(cargoHome, 'bin', binName));

  // When developing from a checkout that built release bins:
  const pkgRoot = path.resolve(__dirname, '..', '..');
  out.push(path.join(pkgRoot, 'target', 'release', binName));

  if (process.env.DEEPSEEK_BUILD_BIN) {
    out.unshift(process.env.DEEPSEEK_BUILD_BIN);
  }
  return out;
}

function findBinary(binName) {
  for (const p of candidatePaths(binName)) {
    try {
      if (fs.existsSync(p) && fs.statSync(p).isFile()) {
        return p;
      }
    } catch {
      // continue
    }
  }
  return null;
}

function run(binName, args) {
  const bin = findBinary(binName);
  if (!bin) {
    console.error(
      `deepseek-build: native binary "${binName}" not found.\n` +
        `Tried: ${candidatePaths(binName).join(', ')}\n` +
        `Fix:\n` +
        `  1) From a git checkout: ./scripts/install.sh\n` +
        `  2) Or: cargo install --path crates/dsb-cli --locked --force --root ~/.deepseek-build\n` +
        `  3) Ensure ~/.deepseek-build/bin is on PATH, then re-run npm/npx.\n` +
        `Auth: ~/.deepseek-build/credentials.json or DEEPSEEK_API_KEY`
    );
    process.exit(127);
  }
  const result = spawnSync(bin, args, {
    stdio: 'inherit',
    env: process.env,
  });
  if (result.error) {
    console.error(`deepseek-build: failed to spawn ${bin}: ${result.error.message}`);
    process.exit(1);
  }
  process.exit(result.status === null ? 1 : result.status);
}

module.exports = { run, findBinary, candidatePaths };
