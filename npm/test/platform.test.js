const assert = require('node:assert/strict');
const test = require('node:test');

const { platformId } = require('../lib/platform');

test('supports the two released platforms', () => {
  assert.equal(platformId('darwin', 'arm64'), 'darwin-arm64');
  assert.equal(platformId('linux', 'x64'), 'linux-x64');
  assert.equal(platformId('darwin', 'x64'), null);
  assert.equal(platformId('linux', 'arm64'), null);
  assert.equal(platformId('win32', 'arm64'), null);
});
