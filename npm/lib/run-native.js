'use strict';

const { spawnSync } = require('child_process');
const fs = require('fs');
const os = require('os');
const path = require('path');

const pkgRoot = path.resolve(__dirname, '..', '..');

/**
 * Resolve native binary for deepseek-build / dsb.
 * Order: env → product home → cargo bin → package native-bin → package target/release
 */
function candidatePaths(binName) {
  const out = [];
  if (process.env.DEEPSEEK_BUILD_BIN) {
    out.push(process.env.DEEPSEEK_BUILD_BIN);
  }
  const envHome = process.env.DEEPSEEK_BUILD_HOME;
  const home = envHome || path.join(os.homedir(), '.deepseek-build');
  out.push(path.join(home, 'bin', binName));

  const cargoHome = process.env.CARGO_HOME || path.join(os.homedir(), '.cargo');
  out.push(path.join(cargoHome, 'bin', binName));

  // npm package-local copies (postinstall may place agent here)
  out.push(path.join(pkgRoot, 'npm', 'native-bin', binName));
  out.push(path.join(pkgRoot, 'target', 'release', binName));
  out.push(
    path.join(pkgRoot, 'third_party', 'grok-build', 'target', 'release', 'xai-grok-pager')
  );

  return out;
}

function findBinary(binName) {
  for (const p of candidatePaths(binName)) {
    try {
      if (fs.existsSync(p) && fs.statSync(p).isFile()) {
        // For agent name, also accept xai-grok-pager path when looking for agent
        return p;
      }
    } catch {
      // continue
    }
  }
  return null;
}

function findAgentBinary() {
  const names = [
    process.env.DEEPSEEK_BUILD_AGENT_BIN,
    path.join(
      process.env.DEEPSEEK_BUILD_HOME || path.join(os.homedir(), '.deepseek-build'),
      'bin',
      'deepseek-build-agent'
    ),
    path.join(pkgRoot, 'npm', 'native-bin', 'deepseek-build-agent'),
    path.join(pkgRoot, 'third_party', 'grok-build', 'target', 'release', 'xai-grok-pager'),
  ].filter(Boolean);
  for (const p of names) {
    try {
      if (fs.existsSync(p) && fs.statSync(p).isFile()) return p;
    } catch {
      // continue
    }
  }
  return null;
}

/**
 * Product contract: bare `dsb` / `deepseek-build` with no args on a TTY
 * must open the DeepSeek full-screen agent TUI.
 *
 * Prefer native wrapper (handles setup + splash + GROK_THEME). If wrapper
 * is missing but agent exists, exec agent directly so install still works.
 */
function run(binName, args) {
  const isBare =
    args.length === 0 ||
    (args.length === 1 && (args[0] === 'agent' || args[0] === '--'));

  // Always prefer installed wrapper when present.
  let bin = findBinary(binName);
  if (!bin && binName === 'dsb') {
    bin = findBinary('deepseek-build');
  }

  if (!bin) {
    const agent = findAgentBinary();
    if (agent && (isBare || args[0] === 'agent')) {
      const agentArgs = args[0] === 'agent' ? args.slice(1) : args;
      return exec(agent, agentArgs, productEnv());
    }
    console.error(
      `deepseek-build: native binary "${binName}" not found.\n` +
        `Tried: ${candidatePaths(binName).join(', ')}\n` +
        `Fix (no Rust required for registry install):\n` +
        `  npm install -g @innocarpe/deepseek-build\n` +
        `  # downloads prebuilts from GitHub Releases (ADR 0009)\n` +
        `  export PATH="$HOME/.deepseek-build/bin:$PATH"\n` +
        `Dev/source only: DEEPSEEK_BUILD_ALLOW_SOURCE_BUILD=1 or ./scripts/install.sh\n` +
        `Then: dsb`
    );
    process.exit(127);
  }

  return exec(bin, args, process.env);
}

function productEnv() {
  const home = process.env.DEEPSEEK_BUILD_HOME || path.join(os.homedir(), '.deepseek-build');
  const theme = process.env.DEEPSEEK_BUILD_THEME || process.env.GROK_THEME || 'deepseeknight';
  return {
    ...process.env,
    GROK_HOME: process.env.GROK_HOME || home,
    GROK_THEME: theme,
    LC_GROK_THEME: theme,
  };
}

function exec(bin, args, env) {
  const result = spawnSync(bin, args, {
    stdio: 'inherit',
    env,
  });
  if (result.error) {
    console.error(`deepseek-build: failed to spawn ${bin}: ${result.error.message}`);
    process.exit(1);
  }
  process.exit(result.status === null ? 1 : result.status);
}

module.exports = { run, findBinary, findAgentBinary, candidatePaths };
