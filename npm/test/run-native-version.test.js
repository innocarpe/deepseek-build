'use strict';

/**
 * The wrapper must not rewrite the agent's version with package.json.
 * Measured: DEEPSEEK_BUILD_VERSION=5.7.0 made a baked 6.0.0 agent print
 * `deepseek-build 5.7.0 (…) [alpha]`.
 */

const assert = require('node:assert/strict');
const test = require('node:test');

const { productEnv } = require('../lib/run-native');

function withEnv(vars, fn) {
  const saved = {};
  for (const [k, v] of Object.entries(vars)) {
    saved[k] = Object.prototype.hasOwnProperty.call(process.env, k) ? process.env[k] : undefined;
    if (v === undefined) delete process.env[k];
    else process.env[k] = v;
  }
  try {
    return fn();
  } finally {
    for (const [k, v] of Object.entries(saved)) {
      if (v === undefined) delete process.env[k];
      else process.env[k] = v;
    }
  }
}

test('productEnv does not stamp the npm package version onto the child', () => {
  const env = withEnv({ DEEPSEEK_BUILD_VERSION: undefined, GROK_TEST_VERSION: undefined }, () =>
    productEnv()
  );
  assert.equal(Object.prototype.hasOwnProperty.call(env, 'DEEPSEEK_BUILD_VERSION'), false);
  assert.equal(env.GROK_INSTALLER, process.env.GROK_INSTALLER || 'npm');
});

test('productEnv keeps a version the caller exported', () => {
  const env = withEnv({ DEEPSEEK_BUILD_VERSION: '9.9.9' }, () => productEnv());
  assert.equal(env.DEEPSEEK_BUILD_VERSION, '9.9.9');
});
