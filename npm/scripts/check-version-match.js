'use strict';

/** Fail if package.json version !== Cargo.toml workspace version. */
const fs = require('fs');
const path = require('path');

const root = path.resolve(__dirname, '..', '..');
const pkg = JSON.parse(fs.readFileSync(path.join(root, 'package.json'), 'utf8'));
const cargo = fs.readFileSync(path.join(root, 'Cargo.toml'), 'utf8');
const m = cargo.match(/^version\s*=\s*"([^"]+)"/m);
if (!m) {
  console.error('check-version-match: no version in Cargo.toml');
  process.exit(1);
}
const cargoVer = m[1];
const npmVer = pkg.version;
if (cargoVer !== npmVer) {
  console.error(`check-version-match: mismatch cargo=${cargoVer} npm=${npmVer}`);
  process.exit(1);
}
if (!/^\d+\.\d+\.\d+/.test(npmVer)) {
  console.error(`check-version-match: npm version not full SemVer: ${npmVer}`);
  process.exit(1);
}
console.log(`check-version-match: ok (${npmVer})`);
