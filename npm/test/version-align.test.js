'use strict';

const assert = require('node:assert/strict');
const test = require('node:test');

const {
  compareSemver,
  reportedVersion,
  downgradeRefusal,
  mismatchWarning,
} = require('../lib/version-align');

test('compareSemver orders releases and pre-releases', () => {
  assert.equal(compareSemver('6.0.0', '5.7.0'), 1);
  assert.equal(compareSemver('5.7.0', '6.0.0'), -1);
  assert.equal(compareSemver('6.0.0', '6.0.0'), 0);
  assert.equal(compareSemver('6.0.0', '6.0.0-alpha.1'), 1);
  assert.equal(compareSemver('6.0.0-alpha.1', '6.0.0'), -1);
  assert.equal(compareSemver('6.0.0-alpha.1', '6.0.0-alpha.2'), -1);
  assert.equal(compareSemver('nope', '6.0.0'), null);
});

test('reportedVersion reads the bake out of a channel suffix', () => {
  assert.equal(
    reportedVersion('deepseek-build 6.0.0 (87b82d2c06d2) [alpha]\n'),
    '6.0.0'
  );
  assert.equal(reportedVersion('dsb 6.0.0\n'), '6.0.0');
  assert.equal(reportedVersion('deepseek-build 6.1.0-alpha.1 (abc)\n'), '6.1.0-alpha.1');
  assert.equal(reportedVersion('not a version'), null);
});

test('downgradeRefusal blocks only a strictly newer agent', () => {
  const blocked = downgradeRefusal('6.0.0', '5.7.0', {});
  assert.match(blocked, /6\.0\.0/);
  assert.match(blocked, /5\.7\.0/);
  assert.equal(downgradeRefusal('5.7.0', '6.0.0', {}), null);
  assert.equal(downgradeRefusal('6.0.0', '6.0.0', {}), null);
  assert.equal(downgradeRefusal(null, '5.7.0', {}), null);
  assert.equal(
    downgradeRefusal('6.0.0', '5.7.0', { DEEPSEEK_BUILD_ALLOW_DOWNGRADE: '1' }),
    null
  );
});

test('mismatchWarning names both versions and stays quiet when they match', () => {
  const newer = mismatchWarning('5.7.0', '6.0.0');
  assert.match(newer, /npm package 5\.7\.0/);
  assert.match(newer, /deepseek-build-agent 6\.0\.0/);
  assert.match(newer, /DEEPSEEK_BUILD_ALLOW_DOWNGRADE/);
  const older = mismatchWarning('6.0.0', '5.7.0');
  assert.match(older, /older than this package/);
  assert.equal(mismatchWarning('6.0.0', '6.0.0'), null);
  assert.equal(mismatchWarning('6.0.0', null), null);
});
