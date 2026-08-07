const assert = require('node:assert/strict');
const test = require('node:test');

const { platformId } = require('../lib/platform');

test('supports Apple Silicon macOS only', () => {
  assert.equal(platformId('darwin', 'arm64'), 'darwin-arm64');
  assert.equal(platformId('darwin', 'x64'), null);
  assert.equal(platformId('linux', 'arm64'), null);
  assert.equal(platformId('linux', 'x64'), null);
  assert.equal(platformId('win32', 'arm64'), null);
});
