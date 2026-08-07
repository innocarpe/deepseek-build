'use strict';

/**
 * Map Node process.platform/arch → release asset platform id (ADR 0009).
 */
function platformId() {
  const p = process.platform;
  const a = process.arch;
  if (p === 'darwin' && a === 'arm64') return 'darwin-arm64';
  if (p === 'darwin' && (a === 'x64' || a === 'ia32')) return 'darwin-x64';
  if (p === 'linux' && a === 'arm64') return 'linux-arm64';
  if (p === 'linux' && (a === 'x64' || a === 'ia32')) return 'linux-x64';
  return null;
}

function releaseAssetName(version, platform) {
  return `deepseek-build-${version}-${platform}.tar.gz`;
}

function releaseDownloadUrl(version, platform) {
  const name = releaseAssetName(version, platform);
  return `https://github.com/innocarpe/deepseek-build/releases/download/v${version}/${name}`;
}

module.exports = {
  platformId,
  releaseAssetName,
  releaseDownloadUrl,
};
