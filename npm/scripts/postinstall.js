'use strict';

/**
 * After `npm install -g @innocarpe/deepseek-build`:
 * 1) Install native wrapper (deepseek-build + dsb) under ~/.deepseek-build/bin
 * 2) Build DeepSeek full-screen agent TUI and install as deepseek-build-agent
 *
 * Product contract: `dsb` with no args on a TTY opens the DeepSeek TUI.
 * Requires Rust (cargo). First agent build can take several minutes.
 * Never hard-fails npm install (wrappers still land; missing agent prints fix).
 */

const { spawnSync } = require('child_process');
const fs = require('fs');
const os = require('os');
const path = require('path');

const pkgRoot = path.resolve(__dirname, '..', '..');
const cargoToml = path.join(pkgRoot, 'Cargo.toml');
const cliToml = path.join(pkgRoot, 'crates', 'dsb-cli', 'Cargo.toml');
const vendorRoot = path.join(pkgRoot, 'third_party', 'grok-build');
const buildAgentScript = path.join(pkgRoot, 'scripts', 'build-grok-pager.sh');

function hasCmd(cmd) {
  const r = spawnSync(cmd, ['--version'], { encoding: 'utf8' });
  return r.status === 0;
}

function which(cmd) {
  const r = spawnSync(process.platform === 'win32' ? 'where' : 'which', [cmd], {
    encoding: 'utf8',
  });
  if (r.status !== 0) return null;
  return (r.stdout || '').trim().split(/\r?\n/)[0] || null;
}

function ensureDir(p) {
  fs.mkdirSync(p, { recursive: true });
}

function copyFile(src, dest) {
  fs.copyFileSync(src, dest);
  try {
    fs.chmodSync(dest, 0o755);
  } catch {
    // windows
  }
}

function main() {
  if (process.env.DEEPSEEK_BUILD_SKIP_POSTINSTALL === '1') {
    console.log('deepseek-build postinstall: skipped (DEEPSEEK_BUILD_SKIP_POSTINSTALL=1)');
    return;
  }
  if (!fs.existsSync(cargoToml) || !fs.existsSync(cliToml)) {
    console.log(
      'deepseek-build postinstall: source tree not present; run from published package with crates/.'
    );
    return;
  }
  if (!hasCmd('cargo')) {
    console.log(
      'deepseek-build postinstall: cargo not found.\n' +
        '  Install Rust once: https://rustup.rs/\n' +
        '  Then re-run: npm install -g @innocarpe/deepseek-build\n' +
        '  Or from the package dir: ./scripts/install.sh'
    );
    return;
  }

  // Prefer homebrew / cargo bin for protoc + dotslash on macOS.
  const pathParts = [
    path.join(os.homedir(), '.cargo', 'bin'),
    '/opt/homebrew/bin',
    '/usr/local/bin',
    process.env.PATH || '',
  ];
  const env = { ...process.env, PATH: pathParts.join(path.delimiter) };

  const home = process.env.DEEPSEEK_BUILD_HOME || path.join(os.homedir(), '.deepseek-build');
  const binDir = path.join(home, 'bin');
  ensureDir(binDir);

  console.log(`deepseek-build postinstall: installing wrapper → ${binDir}`);
  console.log('  (cargo install dsb-cli — usually under a minute)');
  const wrap = spawnSync(
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
    { stdio: 'inherit', cwd: pkgRoot, env }
  );
  if (wrap.status !== 0) {
    console.warn(
      'deepseek-build postinstall: wrapper cargo install failed.\n' +
        '  Retry: ./scripts/install.sh from the package directory.'
    );
    return;
  }
  console.log('deepseek-build postinstall: wrapper OK (deepseek-build + dsb)');

  if (process.env.DEEPSEEK_BUILD_SKIP_AGENT_BUILD === '1') {
    console.log(
      'deepseek-build postinstall: agent build skipped (DEEPSEEK_BUILD_SKIP_AGENT_BUILD=1).\n' +
        '  Bare `dsb` TUI needs deepseek-build-agent — run without skip later.'
    );
    printPathHint(binDir);
    return;
  }

  if (!fs.existsSync(vendorRoot) || !fs.existsSync(buildAgentScript)) {
    console.warn(
      'deepseek-build postinstall: vendored agent tree missing; cannot build TUI agent.\n' +
        '  Package must include third_party/grok-build and scripts/build-grok-pager.sh.'
    );
    printPathHint(binDir);
    return;
  }

  if (!which('protoc') && !which('dotslash')) {
    console.warn(
      'deepseek-build postinstall: need protoc or dotslash for agent build.\n' +
        '  macOS: brew install protobuf && cargo install dotslash --locked\n' +
        '  Then: npm install -g @innocarpe/deepseek-build  (or ./scripts/install.sh)'
    );
    // still leave wrapper installed
    printPathHint(binDir);
    return;
  }

  console.log(
    'deepseek-build postinstall: building DeepSeek full-screen agent TUI…\n' +
      '  First build can take several minutes (Rust + vendor tree).'
  );
  const agentBuild = spawnSync('bash', [buildAgentScript, 'release'], {
    stdio: 'inherit',
    cwd: pkgRoot,
    env,
  });
  if (agentBuild.status !== 0) {
    console.warn(
      'deepseek-build postinstall: agent build failed.\n' +
        '  Wrapper is installed; fix toolchain (protoc/dotslash/Rust) and re-run:\n' +
        '    cd "$(npm root -g)/@innocarpe/deepseek-build" && ./scripts/install.sh'
    );
    printPathHint(binDir);
    return;
  }

  const candidates = [
    path.join(vendorRoot, 'target', 'release', 'xai-grok-pager'),
    path.join(vendorRoot, 'target', 'release', 'xai-grok-pager-bin'),
  ];
  let agentSrc = null;
  for (const c of candidates) {
    if (fs.existsSync(c)) {
      agentSrc = c;
      break;
    }
  }
  if (!agentSrc) {
    console.warn('deepseek-build postinstall: agent binary not found after build.');
    printPathHint(binDir);
    return;
  }

  const agentDest = path.join(binDir, 'deepseek-build-agent');
  copyFile(agentSrc, agentDest);
  // Also next to npm package for run-native fallback
  try {
    const pkgBin = path.join(pkgRoot, 'npm', 'native-bin');
    ensureDir(pkgBin);
    copyFile(agentSrc, path.join(pkgBin, 'deepseek-build-agent'));
  } catch {
    // optional
  }

  console.log('deepseek-build postinstall: agent OK →', agentDest);
  console.log('');
  console.log('DeepSeek Build ready:');
  console.log('  export PATH="' + binDir + ':$PATH"');
  console.log('  dsb          # full-screen DeepSeek TUI');
  console.log('  dsb setup    # API key if needed');
  printPathHint(binDir);
}

function printPathHint(binDir) {
  const pathEnv = process.env.PATH || '';
  if (!pathEnv.split(path.delimiter).includes(binDir)) {
    console.log('');
    console.log('Add to PATH (if `dsb` is not found):');
    console.log(`  export PATH="${binDir}:$PATH"`);
    console.log('  # permanent (zsh): echo \'export PATH="' + binDir + ':$PATH"\' >> ~/.zshrc');
  }
}

main();
